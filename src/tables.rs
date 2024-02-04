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
        code_type TEXT,
        code BYTEA,
        data BYTEA,
        memo BYTEA,
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
// To store account_updates transactions
// the update_id is used primarly for getting all the public keys for account_id
// in a sort of batches, where each batch was a new set of pub_keys for which and account
// was updated in a account_update transaction.
pub fn get_create_account_updates_table(network: &str) -> String {
    // NOTE: We are creating the index here as well so it
    // is used as  reference by the account_public_keys table.
    // Otherwise postgres complains when creating that table
    // due to the missing primary index in account_updates.
    // Importan to mention that update_id is use to link public keys
    // to an update in time.
    format!(
        "CREATE TABLE IF NOT EXISTS {}.account_updates (
        update_id SERIAL PRIMARY KEY,
        account_id TEXT NOT NULL,
        vp_code_hash BYTEA,
        threshold INTEGER,
        tx_id BYTEA NOT NULL UNIQUE
    );",
        network,
    )
}

// To be use by the account_init and account_update transactions
// any account can have many pub_keys
pub fn get_create_account_public_keys_table(network: &str) -> String {
    // We remove the UNIQUE constrain as it will allow us to have many
    // rows(public_keys) pointing to the same update_id, which is correct
    // as many keys are associated to the same account.
    format!(
        "CREATE TABLE IF NOT EXISTS {}.account_public_keys (
        id SERIAL,
        update_id INTEGER REFERENCES {}.account_updates(update_id),
        public_key TEXT NOT NULL
    );",
        network, network
    )
}

pub fn get_create_vote_proposal_table(network: &str) -> String {
    // NOTE: the id is converted to be_bytes because it is
    // defined as a u64, and postgres only supports u32?
    // the vote is a boolean, Yay means true, otherwise vote
    // is a Nay.
    // regarding vote_type, here is its definition:
    //
    // pub enum VoteType {
    //     /// A default vote without Memo
    //     Default,
    //     /// A vote for the PGF stewards
    //     PGFSteward,
    //     /// A vote for a PGF payment proposal
    //     PGFPayment,
    // }
    // so for now we made the vote_type nullable, as it
    // is valid only for vote = true cases. and we will use
    // the integer representation for each field:
    // Default = 0
    // PGFSteward = 1
    // PGFPayment = 2
    // finally the delegations field is a vector of address,
    // it would be store in a different table, as this is a one
    // to many relationship
    //
    // Finally we create the PRIMARY KEY index here
    // to avoid issues, the reason is, because it is use as
    // a reference for the delegations table.
    format!(
        "CREATE TABLE IF NOT EXISTS {}.vote_proposal (
        vote_proposal_id BYTEA,
        vote TEXT NOT NULL,
        voter TEXT NOT NULL,
        tx_id BYTEA NOT NULL
    );",
        network
    )
}

// store the delegations addresses associated to an specific vote_proposal_id
pub fn get_create_delegations_table(network: &str) -> String {
    // NOTE: The vote_proposal_id is in this case not defined as unique
    // because we do not know with certainty if delegator are prohibited
    // of being part of another vote_proposal over time.
    format!(
        "CREATE TABLE IF NOT EXISTS {}.delegations (
        id SERIAL,
        vote_proposal_id BYTEA,
        delegator_id TEXT NOT NULL
    );",
        network
    )
}
