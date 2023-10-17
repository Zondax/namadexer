use crate::{config::DatabaseConfig, error::Error, utils};
use borsh::de::BorshDeserialize;

use namada::proto;
use namada::types::{eth_bridge_pool::PendingTransfer, token, transaction, transaction::TxType};
use sqlx::postgres::{PgPool, PgPoolOptions, PgRow as Row};
use sqlx::{query, QueryBuilder, Transaction};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tendermint::block::Block;
use tendermint_proto::types::evidence::Sum;
use tendermint_proto::types::EvidenceList as RawEvidenceList;
use tracing::{info, instrument};

use crate::{
    DB_SAVE_BLOCK_COUNTER, DB_SAVE_BLOCK_DURATION, DB_SAVE_EVDS_DURATION, DB_SAVE_TXS_DURATION,
    MASP_ADDR,
};

use crate::tables::{
    get_create_block_table_query, get_create_evidences_table_query,
    get_create_transactions_table_query, get_create_tx_bond_table_query,
    get_create_tx_bridge_pool_table_query, get_create_tx_transfer_table_query,
};

use metrics::{histogram, increment_counter};

const BLOCKS_TABLE_NAME: &str = "blocks";
const TX_TABLE_NAME: &str = "transactions";

// Max time to wait for a succesfull database connection
const DATABASE_TIMEOUT: u64 = 60;

#[derive(Clone)]
pub struct Database {
    pool: Arc<PgPool>,
    // we use the network as the name of the schema to allow diffrent net on the same database
    network: String,
}

impl Database {
    pub async fn new(db_config: &DatabaseConfig, network: &str) -> Result<Database, Error> {
        // sqlx expects config of the form:
        // postgres://user:password@host/db_name
        let config = format!(
            "postgres://{}:{}@{}/{}",
            db_config.user, db_config.password, db_config.host, db_config.dbname
        );

        // If timeout setting is not present in the provided configuration,
        // lets use our default timeout.
        let timeout = db_config.connection_timeout.unwrap_or(DATABASE_TIMEOUT);

        let pool = PgPoolOptions::new()
            .max_connections(10)
            .acquire_timeout(Duration::from_secs(timeout))
            .connect(&config)
            .await?;

        let network_schema = network.replace('-', "_");

        Ok(Database {
            pool: Arc::new(pool),
            network: network_schema.to_string(),
        })
    }

    pub fn with_pool(pool: PgPool, network: String) -> Self {
        Self {
            pool: Arc::new(pool),
            network,
        }
    }

    pub async fn transaction<'a>(&self) -> Result<sqlx::Transaction<'a, sqlx::Postgres>, Error> {
        self.pool.begin().await.map_err(Error::from)
    }

    /// Create required tables in the database.
    /// these tables are:
    /// - `blocks` to store relevant information about blocks, for example its id, commits
    /// and block_header
    /// - `transactions` although part of the block data, they are store in a different table
    /// and contain useful information about transactions.
    /// - `evidences` Where block's evidence data is stored.
    #[instrument(skip(self))]
    pub async fn create_tables(&self) -> Result<(), Error> {
        info!("Creating tables if doesn't exist");

        query(format!("CREATE SCHEMA IF NOT EXISTS {}", self.network).as_str())
            .execute(&*self.pool)
            .await?;

        query(get_create_block_table_query(&self.network).as_str())
            .execute(&*self.pool)
            .await?;

        query(get_create_transactions_table_query(&self.network).as_str())
            .execute(&*self.pool)
            .await?;

        query(get_create_evidences_table_query(&self.network).as_str())
            .execute(&*self.pool)
            .await?;

        query(get_create_tx_transfer_table_query(&self.network).as_str())
            .execute(&*self.pool)
            .await?;

        query(get_create_tx_bond_table_query(&self.network).as_str())
            .execute(&*self.pool)
            .await?;

        query(get_create_tx_bridge_pool_table_query(&self.network).as_str())
            .execute(&*self.pool)
            .await?;

        Ok(())
    }

