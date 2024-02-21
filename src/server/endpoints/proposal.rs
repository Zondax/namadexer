use crate::{
    server::{ProposalDetails, ServerState},
    Error,
};
use axum::{
    extract::{Path, State},
    Json,
};
use sqlx::Row as TRow;
use tracing::info;

pub async fn get_proposal(
    State(state): State<ServerState>,
    Path(id): Path<i32>,
) -> Result<Json<Option<ProposalDetails>>, Error> {
    info!("calling /proposal/:id{}", id);

    let row = state.db.get_proposal(&id).await?;
    let Some(row) = row else {
        return Ok(Json(None));
    };
    let mut prop = ProposalDetails::try_from(row)?;

    // Get Votes
    let delegations = state.db.vote_proposal_delegations(prop.id as u64).await?;

    let delegations: Vec<String> = delegations
        .into_iter()
        .filter_map(|row| {
            row.try_get::<Option<String>, _>("delegator_id")
                .ok()
                .flatten()
        })
        .collect::<Vec<String>>();

    prop.add_votes(delegations)?;

    Ok(Json(Some(prop)))
}

// Return a list of the shielded assets and their total compiled using all the shielded transactions (in, internal and out)
pub async fn get_proposals(
    State(state): State<ServerState>,
) -> Result<Json<Vec<ProposalDetails>>, Error> {
    info!("calling /proposals");

    let rows = state.db.get_proposals().await?;

    let mut props: Vec<ProposalDetails> = Vec::new();

    for row in rows {
        let prop = ProposalDetails::try_from(row)?;

        props.push(prop);
    }

    Ok(Json(props))
}
