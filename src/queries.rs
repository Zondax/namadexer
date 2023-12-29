pub(crate) fn insert_block_query(network: &str) -> String {
    format!(
        r#"INSERT INTO {}.blocks (
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
    ) VALUES (
        $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, 
        $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, 
        $21, $22, $23
    )"#,
        network
    )
}
