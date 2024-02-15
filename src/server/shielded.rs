use crate::error::Error;
use crate::MASP_ADDR;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgRow as Row;
use sqlx::Row as TRow;
use std::collections::HashMap;

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

            let amount_str: &str = row.try_get("amount")?;
            let amount: f64 = amount_str.parse::<f64>()?;

            let target: String = row.try_get("target")?;
            let source: String = row.try_get("source")?;

            if target == MASP_ADDR || source == MASP_ADDR {
                if target == source {
                    continue;
                }

                let mut total = match shielded_assets.get(&token) {
                    Some(v) => *v,
                    None => 0.0,
                };

                if target == MASP_ADDR {
                    total += amount;
                } else {
                    total -= amount;
                }

                shielded_assets.insert(token, total);
            };
        }

        Ok(Self { shielded_assets })
    }
}

pub type ShieldedAssets = HashMap<String, f64>;
