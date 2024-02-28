use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::Row as TRow;
use std::collections::HashMap;
use tracing::info;

use crate::{
    server::{blocks::HashID, blocks::TxShort, ServerState},
    BlockInfo, Error,
};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum LatestBlock {
    LastBlock(Box<BlockInfo>),
    LatestBlocks(Vec<BlockInfo>),
}

async fn get_tx_hashes(
    state: &ServerState,
    block: &mut BlockInfo,
    hash: &[u8],
) -> Result<(), Error> {
    let rows = state.db.get_tx_hashes_block(hash).await?;

    let mut tx_hashes: Vec<TxShort> = vec![];
    for row in rows.iter() {
        let hash_id = HashID(row.try_get("hash")?);
        let tx_type: String = row.try_get("tx_type")?;
        tx_hashes.push(TxShort { tx_type, hash_id });
    }

    block.tx_hashes = tx_hashes;

    Ok(())
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
    let mut block = BlockInfo::try_from(&row)?;

    let block_id: Vec<u8> = row.try_get("block_id")?;
    get_tx_hashes(&state, &mut block, &block_id).await?;

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

    let mut block = BlockInfo::try_from(&row)?;

    let block_id: Vec<u8> = row.try_get("block_id")?;
    get_tx_hashes(&state, &mut block, &block_id).await?;

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
        let mut blocks: Vec<BlockInfo> = vec![];

        for row in rows {
            let mut block = BlockInfo::try_from(&row)?;

            let block_id: Vec<u8> = row.try_get("block_id")?;
            get_tx_hashes(&state, &mut block, &block_id).await?;

            blocks.push(block);
        }

        Ok(Json(LatestBlock::LatestBlocks(blocks)))
    } else {
        let row = state.db.get_last_block().await?;

        let mut block = BlockInfo::try_from(&row)?;

        let block_id: Vec<u8> = row.try_get("block_id")?;
        get_tx_hashes(&state, &mut block, &block_id).await?;

        Ok(Json(LatestBlock::LastBlock(Box::new(block))))
    }
}
