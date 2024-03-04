use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::Row as TRow;
use std::collections::HashMap;
use tracing::info;
use serde_json::Value;

use crate::{
    server::{blocks::HashID, blocks::TxShort, ServerState},
    BlockInfo, Error,
};

pub async fn get_txs_by_address(
    State(state): State<ServerState>,
    Path(address): Path<String>,
) -> Result<Json<Option<Value>>, Error> {
    info!("calling /address/:{}", address);

    let rows = state.db.get_txs_by_address(&address).await?;
    

    Ok(Json(None))
}