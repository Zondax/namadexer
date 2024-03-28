# Indexer

The indexer is written in Rust and store blocks and transaction to a PostgreSQL database. It requires to have access to a Namada node through its RPC enpoints.

## Configuration

The indexer must have the following configuration:

Settings.toml
```toml
# Level and format of logging in the indexer
log_level = "info"
log_format = "pretty" # optional; either "pretty" or "json"
chain_name = "public-testnet-15" # IMPORANT! Do not use `.` just put the name of the network and don't have the hash (e.g 'shielded-expedition.b40d8e9055' becomes 'shielded-expedition')

# Connection information for the PostgreSQL database
[database]
host = "localhost"
user = "postgres"
password = "wow"
dbname = "blockchain"
create_index = true

# The tendermint RPC address and port to access the Namada node
[indexer]
tendermint_addr = "http://127.0.0.1:26657"
```

In option it is possible to activate the `prometheus` feature or `jeager` for a better view of the indexer performances. See [telemetry](./telemetry.md)

## Starting the indexer

You will need first to download the `checksums.json` file from Namada. This is required for the indexer to work.
```
$ make download-checksum
```

In order to start the indexer we will need to pass the configuration file path to the environement variable `INDEXER_CONFIG_PATH`.

```
$ INDEXER_CONFIG_PATH="${PWD}/config/Settings.toml" ./indexer
```

## Postgres tables

The tables are automatically created by the indexer if they don't exist.
```sql
            List of relations
 Schema |     Name     | Type  |  Owner   
--------+--------------+-------+----------
 public | blocks       | table | postgres
 public | evidences    | table | postgres
 public | transactions | table | postgres
```

Once the indexer has done the initial syncing it will automatically create indexes to make retrieving data from the server faster.

In addition, we create views for all the different kind of transactions (see all te `tx_*` in `checksums.json`). The views facilitate querying specific data from decoded transactions.

### Blocks

This table contains all the information found in a block and also the commit information. The evidences link to a block can be found in another table `evidences`.

NOTES: replace `shielded_expedition` with whatever other network name you have configured.
```
\d shielded_expedition.blocks

                         Table "shielded_expedition.blocks"
                 Column                  |  Type   | Collation | Nullable | Default 
-----------------------------------------+---------+-----------+----------+---------
 block_id                                | bytea   |           | not null | 
 header_version_app                      | integer |           | not null | 
 header_version_block                    | integer |           | not null | 
 header_chain_id                         | text    |           | not null | 
 header_height                           | integer |           | not null | 
 header_time                             | text    |           | not null | 
 header_last_block_id_hash               | bytea   |           |          | 
 header_last_block_id_parts_header_total | integer |           |          | 
 header_last_block_id_parts_header_hash  | bytea   |           |          | 
 header_last_commit_hash                 | bytea   |           |          | 
 header_data_hash                        | bytea   |           |          | 
 header_validators_hash                  | bytea   |           | not null | 
 header_next_validators_hash             | bytea   |           | not null | 
 header_consensus_hash                   | bytea   |           | not null | 
 header_app_hash                         | text    |           | not null | 
 header_last_results_hash                | bytea   |           |          | 
 header_evidence_hash                    | bytea   |           |          | 
 header_proposer_address                 | text    |           | not null | 
 commit_height                           | integer |           |          | 
 commit_round                            | integer |           |          | 
 commit_block_id_hash                    | bytea   |           |          | 
 commit_block_id_parts_header_total      | integer |           |          | 
 commit_block_id_parts_header_hash       | bytea   |           |          | 
```

### Commit Signatures

```
\d shielded_expedition.commit_signatures

        Table "shielded_expedition.commit_signatures"
      Column       |  Type   | Collation | Nullable | Default 
-------------------+---------+-----------+----------+---------
 block_id          | bytea   |           | not null | 
 block_id_flag     | integer |           | not null | 
 validator_address | bytea   |           | not null | 
 timestamp         | text    |           |          | 
 signature         | bytea   |           | not null | 
```

### Evidences

The `evidences` table contains the evidences of validators misbehavior. Only one evidence is being used in Namada : Duplicate Vote Evidence.

```
\d shielded_expedition.evidences

             Table "shielded_expedition.evidences"
       Column       |  Type   | Collation | Nullable | Default 
--------------------+---------+-----------+----------+---------
 block_id           | bytea   |           | not null | 
 height             | integer |           |          | 
 time               | text    |           |          | 
 address            | bytea   |           |          | 
 total_voting_power | text    |           | not null | 
 validator_power    | text    |           | not null | 
```

### Transactions

The `transactions` table contains all the transactions that either encrypted or decrypted (defined by the `tx_type`). The decrypted data is then stored as a json object under `data`. The data is decoded in the indexer side before being stored.

NOTE: it doesn't seem to be worth storing the encrypted data as no computation can be done over it. If a specific use case is mentioned it can be added.

```
\d shielded_expedition.transactions

              Table "shielded_expedition.transactions"
         Column          |  Type   | Collation | Nullable | Default 
-------------------------+---------+-----------+----------+---------
 hash                    | bytea   |           | not null | 
 block_id                | bytea   |           | not null | 
 tx_type                 | text    |           | not null | 
 wrapper_id              | bytea   |           |          | 
 fee_amount_per_gas_unit | text    |           |          | 
 fee_token               | text    |           |          | 
 gas_limit_multiplier    | bigint  |           |          | 
 code                    | bytea   |           |          | 
 data                    | json    |           |          | 
 return_code             | integer |           |          | 

```

## Postgres views

All the views created.

Views might change after namada changes.


