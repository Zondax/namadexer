pub fn get_create_tx_become_validator_view_query(network: &str) -> String {
    format!("CREATE OR REPLACE VIEW {network}.tx_become_validator AS 
    SELECT 
    data->>'address' AS address,
    data->>'consensus_key' AS consensus_key, 
    data->>'eth_cold_key' AS eth_cold_key, 
    data->>'eth_hot_key' AS eth_hot_key, 
    data->>'protocol_key' AS protocol_key, 
    data->>'commission_rate' AS commission_rate, 
    data->>'max_commission_rate_change' AS max_commission_rate_change, 
    data->>'email' AS email, 
    data->>'description' AS description, 
    data->>'website' AS website, 
    data->>'discord_handle' AS discord_handle, 
    data->>'avatar' AS avatar
    FROM {network}.transactions WHERE code = '\\x5938c2b9962eb57bf309a604afc9aa6bc55851fd1791346c5c5795abaaa5f295';")
}

pub fn get_create_tx_bond_view_query(network: &str) -> String {
    format!("CREATE OR REPLACE VIEW {network}.tx_bond AS
    SELECT
    data->>'validator' AS validator,
    data->>'amount' AS amount,
    data->>'source' AS source
    FROM {network}.transactions WHERE code = '\\x40e55cdef50e0771eb2b3cfe78b841d988345113f7fbeaa8b158de04589bb9fc';")
}

pub fn get_create_tx_bridge_pool_view_query(network: &str) -> String {
    format!("CREATE OR REPLACE VIEW {network}.tx_bridge_pool AS
    SELECT
    data
    FROM {network}.transactions WHERE code = '\\x66412b1b01b6659dc196bef86bdb540181d90c2f984a13597f0aa2a4f9c9c907';")
}

pub fn get_create_tx_change_consensus_key_view_query(network: &str) -> String {
    format!("CREATE OR REPLACE VIEW {network}.tx_change_consensus_key AS
    SELECT
    data->>'validator' AS validator,
    data->>'consensus_key' AS consensus_key
    FROM {network}.transactions WHERE code = '\\x919be890123e9fbe7f85640811f3ca8edfebe94b7314118afbc84ff9c75ec488';")
}

pub fn get_create_tx_change_validator_comission_view_query(network: &str) -> String {
    format!("CREATE OR REPLACE VIEW {network}.tx_change_validator_comission AS
    SELECT
    data->>'validator' AS validator,
    data->>'new_rate' AS new_rate
    FROM {network}.transactions WHERE code = '\\xf76e025e52c75e34937e76fae43c6ed7544e2268e0bcdafeca0b05f9b7484b36';")
}

pub fn get_create_tx_change_validator_metadata_view_query(network: &str) -> String {
    format!("CREATE OR REPLACE VIEW {network}.tx_change_validator_metadata AS
    SELECT
    data->>'validator' AS validator,
    data->>'email' AS email,
    data->>'description' AS description,
    data->>'website' AS website,
    data->>'discord_handle' AS discord_handle,
    data->>'avatar' AS avatar,
    data->>'commission_rate' AS commission_rate
    FROM {network}.transactions WHERE code = '\\x1f0981a2ff60b5e9619c5464839d8d4e08dac72a8185a21285ae7f0e9498dd8c';")
}

pub fn get_create_tx_claim_rewards_view_query(network: &str) -> String {
    format!("CREATE OR REPLACE VIEW {network}.tx_claim_rewards AS
    SELECT
    data->>'validator' AS validator,
    data->>'source' AS source
    FROM {network}.transactions WHERE code = '\\x4af7ca07f6e6f2ad87ffe2c5fca90224544c45cc263e2e4d05775d782cac1f48';")
}

pub fn get_create_tx_deactivate_validator_view_query(network: &str) -> String {
    format!("CREATE OR REPLACE VIEW {network}.tx_deactivate_validator AS
    SELECT
    data AS address
    FROM {network}.transactions WHERE code = '\\x0faaf9b55c150cdf8b2ea6a05c5fae725735b4fee44aa5da79bcd1881cb43f78';")
}

pub fn get_create_tx_ibc_view_query(network: &str) -> String {
    format!("CREATE OR REPLACE VIEW {network}.tx_ibc AS
    SELECT
    data
    FROM {network}.transactions WHERE code = '\\xf99df82e284dcb96a12b409bc43aa7dc77b346ab0b2d3f0a9a39807e749ce8ee';")
}

pub fn get_create_tx_init_account_view_query(network: &str) -> String {
    format!("CREATE OR REPLACE VIEW {network}.tx_init_account AS
    SELECT
    data->>'public_keys' AS public_keys,
    data->>'vp_code_hash' AS vp_code_hash,
    data->>'threshold' AS threshold
    FROM {network}.transactions WHERE code = '\\x51e79a8f5d39b40db8531610398a8631e27390d48af80c916382779d7b0e7e41';")
}

pub fn get_create_tx_init_proposal_view_query(network: &str) -> String {
    format!("CREATE OR REPLACE VIEW {network}.tx_init_proposal AS
    SELECT
    data->>'id' AS id,
    data->>'content' AS content,
    data->>'author' AS author,
    data->>'r#type' AS rtype,
    data->>'voting_start_epoch' AS voting_start_epoch,
    data->>'voting_end_epoch' AS voting_end_epoch,
    data->>'grace_epoch' AS grace_epoch
    FROM {network}.transactions WHERE code = '\\xb0a4e44eb0e8e3a1af49ebd2b0483260c67a17f974c7517904e88897279d1b29';")
}

pub fn get_create_tx_reactivate_validator_view_query(network: &str) -> String {
    format!("CREATE OR REPLACE VIEW {network}.tx_reactivate_validator AS
    SELECT
    data AS address
    FROM {network}.transactions WHERE code = '\\xc94c4e6d549c921b9a483675f0e3af45eef79d4489bd35061fd285af6189b20d';")
}

pub fn get_create_tx_redelegate_view_query(network: &str) -> String {
    format!("CREATE OR REPLACE VIEW {network}.tx_redelegate AS
    SELECT
    data->>'redel_bond_start' AS redel_bond_start,
    data->>'src_validator' AS src_validator,
    data->>'bond_start' AS bond_start,
    data->>'amount' AS amount
    FROM {network}.transactions WHERE code = '\\x4afaa8c4ea6138a43d465ed09e86aedb16c54f63fd09a752a3aca5a26542e126';")
}

pub fn get_create_tx_resign_steward_view_query(network: &str) -> String {
    format!("CREATE OR REPLACE VIEW {network}.tx_resign_steward AS
    SELECT
    data AS address
    FROM {network}.transactions WHERE code = '\\x7655ed64d1b07900672aee307f679b35af77337c077648adc11914348d1f130f';")
}

pub fn get_create_tx_reveal_pk_view_query(network: &str) -> String {
    format!("CREATE OR REPLACE VIEW {network}.tx_reveal_pk AS
    SELECT
    data AS public_key
    FROM {network}.transactions WHERE code = '\\x283fd236d971dd0f7ca1a329b508a4039946f40f1c9792863fe6b0fa05d74832';")
}

pub fn get_create_tx_transfert_view_query(network: &str) -> String {
    format!("CREATE OR REPLACE VIEW {network}.tx_transfert AS
    SELECT
    data->>'source' AS source,
    data->>'target' AS target,
    data->>'token' AS token,
    data->>'amount' AS amount
    FROM {network}.transactions WHERE code = '\\x0960374d23acbac1feb27b3888095859217936c900cef54e559d215cec3206ef';")
}

pub fn get_create_tx_unbond_view_query(network: &str) -> String {
    format!("CREATE OR REPLACE VIEW {network}.tx_unbond AS
    SELECT
    data->>'validator' AS validator,
    data->>'amount' AS amount,
    data->>'source' AS source
    FROM {network}.transactions WHERE code = '\\xe39415d64bdc3c16f9b21ebee4e9496f8ce5cdd5551d1e611c449d8bfdcffae0';")
}

pub fn get_create_tx_unjail_validator_view_query(network: &str) -> String {
    format!("CREATE OR REPLACE VIEW {network}.tx_unjail_validator AS
    SELECT
    data AS address
    FROM {network}.transactions WHERE code = '\\x2b1451721dcdd069a19cba1f9b338bb6a45d85d0d56ba7ca952742d3ec5878b3';")
}

pub fn get_create_tx_update_account_view_query(network: &str) -> String {
    format!("CREATE OR REPLACE VIEW {network}.tx_update_account AS
    SELECT
    data->>'addr' AS addr,
    data->>'vp_code_hash' AS vp_code_hash,
    data->>'public_keys' AS public_keys,
    data->>'threshold' AS threshold
    FROM {network}.transactions WHERE code = '\\x70f91d4f778d05d40c5a56490ced906b016e4b7a2a2ef5ff0ac0541ff28c5a22';")
}

pub fn get_create_tx_update_steward_commission_view_query(network: &str) -> String {
    format!("CREATE OR REPLACE VIEW {network}.tx_update_steward_commission AS
    SELECT
    data->>'steward' AS steward,
    data->>'commission' AS commission 
    FROM {network}.transactions WHERE code = '\\xed0ccfa4a8c8fa86f9f14c3b53b7f126ce4a86849815dad784b5aa35011a1db6';")
}

pub fn get_create_tx_vote_proposal_view_query(network: &str) -> String {
    format!("CREATE OR REPLACE VIEW {network}.tx_vote_proposal AS
    SELECT
    data->>'id' AS id,
    data->>'vote' AS vote,
    data->>'voter' AS voter,
    data->>'delegations' AS delegations
    FROM {network}.transactions WHERE code = '\\xccdbe81f664ca6c2caa11426927093dc10ed95e75b3f2f45bffd8514fee47cd0';")
}

pub fn get_create_tx_withdraw_view_query(network: &str) -> String {
    format!("CREATE OR REPLACE VIEW {network}.tx_withdraw AS
    SELECT
    data->'validator' AS validator,
    data->'source' AS source
    FROM {network}.transactions WHERE code = '\\x69560777e2656b2872a49080c51bb5b1a498a7ffd79dae491d36b301a1b012e6';")
}
