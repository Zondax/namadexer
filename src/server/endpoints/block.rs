use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;

use crate::{
    server::ServerState,
    BlockInfo, Error,
};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum LatestBlock {
    LastBlock(Box<BlockInfo>),
    LatestBlocks(LatestBlocks),
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct LatestBlocks {
    pub blocks: Vec<BlockInfo>,
}

pub async fn get_block_by_hash(
    State(state): State<ServerState>,
    Path(hash): Path<String>,
) -> Result<Json<Option<BlockInfo>>, Error> {
    info!("calling /block/hash/:block_hash");

    let id = hex::decode(hash)?;

    let row = state.db.block_by_id(&id).await?;
    let Some(row) = row else {
        return Ok(Json(None));
    };
    let block = BlockInfo::try_from(&row)?;

    Ok(Json(Some(block)))
}

pub async fn get_block_by_height(
    State(state): State<ServerState>,
    Path(height): Path<u32>,
) -> Result<Json<Option<BlockInfo>>, Error> {
    info!("calling /block/height/:block_height");

    let row = state.db.block_by_height(height).await?;
    let Some(row) = row else {
        return Ok(Json(None));
    };

    let block = BlockInfo::try_from(&row)?;

    Ok(Json(Some(block)))
}

pub async fn get_last_block(
    State(state): State<ServerState>,
    Query(params): Query<HashMap<String, i32>>,
) -> Result<Json<LatestBlock>, Error> {
    info!("calling /block/last");

    let num = params.get("num");
    let offset = params.get("offset");

    if let Some(n) = num {
        let rows = state.db.get_lastest_blocks(n, offset).await?;
        let mut blocks: LatestBlocks = LatestBlocks { blocks: vec![] };

        for row in rows {
            let block = BlockInfo::try_from(&row)?;

            blocks.blocks.push(block);
        }

        Ok(Json(LatestBlock::LatestBlocks(blocks)))
    } else {
        let row = state.db.get_last_block().await?;

        let block = BlockInfo::try_from(&row)?;

        Ok(Json(LatestBlock::LastBlock(Box::new(block))))
    }
}
