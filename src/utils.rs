use namada_sdk::tx::data::TxType;
use std::collections::HashMap;
use std::{env, fs};

const CHECKSUMS_FILE_PATH_ENV: &str = "CHECKSUMS_FILE_PATH";
const CHECKSUMS_REMOTE_URL_ENV: &str = "CHECKSUMS_REMOTE_URL";
const CHECKSUMS_PROCESSED_FILE_PATH_ENV: &str = "CHECKSUMS_PROCESSED_FILE_PATH";
const CHECKSUMS_DEFAULT_PATH: &str = "checksums.json";

pub fn tx_type_name(tx_type: &TxType) -> String {
    match tx_type {
        TxType::Raw => "Raw".to_string(),
        TxType::Wrapper(_) => "Wrapper".to_string(),
        TxType::Decrypted(_) => "Decrypted".to_string(),
        TxType::Protocol(_) => "Protocol".to_string(),
    }
}

pub fn load_checksums() -> Result<HashMap<String, String>, crate::Error> {
    let checksums_processed_file_path = env::var(CHECKSUMS_PROCESSED_FILE_PATH_ENV);
    let checksums = match checksums_processed_file_path {
        Ok(ref path) => fs::read_to_string(path)?,
        _ => "".to_string(),
    };

    if checksums.len() > 0 {
        let json: serde_json::Value = serde_json::from_str(&checksums)?;
        let obj = json.as_object().ok_or(crate::Error::InvalidChecksum)?;
        let mut checksums_map = HashMap::new();
        for (key, value) in obj.iter() {

            checksums_map.insert(key.to_string(), value.as_str().unwrap().to_string());
        }
        return Ok(checksums_map);
    }

    let checksums_file_path = env::var(CHECKSUMS_FILE_PATH_ENV);
    let checksums_remote_url = env::var(CHECKSUMS_REMOTE_URL_ENV);

    let checksums = match (checksums_file_path, checksums_remote_url) {
        (Ok(path), _) => fs::read_to_string(path)?,
        (_, Ok(url)) => ureq::get(&url)
            .call()
            .map_err(|e| crate::Error::Generic(Box::new(e)))?
            .into_string()?,
        _ => fs::read_to_string(CHECKSUMS_DEFAULT_PATH)?,
    };

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
