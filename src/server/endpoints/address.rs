use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row as TRow;
use std::collections::HashMap;
use tracing::info;

use crate::{
    server::{blocks::HashID, blocks::TxShort, ServerState, TxInfo},
    BlockInfo, Error,
};

pub async fn get_txs_by_address(
    State(state): State<ServerState>,
    Path(address): Path<String>,
) -> Result<Json<Option<Vec<TxInfo>>>, Error> {
    info!("calling /address/:{}", address);

    let rows = state.db.get_txs_by_address(&address).await?;

    if rows.is_empty() {
        return Ok(Json(None));
    }

    let mut response: Vec<TxInfo> = vec![];
    for row in rows {
        let tx = TxInfo::try_from(row)?;
        response.push(tx);
    }

    Ok(Json(Some(response)))
}
