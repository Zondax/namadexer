use axum::{
    extract::{Path, State},
    Json,
};
use tracing::info;

use crate::{
    server::{shielded, tx::VoteProposalTx, ServerState, TxInfo},
    Error,
};

use sqlx::Row as TRow;

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

pub async fn get_vote_proposal(
    State(state): State<ServerState>,
    Path(proposal_id): Path<u64>,
) -> Result<Json<Option<VoteProposalTx>>, Error> {
    let vote_proposal_data = state.db.vote_proposal_data(proposal_id).await?;

    let Some(vote_proposal_data) = vote_proposal_data else {
        return Ok(Json(None));
    };

    let mut tx = VoteProposalTx::try_from(vote_proposal_data)?;

    let delegations = state.db.vote_proposal_delegations(proposal_id).await?;
    // TODO: is it ok to have vote_proposals with empty delegator list?

    let delegations: Vec<String> = delegations
        .into_iter()
        .filter_map(|row| {
            row.try_get::<Option<String>, _>("delegator_id")
                .ok()
                .flatten()
        })
        .collect::<Vec<String>>();

    tx.delegations = delegations;

    Ok(Json(Some(tx)))
}