```sql
           View "shielded_expedition.tx_become_validator"
           Column           | Type | Collation | Nullable | Default 
----------------------------+------+-----------+----------+---------
 address                    | text |           |          | 
 consensus_key              | text |           |          | 
 eth_cold_key               | text |           |          | 
 eth_hot_key                | text |           |          | 
 protocol_key               | text |           |          | 
 commission_rate            | text |           |          | 
 max_commission_rate_change | text |           |          | 
 email                      | text |           |          | 
 description                | text |           |          | 
 website                    | text |           |          | 
 discord_handle             | text |           |          | 
 avatar                     | text |           |          | 

        View "shielded_expedition.tx_bond"
  Column   | Type | Collation | Nullable | Default 
-----------+------+-----------+----------+---------
 validator | text |           |          | 
 amount    | text |           |          | 
 source    | text |           |          | 

   View "shielded_expedition.tx_bridge_pool"
 Column | Type | Collation | Nullable | Default 
--------+------+-----------+----------+---------
 data   | json |           |          | 

  View "shielded_expedition.tx_change_consensus_key"
    Column     | Type | Collation | Nullable | Default 
---------------+------+-----------+----------+---------
 validator     | text |           |          | 
 consensus_key | text |           |          | 

View "shielded_expedition.tx_change_validator_comission"
  Column   | Type | Collation | Nullable | Default 
-----------+------+-----------+----------+---------
 validator | text |           |          | 
 new_rate  | text |           |          | 

 View "shielded_expedition.tx_change_validator_metadata"
     Column      | Type | Collation | Nullable | Default 
-----------------+------+-----------+----------+---------
 validator       | text |           |          | 
 email           | text |           |          | 
 description     | text |           |          | 
 website         | text |           |          | 
 discord_handle  | text |           |          | 
 avatar          | text |           |          | 
 commission_rate | text |           |          | 

    View "shielded_expedition.tx_claim_rewards"
  Column   | Type | Collation | Nullable | Default 
-----------+------+-----------+----------+---------
 validator | text |           |          | 
 source    | text |           |          | 

View "shielded_expedition.tx_deactivate_validator"
 Column  | Type | Collation | Nullable | Default 
---------+------+-----------+----------+---------
 address | json |           |          | 

       View "shielded_expedition.tx_ibc"
 Column | Type | Collation | Nullable | Default 
--------+------+-----------+----------+---------
 data   | json |           |          | 

      View "shielded_expedition.tx_init_account"
    Column    | Type | Collation | Nullable | Default 
--------------+------+-----------+----------+---------
 public_keys  | text |           |          | 
 vp_code_hash | text |           |          | 
 threshold    | text |           |          | 

        View "shielded_expedition.tx_init_proposal"
       Column       | Type | Collation | Nullable | Default 
--------------------+------+-----------+----------+---------
 id                 | text |           |          | 
 content            | text |           |          | 
 author             | text |           |          | 
 rtype              | text |           |          | 
 voting_start_epoch | text |           |          | 
 voting_end_epoch   | text |           |          | 
 grace_epoch        | text |           |          | 

View "shielded_expedition.tx_reactivate_validator"
 Column  | Type | Collation | Nullable | Default 
---------+------+-----------+----------+---------
 address | json |           |          | 

         View "shielded_expedition.tx_redelegate"
      Column      | Type | Collation | Nullable | Default 
------------------+------+-----------+----------+---------
 redel_bond_start | text |           |          | 
 src_validator    | text |           |          | 
 bond_start       | text |           |          | 
 amount           | text |           |          | 

  View "shielded_expedition.tx_resign_steward"
 Column  | Type | Collation | Nullable | Default 
---------+------+-----------+----------+---------
 address | json |           |          | 

      View "shielded_expedition.tx_reveal_pk"
   Column   | Type | Collation | Nullable | Default 
------------+------+-----------+----------+---------
 public_key | json |           |          | 

    View "shielded_expedition.tx_transfert"
 Column | Type | Collation | Nullable | Default 
--------+------+-----------+----------+---------
 source | text |           |          | 
 target | text |           |          | 
 token  | text |           |          | 
 amount | text |           |          | 

       View "shielded_expedition.tx_unbond"
  Column   | Type | Collation | Nullable | Default 
-----------+------+-----------+----------+---------
 validator | text |           |          | 
 amount    | text |           |          | 
 source    | text |           |          | 

 View "shielded_expedition.tx_unjail_validator"
 Column  | Type | Collation | Nullable | Default 
---------+------+-----------+----------+---------
 address | json |           |          | 

     View "shielded_expedition.tx_update_account"
    Column    | Type | Collation | Nullable | Default 
--------------+------+-----------+----------+---------
 addr         | text |           |          | 
 vp_code_hash | text |           |          | 
 public_keys  | text |           |          | 
 threshold    | text |           |          | 

View "shielded_expedition.tx_update_steward_commission"
   Column   | Type | Collation | Nullable | Default 
------------+------+-----------+----------+---------
 steward    | text |           |          | 
 commission | text |           |          | 

     View "shielded_expedition.tx_vote_proposal"
   Column    | Type | Collation | Nullable | Default 
-------------+------+-----------+----------+---------
 id          | text |           |          | 
 vote        | text |           |          | 
 voter       | text |           |          | 
 delegations | text |           |          | 

      View "shielded_expedition.tx_withdraw"
  Column   | Type | Collation | Nullable | Default 
-----------+------+-----------+----------+---------
 validator | json |           |          | 
 source    | json |           |          | 

```

## Indexer logic

![Indexer graph](./assets/indexer_graph.jpg)

