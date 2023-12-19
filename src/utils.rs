use namada_sdk::core::types::transaction::TxType;
use std::collections::HashMap;
use std::fs;

pub fn tx_type_name(tx_type: &TxType) -> String {
    match tx_type {
        TxType::Raw => "Raw".to_string(),
        TxType::Wrapper(_) => "Wrapper".to_string(),
        TxType::Decrypted(_) => "Decrypted".to_string(),
        TxType::Protocol(_) => "Protocol".to_string(),
    }
}

pub fn load_checksums() -> Result<HashMap<String, String>, crate::Error> {
    let checksums = fs::read_to_string("./checksums.json")?;
    let json: serde_json::Value = serde_json::from_str(&checksums)?;
    let obj = json.as_object().ok_or(crate::Error::InvalidChecksum)?;

    let mut checksums_map = HashMap::new();
    for value in obj.iter() {
        let hash = value
            .1
            .as_str()
            .ok_or(crate::Error::InvalidChecksum)?
            .split('.')
            .collect::<Vec<&str>>()[1];
        let type_tx = value.0.split('.').collect::<Vec<&str>>()[0];

        checksums_map.insert(hash.to_string(), type_tx.to_string());
    }

    Ok(checksums_map)
}
