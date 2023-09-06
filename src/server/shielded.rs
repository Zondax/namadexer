use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::MASP_ADDR;
use crate::error::Error;
use sqlx::postgres::PgRow as Row;
use sqlx::Row as TRow;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ShieldedAssetsResponse {
    pub shielded_assets: ShieldedAssets,
}

impl TryFrom<&Vec<Row>> for ShieldedAssetsResponse {
    type Error = Error;

    fn try_from(rows: &Vec<Row>) -> Result<Self, Self::Error> {

        let mut shielded_assets: ShieldedAssets = ShieldedAssets::new();

        for row in rows {
            let token: String = row.try_get("token")?;
            let amount: u64 = u64::from_str_radix(row.try_get("amount")?, 10)?;

            let target: String = row.try_get("target")?;
            let source: String = row.try_get("source")?;

            if target == MASP_ADDR || source == MASP_ADDR {
                if target == source {
                    continue
                }
    
                let mut total = match shielded_assets.get(&token) {
                    Some(v) => *v,
                    None => 0,
                };

                if target == MASP_ADDR {
                    total += amount;
                } else {
                    total -= amount;
                }

                shielded_assets.insert(token, total);
            };
        }

        Ok(Self {
            shielded_assets,
        })
    }
}


pub type ShieldedAssets = HashMap<String, u64>;