use serde::{Deserialize, Serialize};
use axum::{
    extract::{Path, State, Query},
    Json,
};
use sqlx::postgres::PgRow as Row;
use sqlx::Row as TRow;
use tracing::{info, instrument};
use std::collections::HashMap;

use crate::{server::ServerState, Error};

// Retrieve the count of commit for a range of blocks from the sql query result.
#[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
#[repr(transparent)]
struct CommitCount(pub i64);

impl TryFrom<&Row> for CommitCount {
    type Error = Error;

    #[instrument(level = "trace", skip(row))]
    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        let count: i64 = row.try_get("count")?;

        Ok(CommitCount(count))
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct UptimeValue{
    pub uptime: f64,
}

pub async fn get_validator_uptime(
    State(state): State<ServerState>,
    Path(validator_address): Path<String>,
    Query(params): Query<HashMap<String, i32>>,
) -> Result<Json<UptimeValue>, Error> {
    info!("calling /validator/:validator_address/uptime");
    
    let start = params.get("start");
    let end = params.get("end");

    let va = hex::decode(validator_address)?;
    let row = state.db.validator_uptime(&va, start, end).await?;
    let cc = CommitCount::try_from(&row)?;

    // default range is 500 blocks
    let mut ranger_size: f64 = 500.0;

    if let (Some(s), Some(e)) = (start, end) {
        ranger_size = (e - s).into();
    }

    let uv = UptimeValue{ uptime: (cc.0 as f64)/ranger_size };


    Ok(Json(uv))
}
