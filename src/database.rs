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

        query(format!("CREATE SCHEMA {}", self.network).as_str())
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
        let mut copy_in_block = sqlx_tx.copy_in_raw(&format!("COPY {}.blocks FROM stdin (DELIMITER ',', NULL '')", network)).await?;

        let block_id = block.header.hash().as_bytes().to_vec();

        let statement = format!(
            "\\\\x{},{},{},{},{},{},\\\\x{},{},\\\\x{},\\\\x{},\\\\x{},\\\\x{},\\\\x{},\\\\x{},{},\\\\x{},\\\\x{},{},{},{},\\\\x{},{},\\\\x{}",
            hex::encode(&block_id),
            block.header.version.app,
            block.header.version.block,
            block.header.chain_id.as_str(),
            block.header.height.value(),
            block.header.time.to_rfc3339(),
            block.header.last_block_id.map_or("".to_string(), |id| hex::encode(id.hash.as_bytes().to_vec())),
            block.header.last_block_id.map_or("".to_string(),|id| id.part_set_header.total.to_string()),
            block.header.last_block_id.map_or("".to_string(), |id| hex::encode(id.part_set_header.hash.as_bytes().to_vec())),
            block.header.last_commit_hash.map_or("".to_string(), |lch| hex::encode(lch.as_bytes().to_vec())),
            block.header.data_hash.map_or("".to_string(),|dh| hex::encode(dh.as_bytes().to_vec())),
            hex::encode(block.header.validators_hash.as_bytes().to_vec()),
            hex::encode(block.header.next_validators_hash.as_bytes().to_vec()),
            hex::encode(block.header.consensus_hash.as_bytes().to_vec()),
            block.header.app_hash.to_string(),
            block.header.last_results_hash.map_or("".to_string(),|lrh| hex::encode(lrh.as_bytes().to_vec())),
            block.header.evidence_hash.map_or("".to_string(), |eh| hex::encode(eh.as_bytes().to_vec())),
            block.header.proposer_address.to_string(),
            block.last_commit.as_ref().map_or("".to_string(), |c| c.height.value().to_string()),
            block.last_commit.as_ref().map_or("".to_string(), |c| c.round.value().to_string()),
            block.last_commit.as_ref().map_or("".to_string(), |c| hex::encode(c.block_id.hash.as_bytes().to_vec())),
            block.last_commit.as_ref().map_or("".to_string(), |c| c.block_id.part_set_header.total.to_string()),
            block.last_commit.as_ref().map_or("".to_string(), |c| hex::encode(c.block_id.part_set_header.hash.as_bytes().to_vec())),
        );

        copy_in_block.send(statement.as_bytes()).await?;
        let row_count = copy_in_block.finish().await?;

        println!("saved block : {}", row_count);
        println!("Evidences save");

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

        let mut statement: String = String::new();
        for (block_id, height, time, address, total_voting_power, validator_power) in evidences_data.into_iter() {
            statement.push_str(&format!("\\\\x{},{},{},\\\\x{},{},{}\n",
                hex::encode(block_id),
                height.map_or("".to_string(), |h| h.to_string()),
                time.map_or("".to_string(), |t| t.to_string()),
                hex::encode(address.map_or(vec![], |a| a)),
                total_voting_power,
                validator_power,    
            ));
        };

        println!("Trying to create copy_in");
        let mut copy_in_evidences = sqlx_tx.copy_in_raw(&format!("COPY {}.evidences FROM stdin (DELIMITER ',', NULL '')", network)).await?;
        println!("{}", &statement);
        copy_in_evidences.send(statement.as_bytes()).await?;
        let row_count = copy_in_evidences.finish().await?;

        let dur = instant.elapsed();

        if row_count as usize != num_evidences {
            return Err(Error::FailCopyIn);
        }

        let labels = [
            ("bulk_insert", "evidences".to_string()),
            ("num_evidences", num_evidences.to_string()),
        ];

        histogram!(DB_SAVE_EVDS_DURATION, dur.as_secs_f64() * 1000.0, &labels);

        Ok(())
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
        let mut tx_values = String::new();
        let mut tx_transfer = String::new();
        let mut tx_bond = String::new();
        let mut tx_bridge_pool = String::new();

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

                        // let mut query_builder: QueryBuilder<_> = QueryBuilder::new(format!(
                        //     "INSERT INTO {}.tx_transfer(
                        //         tx_id,
                        //         source, 
                        //         target, 
                        //         token,
                        //         amount,
                        //         key,
                        //         shielded
                        //     )",
                        //     network
                        // ));

                        // let query = query_builder
                        //     .push_values(std::iter::once(0), |mut b, _| {
                        //         b.push_bind(tx.header_hash().0.as_slice().to_vec())
                        //             .push_bind(transfer.source.to_string())
                        //             .push_bind(transfer.target.to_string())
                        //             .push_bind(transfer.token.to_string())
                        //             .push_bind(transfer.amount.to_string())
                        //             .push_bind(transfer.key.as_ref().map(|k| k.to_string()))
                        //             .push_bind(transfer.shielded.as_ref().map(|s| s.to_vec()));
                        //     })
                        //     .build();
                        // query.execute(&mut *sqlx_tx).await?;

                        tx_transfer.push_str(&format!("\\\\x{},{},{},{},{},{},\\\\x{}\n",
                            hex::encode(tx.header_hash().0.as_slice()),
                            transfer.source.to_string(),
                            transfer.target.to_string(),
                            transfer.token.to_string(),
                            transfer.amount.to_string(),
                            transfer.key.as_ref().map_or("".to_string(), |k| k.to_string()),
                            hex::encode(transfer.shielded.as_ref().map_or(vec![],|s| s.to_vec())),
                        ))
                    }
                    "tx_bond" => {
                        info!("Saving tx_bond");
                        let data = tx.data().ok_or(Error::InvalidTxData)?;
                        let bond = transaction::pos::Bond::try_from_slice(&data[..])?;

                        // let mut query_builder: QueryBuilder<_> = QueryBuilder::new(format!(
                        //     "INSERT INTO {}.tx_bond(
                        //         tx_id,
                        //         validator,
                        //         amount,
                        //         source,
                        //         bond
                        //     )",
                        //     network
                        // ));

                        // let query = query_builder
                        //     .push_values(std::iter::once(0), |mut b, _| {
                        //         b.push_bind(tx.header_hash().0.as_slice().to_vec())
                        //             .push_bind(bond.validator.to_string())
                        //             .push_bind(bond.amount.to_string_native())
                        //             .push_bind(bond.source.as_ref().map(|s| s.to_string()))
                        //             .push_bind(true);
                        //     })
                        //     .build();
                        // query.execute(&mut *sqlx_tx).await?;

                        tx_bond.push_str(&format!("\\\\x{},{},{},{},TRUE\n",
                            hex::encode(tx.header_hash().0.as_slice()),
                            bond.validator.to_string(),
                            bond.amount.to_string_native(),
                            bond.source.as_ref().map_or("".to_string(), |s| s.to_string())
                        ))
                    }
                    "tx_unbond" => {
                        info!("Saving tx_unbond");
                        let data = tx.data().ok_or(Error::InvalidTxData)?;
                        let unbond = transaction::pos::Unbond::try_from_slice(&data[..])?;

                        // let mut query_builder: QueryBuilder<_> = QueryBuilder::new(format!(
                        //     "INSERT INTO {}.tx_bond(
                        //         tx_id,
                        //         validator,
                        //         amount,
                        //         source,
                        //         bond
                        //     )",
                        //     network
                        // ));

                        // let query = query_builder
                        //     .push_values(std::iter::once(0), |mut b, _| {
                        //         b.push_bind(tx.header_hash().0.as_slice().to_vec())
                        //             .push_bind(unbond.validator.to_string())
                        //             .push_bind(unbond.amount.to_string_native())
                        //             .push_bind(
                        //                 unbond
                        //                     .source
                        //                     .as_ref()
                        //                     .map_or("".to_string(), |s| s.to_string()),
                        //             )
                        //             .push_bind(false);
                        //     })
                        //     .build();
                        // query.execute(&mut *sqlx_tx).await?;

                        tx_bond.push_str(&format!("\\\\x{},{},{},{},False\n",
                            hex::encode(tx.header_hash().0.as_slice()),
                            unbond.validator.to_string(),
                            unbond.amount.to_string_native(),
                            unbond.source.as_ref().map_or("".to_string(), |s| s.to_string())
                        ));
                    }
                    // this is an ethereum transaction
                    "tx_bridge_pool" => {
                        info!("Saving tx_bridge_pool");
                        let data = tx.data().ok_or(Error::InvalidTxData)?;
                        // Only TransferToEthereum type is supported at the moment by namada and us.
                        let tx_bridge = PendingTransfer::try_from_slice(&data[..])?;

                        // let mut query_builder: QueryBuilder<_> = QueryBuilder::new(format!(
                        //     "INSERT INTO {}.tx_bridge_pool(
                        //         tx_id,
                        //         asset,
                        //         recipient,
                        //         sender,
                        //         amount,
                        //         gas_amount,
                        //         payer
                        //     )",
                        //     network
                        // ));

                        // let query = query_builder
                        //     .push_values(std::iter::once(0), |mut b, _| {
                        //         b.push_bind(tx.header_hash().0.as_slice().to_vec())
                        //             .push_bind(tx_bridge.transfer.asset.to_string())
                        //             .push_bind(tx_bridge.transfer.recipient.to_string())
                        //             .push_bind(tx_bridge.transfer.sender.to_string())
                        //             .push_bind(tx_bridge.transfer.amount.to_string_native())
                        //             .push_bind(tx_bridge.gas_fee.amount.to_string_native())
                        //             .push_bind(tx_bridge.gas_fee.payer.to_string())
                        //             .push_bind(false);
                        //     })
                        //     .build();
                        // query.execute(&mut *sqlx_tx).await?;

                        tx_bridge_pool.push_str(&format!("\\\\x{},{},{},{},{},{},{}\n",
                            hex::encode(tx.header_hash().0.as_slice()),
                            tx_bridge.transfer.asset.to_string(),
                            tx_bridge.transfer.recipient.to_string(),
                            tx_bridge.transfer.sender.to_string(),
                            tx_bridge.transfer.amount.to_string_native(),
                            tx_bridge.gas_fee.amount.to_string_native(),
                            tx_bridge.gas_fee.payer.to_string(),
                        ))
                    }
                    _ => {}
                }
            }

            tx_values.push_str(&format!("\\\\x{},\\\\x{},{},\\\\x{},\\\\x{}\n",
                hex::encode(tx.header_hash().0.as_slice()),
                hex::encode(block_id.to_vec()),
                utils::tx_type_name(&tx.header.tx_type),
                hex::encode(code),
                hex::encode(tx.data().map_or(vec![], |v| v.to_vec())),
            ));
        }

        let num_transactions = txs.len();
        let dur = instant.elapsed();

        let mut copy_in_transactions = sqlx_tx.copy_in_raw(&format!("COPY {}.transactions FROM stdin (DELIMITER ',', NULL '')", network)).await?;
        copy_in_transactions.send(tx_values.as_bytes()).await?;
        let count_row = copy_in_transactions.finish().await?;

        if count_row as usize != num_transactions {
            println!("{} {}", count_row, num_transactions);
            return Err(Error::FailCopyIn);
        }

        let mut copy_in_tx_transfer = sqlx_tx.copy_in_raw(&format!("COPY {}.tx_transfer FROM stdin (DELIMITER ',', NULL '')", network)).await?;
        copy_in_tx_transfer.send(tx_transfer.as_bytes()).await?;
        let _ = copy_in_tx_transfer.finish().await?;

        let mut copy_in_tx_bond = sqlx_tx.copy_in_raw(&format!("COPY {}.tx_bond FROM stdin (DELIMITER ',', NULL '')", network)).await?;
        copy_in_tx_bond.send(tx_bond.as_bytes()).await?;
        let _ = copy_in_tx_bond.finish().await?;

        let mut copy_in_tx_bridge_pool = sqlx_tx.copy_in_raw(&format!("COPY {}.tx_transfer FROM stdin (DELIMITER ',', NULL '')", network)).await?;
        copy_in_tx_bridge_pool.send(tx_bridge_pool.as_bytes()).await?;
        let _ = copy_in_tx_bridge_pool.finish().await?;

        let labels = [
            ("bulk_insert", "transactions".to_string()),
            ("num_transactions", num_transactions.to_string()),
        ];

        histogram!(DB_SAVE_TXS_DURATION, dur.as_secs_f64() * 1000.0, &labels);

        Ok(())
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
            "SELECT indexname, indexdef FROM pg_indexes WHERE tablename = '{}.{BLOCKS_TABLE_NAME}';",
            self.network
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
