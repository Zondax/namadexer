use axum::{
    extract::{Path, State},
    Json,
};
use tracing::info;

use crate::{
    server::{shielded, ServerState, TxInfo},
    Error,
};

use sqlx::Row as TRow;

pub async fn get_tx_by_hash(
    State(state): State<ServerState>,
    Path(hash): Path<String>,
) -> Result<Json<Option<TxInfo>>, Error> {
    info!("calling /tx/:tx_hash{}", hash);

    let hash = hex::decode(hash)?;

    let row = state.db.get_tx(&hash).await?;
    let Some(row) = row else {
        return Ok(Json(None));
    };
    let tx = TxInfo::try_from(row)?;

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

pub async fn get_vote_proposal(
    State(state): State<ServerState>,
    Path(proposal_id): Path<i64>,
) -> Result<Json<serde_json::Value>, Error> {
    let mut votes: Vec<serde_json::Value> = vec![];
    let rows = state.db.vote_proposal_data(proposal_id).await?;
    for row in rows {
        let vote_proposal_data: serde_json::Value = row.try_get("data")?;

        votes.push(vote_proposal_data);
    }

    Ok(Json(serde_json::Value::Array(votes)))
}
