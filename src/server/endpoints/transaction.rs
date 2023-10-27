use axum::{
    extract::{Path, State},
    Json,
};
use tracing::info;

use crate::{
    server::{shielded, ServerState, TxInfo},
    Error,
};

pub async fn get_tx_by_hash(
    State(state): State<ServerState>,
    Path(hash): Path<String>,
) -> Result<Json<Option<TxInfo>>, Error> {
    info!("calling /tx/:tx_hash");

    let hash = hex::decode(hash)?;

    let row = state.db.get_tx(&hash).await?;
    let Some(row) = row else {
        return Ok(Json(None));
    };
    let mut tx = TxInfo::try_from(row)?;

    // ignore the error for now
    _ = tx.decode_tx(&state.checksums_map);

    Ok(Json(Some(tx)))
}

// Return a list of the shielded assets and their total compiled using all the shielded transactions (in, internal and out)
pub async fn get_shielded_tx(
    State(state): State<ServerState>,
) -> Result<Json<shielded::ShieldedAssetsResponse>, Error> {
    let rows = state.db.get_shielded_tx().await?;

    let shielded_assests_response = shielded::ShieldedAssetsResponse::try_from(&rows)?;

    Ok(Json(shielded_assests_response))
}
