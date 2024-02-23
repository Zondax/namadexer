use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{
    server::{shielded, tx::VoteProposalTx, ServerState, TxDetails, TxInfo},
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
    let mut tx = TxInfo::try_from(row)?;

    // ignore the error for now
    _ = tx.decode_tx(&state.checksums_map);

    Ok(Json(Some(tx)))
}

#[derive(Deserialize)]
pub struct PaginationQuery {
    page: Option<u32>,
    limit: Option<u32>,
}

#[derive(Serialize, Deserialize)]
pub struct PaginatedResponse {
    data: Vec<TxDetails>,
    pagination: PaginationMetadata,
}

#[derive(Serialize, Deserialize)]
pub struct PaginationMetadata {
    total_records: u32,
    current_page: u32,
    total_pages: u32,
    next_page: Option<u32>,
    prev_page: Option<u32>,
}

pub async fn get_tx_by_memo(
    State(state): State<ServerState>,
    Path(memo): Path<String>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse>, Error> {
    info!("calling /tx_by_memo/:memo{}", memo);

    let page = pagination.page.unwrap_or(1);
    let limit = pagination.limit.unwrap_or(50);

    let offset = (page - 1) * limit;

    let total_records = state.db.get_total_tx_count_by_memo(memo.clone()).await?;
    let tx_counter: i64 = total_records.try_get("counter").unwrap_or(0);

    let rows = state.db.get_tx_memo(memo, limit, offset).await?;

    let mut infos: Vec<TxDetails> = Vec::new();

    for row in rows {
        let mut tx = TxDetails::try_from(row)?;

        // ignore the error for now
        _ = tx.decode_tx(&state.checksums_map);

        infos.push(tx);
    }

    // Calculate pagination metadata
    let total_pages = (tx_counter as f64 / limit as f64).ceil() as u32;
    let next_page = if page < total_pages {
        Some(page + 1)
    } else {
        None
    };
    let prev_page = if page > 1 { Some(page - 1) } else { None };

    Ok(Json(PaginatedResponse {
        data: infos,
        pagination: PaginationMetadata {
            total_records: tx_counter as u32,
            current_page: page,
            total_pages,
            next_page,
            prev_page,
        },
    }))
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
