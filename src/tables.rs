pub fn get_create_block_table_query(network: &str) -> String {
    format!(
        "CREATE TABLE IF NOT EXISTS {}.blocks (
        block_id BYTEA NOT NULL,
        header_version_app INTEGER NOT NULL,
        header_version_block INTEGER NOT NULL,
        header_chain_id TEXT NOT NULL,
        header_height INTEGER NOT NULL,
        header_time TEXT NOT NULL,
        header_last_block_id_hash BYTEA,
        header_last_block_id_parts_header_total INTEGER,
        header_last_block_id_parts_header_hash BYTEA,
        header_last_commit_hash BYTEA,
        header_data_hash BYTEA,
        header_validators_hash BYTEA NOT NULL,
        header_next_validators_hash BYTEA NOT NULL,
        header_consensus_hash BYTEA NOT NULL,
        header_app_hash TEXT NOT NULL,
        header_last_results_hash BYTEA,
        header_evidence_hash BYTEA,
        header_proposer_address TEXT NOT NULL,
        commit_height INTEGER,
        commit_round INTEGER,
        commit_block_id_hash BYTEA,
        commit_block_id_parts_header_total INTEGER,
        commit_block_id_parts_header_hash BYTEA
    );",
        network
    )
}

pub fn get_create_transactions_table_query(network: &str) -> String {
    format!(
        "CREATE TABLE IF NOT EXISTS {}.transactions (
        hash BYTEA NOT NULL,
        block_id BYTEA NOT NULL,
        tx_type TEXT NOT NULL,
        wrapper_id BYTEA,
        fee_amount_per_gas_unit TEXT,
        fee_token TEXT,
        gas_limit_multiplier BIGINT,
        code BYTEA,
        data JSON,
        return_code INTEGER
    );",
        network
    )
}

pub fn get_create_evidences_table_query(network: &str) -> String {
    format!(
        "CREATE TABLE IF NOT EXISTS {}.evidences (
        block_id BYTEA NOT NULL,
        height INTEGER,
        time TEXT,
        address BYTEA,
        total_voting_power TEXT NOT NULL,
        validator_power TEXT NOT NULL
    );",
        network
    )
}

pub fn get_create_commit_signatures_table_query(network: &str) -> String {
    format!(
        "CREATE TABLE IF NOT EXISTS {}.commit_signatures (
        block_id BYTEA NOT NULL,
        block_id_flag INTEGER NOT NULL,
        validator_address BYTEA NOT NULL,
        timestamp TEXT,
        signature BYTEA NOT NULL
    );",
        network
    )
}