    /// Inner implementation that uses a postgres-transaction
    /// to ensure database coherence.
    #[instrument(skip(block, checksums_map, sqlx_tx))]
    async fn save_block_impl<'a>(
        block: &Block,
        checksums_map: &HashMap<String, String>,
        sqlx_tx: &mut Transaction<'a, sqlx::Postgres>,
        network: &str,
    ) -> Result<(), Error> {
        let mut query_builder: QueryBuilder<_> = QueryBuilder::new(format!(
            "INSERT INTO {}.blocks(
                block_id,
                header_version_app,
                header_version_block,
                header_chain_id,
                header_height,
                header_time,
                header_last_block_id_hash,
                header_last_block_id_parts_header_total,
                header_last_block_id_parts_header_hash,
                header_last_commit_hash,
                header_data_hash,
                header_validators_hash,
                header_next_validators_hash,
                header_consensus_hash,
                header_app_hash,
                header_last_results_hash,
                header_evidence_hash,
                header_proposer_address,
                commit_height,
                commit_round,
                commit_block_id_hash,
                commit_block_id_parts_header_total,
                commit_block_id_parts_header_hash
            )",
            network
        ));
        let block_id = block.header.hash().as_bytes().to_vec();

        let query_block = query_builder
            .push_values(std::iter::once(0), |mut b, _| {
                b.push_bind(block_id.clone())
                    .push_bind(block.header.version.app as i32)
                    .push_bind(block.header.version.block as i32)
                    .push_bind(block.header.chain_id.as_str())
                    .push_bind(block.header.height.value() as i32)
                    .push_bind(block.header.time.to_rfc3339())
                    .push_bind(
                        block
                            .header
                            .last_block_id
                            .map(|id| id.hash.as_bytes().to_vec()),
                    )
                    .push_bind(
                        block
                            .header
                            .last_block_id
                            .map(|id| id.part_set_header.total as i32),
                    )
                    .push_bind(
                        block
                            .header
                            .last_block_id
                            .map(|id| id.part_set_header.hash.as_bytes().to_vec()),
                    )
                    .push_bind(
                        block
                            .header
                            .last_commit_hash
                            .map(|lch| lch.as_bytes().to_vec()),
                    )
                    .push_bind(block.header.data_hash.map(|dh| dh.as_bytes().to_vec()))
                    .push_bind(block.header.validators_hash.as_bytes().to_vec())
                    .push_bind(block.header.next_validators_hash.as_bytes().to_vec())
                    .push_bind(block.header.consensus_hash.as_bytes().to_vec())
                    .push_bind(block.header.app_hash.to_string())
                    .push_bind(
                        block
                            .header
                            .last_results_hash
                            .map(|lrh| lrh.as_bytes().to_vec()),
                    )
                    .push_bind(block.header.evidence_hash.map(|eh| eh.as_bytes().to_vec()))
                    .push_bind(block.header.proposer_address.to_string())
                    .push_bind(block.last_commit.as_ref().map(|c| c.height.value() as i32))
                    .push_bind(block.last_commit.as_ref().map(|c| c.round.value() as i32))
                    .push_bind(
                        block
                            .last_commit
                            .as_ref()
                            .map(|c| c.block_id.hash.as_bytes().to_vec()),
                    )
                    .push_bind(
                        block
                            .last_commit
                            .as_ref()
                            .map(|c| c.block_id.part_set_header.total as i32),
                    )
                    .push_bind(
                        block
                            .last_commit
                            .as_ref()
                            .map(|c| c.block_id.part_set_header.hash.as_bytes().to_vec()),
                    );
            })
            .build();

        query_block.execute(&mut *sqlx_tx).await?;

        let evidence_list = RawEvidenceList::from(block.evidence().clone());
        Self::save_evidences(evidence_list, &block_id, sqlx_tx, network).await?;
        Self::save_transactions(
            block.data.as_ref(),
            &block_id,
            checksums_map,
            sqlx_tx,
            network,
        )
        .await?;

        Ok(())
    }

    /// Save a block and commit database
    #[instrument(skip(self, block, checksums_map))]
    pub async fn save_block(
        &self,
        block: &Block,
        checksums_map: &HashMap<String, String>,
    ) -> Result<(), Error> {
        let instant = tokio::time::Instant::now();
        // Lets use postgres transaction internally for 2 reasons:
        // - A block could contain many evidences and Txs, so this approach allows
        // saving all of them and commit at the end.
        // - Errors could happen in the middle either while processing
        // transactions, evidences or blocks. with postgres-transaction
        // we ensure database integrity, and commit only if all operations
        // succeeded.
        let mut sqlx_tx = self.transaction().await?;

        Self::save_block_impl(block, checksums_map, &mut sqlx_tx, self.network.as_str()).await?;

        let res = sqlx_tx.commit().await.map_err(Error::from);

        let dur = instant.elapsed();

        let mut status = "Ok".to_string();
        if let Err(e) = &res {
            status = e.to_string();
        }

        let labels = [
            ("save_block", block.header.height.value().to_string()),
            ("status", status),
        ];

        histogram!(DB_SAVE_BLOCK_DURATION, dur.as_secs_f64() * 1000.0, &labels);

        if res.is_ok() {
            // update our counter for processed blocks since service started.
            increment_counter!(DB_SAVE_BLOCK_COUNTER, &labels);
        }

        res
    }

    /// Save a block, the operation uses a sqlx::Transaction,
    /// It is up to the caller to commit the operation.
    /// this method is meant to be used when caller is saving
    /// many blocks, and can commit after it.
    #[instrument(skip(block, checksums_map, sqlx_tx, network))]
    pub async fn save_block_tx<'a>(
        block: &Block,
        checksums_map: &HashMap<String, String>,
        sqlx_tx: &mut Transaction<'a, sqlx::Postgres>,
        network: &str,
    ) -> Result<(), Error> {
        Self::save_block_impl(block, checksums_map, sqlx_tx, network).await
    }

    /// Save all the evidences in the list, it is up to the caller to
    /// call sqlx_tx.commit().await?; for the changes to take place in
    /// database.
    #[instrument(skip(evidences, block_id, sqlx_tx))]
    async fn save_evidences<'a>(
        evidences: RawEvidenceList,
        block_id: &[u8],
        sqlx_tx: &mut Transaction<'a, sqlx::Postgres>,
        network: &str,
    ) -> Result<(), Error> {
        info!("saving evidences");

        let mut query_builder: QueryBuilder<_> = QueryBuilder::new(format!(
            "INSERT INTO {}.evidences(
                    block_id,
                    height,
                    time,
                    address,
                    total_voting_power,
                    validator_power
            )",
            network
        ));

        let instant = tokio::time::Instant::now();

        // Same as transactions regarding limitations in field binding
        // in postgres, but it is unlikely to have such hight amount
        // of evidences per block.
        // (block_id, height, time, address, total_voting_power, validator_power)
        let mut evidences_data = Vec::new();

        for evidence in evidences.evidence {
            let Some(s) = evidence.sum else {
                tracing::debug!("No evidence");
                continue;
            };

            match s {
                Sum::DuplicateVoteEvidence(dve) => {
                    evidences_data.push((
                        block_id,
                        dve.vote_a.as_ref().map(|v| v.height),
                        dve.vote_a
                            .as_ref()
                            .and_then(|v| v.timestamp.as_ref())
                            .map(|t| t.seconds.to_string()),
                        dve.vote_a.as_ref().map(|v| v.validator_address.clone()),
                        dve.total_voting_power,
                        dve.validator_power,
                    ));
                }
                _ => tracing::warn!("Unknown evidence."),
            }
        }

        let num_evidences = evidences_data.len();

        if num_evidences == 0 {
            let labels = [
                ("bulk_insert", "evidences".to_string()),
                ("status", "Ok".to_string()),
                ("num_evidences", num_evidences.to_string()),
            ];
            let dur = instant.elapsed();
            histogram!(DB_SAVE_EVDS_DURATION, dur.as_secs_f64() * 1000.0, &labels);

            return Ok(());
        }

        let res = query_builder
            .push_values(
                evidences_data.into_iter(),
                |mut b, (block_id, height, time, address, total_voting_power, validator_power)| {
                    b.push_bind(block_id)
                        .push_bind(height)
                        .push_bind(time)
                        .push_bind(address)
                        .push_bind(total_voting_power)
                        .push_bind(validator_power);
                },
            )
            .build()
            .execute(&mut *sqlx_tx)
            .await
            .map(|_| ())
            .map_err(Error::from);

        let dur = instant.elapsed();

        let mut status = "Ok".to_string();
        if let Err(e) = &res {
            status = e.to_string();
        }

        let labels = [
            ("bulk_insert", "evidences".to_string()),
            ("status", status),
            ("num_evidences", num_evidences.to_string()),
        ];

        histogram!(DB_SAVE_EVDS_DURATION, dur.as_secs_f64() * 1000.0, &labels);

        res
    }

    /// Save all the transactions in txs, it is up to the caller to
    /// call sqlx_tx.commit().await?; for the changes to take place in
    /// database.
    #[instrument(skip(txs, block_id, sqlx_tx, checksums_map))]
    async fn save_transactions<'a>(
        txs: &Vec<Vec<u8>>,
        block_id: &[u8],
        checksums_map: &HashMap<String, String>,
        sqlx_tx: &mut Transaction<'a, sqlx::Postgres>,
        network: &str,
    ) -> Result<(), Error> {
        // use for metrics
        let instant = tokio::time::Instant::now();

        if txs.is_empty() {
            let labels = [
                ("bulk_insert", "transactions".to_string()),
                ("status", "Ok".to_string()),
                ("num_transactions", 0.to_string()),
            ];

            let dur = instant.elapsed();

            histogram!(DB_SAVE_TXS_DURATION, dur.as_secs_f64() * 1000.0, &labels);
            return Ok(());
        }

        info!(message = "Saving transactions");
        let mut query_builder: QueryBuilder<_> = QueryBuilder::new(format!(
            "INSERT INTO {}.transactions(
                    hash, 
                    block_id, 
                    tx_type,
                    code,
                    data
                )",
            network
        ));

        // this will holds tuples (hash, block_id, tx_type, code, data)
        // in order to push txs.len at once in a single query.
        // the limit for bind values in postgres is 65535 values, that means that
        // to hit that limit a block would need to have:
        // n_tx = 65535/5 = 13107
        // being 5 the number of columns.
        let mut tx_values = Vec::with_capacity(txs.len());

        for t in txs.iter() {
            let tx = proto::Tx::try_from(t.as_slice()).map_err(|_| Error::InvalidTxData)?;
            let mut code = Default::default();

            // Decrypted transaction give access to the raw data
            if let TxType::Decrypted(..) = tx.header().tx_type {
                code = tx
                    .get_section(tx.code_sechash())
                    .and_then(|s| s.code_sec())
                    .map(|s| s.code.hash().0)
                    .ok_or(Error::InvalidTxData)?;

                let code_hex = hex::encode(code.as_slice());

                let unknown_type = "unknown".to_string();
                let type_tx = checksums_map.get(&code_hex).unwrap_or(&unknown_type);

                // decode tx_transfer, tx_bond and tx_unbound to store the decoded data in their tables
                match type_tx.as_str() {
                    "tx_transfer" => {
                        info!("Saving tx_transfer");
                        let data = tx.data().ok_or(Error::InvalidTxData)?;
                        let transfer = token::Transfer::try_from_slice(&data[..])?;

                        let mut query_builder: QueryBuilder<_> = QueryBuilder::new(format!(
                            "INSERT INTO {}.tx_transfer(
                                tx_id,
                                source, 
                                target, 
                                token,
                                amount,
                                key,
                                shielded
                            )",
                            network
                        ));

                        let query = query_builder
                            .push_values(std::iter::once(0), |mut b, _| {
                                b.push_bind(tx.header_hash().0.as_slice().to_vec())
                                    .push_bind(transfer.source.to_string())
                                    .push_bind(transfer.target.to_string())
                                    .push_bind(transfer.token.to_string())
                                    .push_bind(transfer.amount.to_string())
                                    .push_bind(transfer.key.as_ref().map(|k| k.to_string()))
                                    .push_bind(transfer.shielded.as_ref().map(|s| s.to_vec()));
                            })
                            .build();
                        query.execute(&mut *sqlx_tx).await?;
                    }
                    "tx_bond" => {
                        info!("Saving tx_bond");
                        let data = tx.data().ok_or(Error::InvalidTxData)?;
                        let bond = transaction::pos::Bond::try_from_slice(&data[..])?;

                        let mut query_builder: QueryBuilder<_> = QueryBuilder::new(format!(
                            "INSERT INTO {}.tx_bond(
                                tx_id,
                                validator,
                                amount,
                                source,
                                bond
                            )",
                            network
                        ));

                        let query = query_builder
                            .push_values(std::iter::once(0), |mut b, _| {
                                b.push_bind(tx.header_hash().0.as_slice().to_vec())
                                    .push_bind(bond.validator.to_string())
                                    .push_bind(bond.amount.to_string_native())
                                    .push_bind(bond.source.as_ref().map(|s| s.to_string()))
                                    .push_bind(true);
                            })
                            .build();
                        query.execute(&mut *sqlx_tx).await?;
                    }
                    "tx_unbond" => {
                        info!("Saving tx_unbond");
                        let data = tx.data().ok_or(Error::InvalidTxData)?;
                        let unbond = transaction::pos::Unbond::try_from_slice(&data[..])?;

                        let mut query_builder: QueryBuilder<_> = QueryBuilder::new(format!(
                            "INSERT INTO {}.tx_bond(
                                tx_id,
                                validator,
                                amount,
                                source,
                                bond
                            )",
                            network
                        ));

                        let query = query_builder
                            .push_values(std::iter::once(0), |mut b, _| {
                                b.push_bind(tx.header_hash().0.as_slice().to_vec())
                                    .push_bind(unbond.validator.to_string())
                                    .push_bind(unbond.amount.to_string_native())
                                    .push_bind(
                                        unbond
                                            .source
                                            .as_ref()
                                            .map_or("".to_string(), |s| s.to_string()),
                                    )
                                    .push_bind(false);
                            })
                            .build();
                        query.execute(&mut *sqlx_tx).await?;
                    }
                    // this is an ethereum transaction
                    "tx_bridge_pool" => {
                        info!("Saving tx_bridge_pool");
                        let data = tx.data().ok_or(Error::InvalidTxData)?;
                        // Only TransferToEthereum type is supported at the moment by namada and us.
                        let tx_bridge = PendingTransfer::try_from_slice(&data[..])?;

                        let mut query_builder: QueryBuilder<_> = QueryBuilder::new(format!(
                            "INSERT INTO {}.tx_bridge_pool(
                                tx_id,
                                asset,
                                recipient,
                                sender,
                                amount,
                                gas_amount,
                                payer
                            )",
                            network
                        ));

                        let query = query_builder
                            .push_values(std::iter::once(0), |mut b, _| {
                                b.push_bind(tx.header_hash().0.as_slice().to_vec())
                                    .push_bind(tx_bridge.transfer.asset.to_string())
                                    .push_bind(tx_bridge.transfer.recipient.to_string())
                                    .push_bind(tx_bridge.transfer.sender.to_string())
                                    .push_bind(tx_bridge.transfer.amount.to_string_native())
                                    .push_bind(tx_bridge.gas_fee.amount.to_string_native())
                                    .push_bind(tx_bridge.gas_fee.payer.to_string())
                                    .push_bind(false);
                            })
                            .build();
                        query.execute(&mut *sqlx_tx).await?;
                    }
                    _ => {}
                }
            }

            tx_values.push((
                tx.header_hash().0.as_slice().to_vec(),
                block_id.to_vec(),
                utils::tx_type_name(&tx.header.tx_type),
                code,
                tx.data().map(|v| v.to_vec()),
            ));
        }

        let num_transactions = tx_values.len();

        // bulk insert to speed-up this
        // there might be limits regarding the number of parameter
        // but number of transaction is low in comparisson with
        // postgres limit
        let res = query_builder
            .push_values(
                tx_values.into_iter(),
                |mut b, (hash, block_id, tx_type, code, data)| {
                    b.push_bind(hash)
                        .push_bind(block_id)
                        .push_bind(tx_type)
                        .push_bind(code)
                        .push_bind(data);
                },
            )
            .build()
            .execute(&mut *sqlx_tx)
            .await
            .map(|_| ())
            .map_err(Error::from);

        let dur = instant.elapsed();

        let mut status = "Ok".to_string();
        if let Err(e) = &res {
            status = e.to_string();
        }

        let labels = [
            ("bulk_insert", "transactions".to_string()),
            ("status", status),
            ("num_transactions", num_transactions.to_string()),
        ];

        histogram!(DB_SAVE_TXS_DURATION, dur.as_secs_f64() * 1000.0, &labels);

        res
    }

    pub async fn create_indexes(&self) -> Result<(), Error> {
        // we create indexes on the tables to facilitate querying data
        query(
            format!(
                "
                ALTER TABLE {}.blocks ADD CONSTRAINT pk_block_id PRIMARY KEY (block_id);
            ",
                self.network
            )
            .as_str(),
        )
        .execute(&*self.pool)
        .await?;

        query(
            format!(
                "CREATE UNIQUE INDEX ux_header_height ON {}.blocks (header_height);",
                self.network
            )
            .as_str(),
        )
        .execute(&*self.pool)
        .await?;

        query(
            format!(
                "ALTER TABLE {}.transactions ADD CONSTRAINT pk_hash PRIMARY KEY (hash);",
                self.network
            )
            .as_str(),
        )
        .execute(&*self.pool)
        .await?;

        query(format!("ALTER TABLE {0}.transactions ADD CONSTRAINT fk_block_id FOREIGN KEY (block_id) REFERENCES {0}.blocks (block_id);", self.network).as_str())
            .execute(&*self.pool)
            .await?;

        query(
            format!(
                "ALTER TABLE {}.tx_transfer ADD CONSTRAINT pk_tx_id_transfer PRIMARY KEY (tx_id);",
                self.network
            )
            .as_str(),
        )
        .execute(&*self.pool)
        .await?;

        query(
            format!(
                "CREATE INDEX x_source_transfer ON {}.tx_transfer USING HASH (source);",
                self.network
            )
            .as_str(),
        )
        .execute(&*self.pool)
        .await?;

        query(
            format!(
                "CREATE INDEX x_target_transfer ON {}.tx_transfer USING HASH (target);",
                self.network
            )
            .as_str(),
        )
        .execute(&*self.pool)
        .await?;

        query(
            format!(
                "ALTER TABLE {}.tx_bond ADD CONSTRAINT pk_tx_id_bond PRIMARY KEY (tx_id);",
                self.network
            )
            .as_str(),
        )
        .execute(&*self.pool)
        .await?;

        query(
            format!(
                "ALTER TABLE {}.tx_bridge_pool ADD CONSTRAINT pk_tx_id_bridge PRIMARY KEY (tx_id);",
                self.network
            )
            .as_str(),
        )
        .execute(&*self.pool)
        .await?;

        query(
            format!(
                "CREATE INDEX x_validator_bond ON {}.tx_bond USING HASH (validator);",
                self.network
            )
            .as_str(),
        )
        .execute(&*self.pool)
        .await?;

        query(
            format!(
                "CREATE INDEX x_source_bond ON {}.tx_bond USING HASH (source);",
                self.network
            )
            .as_str(),
        )
        .execute(&*self.pool)
        .await?;

        Ok(())
    }

    #[instrument(skip(self, block_id))]
    pub async fn block_by_id(&self, block_id: &[u8]) -> Result<Option<Row>, Error> {
        // query for the block if it exists
        let str = format!(
            "SELECT * FROM {}.{BLOCKS_TABLE_NAME} WHERE block_id=$1",
            self.network
        );
        query(&str)
            .bind(block_id)
            .fetch_optional(&*self.pool)
            .await
            .map_err(Error::from)
    }

    /// Returns the block at `block_height` if present, otherwise returns an Error.
    #[instrument(skip(self))]
    pub async fn block_by_height(&self, block_height: u32) -> Result<Option<Row>, Error> {
        let str = format!(
            "SELECT * FROM {}.{BLOCKS_TABLE_NAME} WHERE header_height={block_height}",
            self.network
        );

        query(&str)
            .fetch_optional(&*self.pool)
            .await
            .map_err(Error::from)
    }

    #[instrument(skip(self))]
    /// Returns the latest block, otherwise returns an Error.
    pub async fn get_last_block(&self) -> Result<Row, Error> {
        let str = format!("SELECT * FROM {0}.{BLOCKS_TABLE_NAME} WHERE header_height = (SELECT MAX(header_height) FROM {0}.{BLOCKS_TABLE_NAME})", self.network);

        // use query_one as the row matching max height is unique.
        query(&str)
            .fetch_one(&*self.pool)
            .await
            .map_err(Error::from)
    }

    #[instrument(skip(self))]
    /// Returns the latest height value, otherwise returns an Error.
    pub async fn get_last_height(&self) -> Result<Row, Error> {
        let str = format!(
            "SELECT MAX(header_height) AS header_height FROM {}.{BLOCKS_TABLE_NAME}",
            self.network
        );

        // use query_one as the row matching max height is unique.
        query(&str)
            .fetch_one(&*self.pool)
            .await
            .map_err(Error::from)
    }

    #[instrument(skip(self))]
    /// Returns Transaction identified by hash
    pub async fn get_tx(&self, hash: &[u8]) -> Result<Option<Row>, Error> {
        // query for transaction with hash
        let str = format!(
            "SELECT * FROM {}.{TX_TABLE_NAME} WHERE hash=$1",
            self.network
        );

        query(&str)
            .bind(hash)
            .fetch_optional(&*self.pool)
            .await
            .map_err(Error::from)
    }

    #[instrument(skip(self))]
    /// Returns all the tx hashes for a block
    pub async fn get_tx_hashes_block(&self, hash: &[u8]) -> Result<Vec<Row>, Error> {
        // query for all tx hash that are in a block identified by the block_id
        let str = format!("SELECT t.hash FROM {0}.{BLOCKS_TABLE_NAME} b JOIN {0}.{TX_TABLE_NAME} t ON b.block_id = t.block_id WHERE b.block_id =$1;", self.network);

        query(&str)
            .bind(hash)
            .fetch_all(&*self.pool)
            .await
            .map_err(Error::from)
    }

    #[instrument(skip(self))]
    /// Returns Shielded transactions
    pub async fn get_shielded_tx(&self) -> Result<Vec<Row>, Error> {
        // query for transaction with hash
        let str = format!(
            "SELECT * FROM {}.tx_transfer WHERE source = '{MASP_ADDR}' OR target = '{MASP_ADDR}'",
            self.network
        );

        query(&str)
            .fetch_all(&*self.pool)
            .await
            .map_err(Error::from)
    }

    #[instrument(skip(self))]
    /// Returns info about the indexes existing on the table, otherwise returns an Error.
    pub async fn check_indexes(&self) -> Result<Vec<Row>, Error> {
        let str = format!(
            "SELECT indexname, indexdef FROM pg_indexes WHERE tablename = '{BLOCKS_TABLE_NAME}';"
        );

        query(&str)
            .fetch_all(&*self.pool)
            .await
            .map_err(Error::from)
    }

    pub fn pool(&self) -> &PgPool {
        self.pool.as_ref()
    }
}
