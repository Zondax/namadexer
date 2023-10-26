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
        fee_amount_per_gas_unit TEXT,
        fee_token TEXT,
        gas_limit_multiplier BIGINT,
        code BYTEA,
        data BYTEA
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

pub fn get_create_tx_transfer_table_query(network: &str) -> String {
    format!(
        "CREATE TABLE IF NOT EXISTS {}.tx_transfer (
        tx_id BYTEA NOT NULL,
        source TEXT NOT NULL,
        target TEXT NOT NULL,
        token TEXT NOT NULL,
        amount TEXT NOT NULL,
        key TEXT,
        shielded BYTEA
    );",
        network
    )
}

pub fn get_create_tx_bond_table_query(network: &str) -> String {
    format!(
        "CREATE TABLE IF NOT EXISTS {}.tx_bond (
        tx_id BYTEA NOT NULL,
        validator TEXT NOT NULL,
        amount TEXT NOT NULL,
        source TEXT,
        bond BOOL NOT NULL
    );",
        network
    )
}

pub fn get_create_tx_bridge_pool_table_query(network: &str) -> String {
    format!(
        "CREATE TABLE IF NOT EXISTS {}.tx_bridge_pool (
        tx_id BYTEA NOT NULL,
        asset TEXT NOT NULL,
        recipient TEXT NOT NULL,
        sender TEXT NOT NULL,
        amount TEXT NOT NULL,
        gas_amount TEXT NOT NULL,
        payer TEXT NOT NULL
    );",
        network
    )
}
