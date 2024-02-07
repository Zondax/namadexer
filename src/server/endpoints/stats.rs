use axum::{
    extract::State,
    Json,
};
use tendermint::{block::Height, chain, Time};

use crate::{
    server::ServerState, BlockInfo, Error
};
use sqlx::Row as TRow;
use sqlx::postgres::PgRow as Row;
use tracing::instrument;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct TxStat {
    pub return_code: i32,
    pub tx_count: i64,
}

impl TryFrom<&Row> for TxStat {
    type Error = Error;

    #[instrument(level = "trace", skip(row))]
    fn try_from(row: &Row) -> Result<Self, Self::Error> {

        let return_code: i32 = row.try_get("return_code")?;
        let tx_count: i64 = row.try_get("tx_count")?;

        Ok(TxStat {
            return_code: return_code,
            tx_count: tx_count,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct StatsResponse {
    pub tx_stats: Vec<TxStat>,
    pub chain_id: chain::Id,
    pub time: Time,
    pub height: Height
}

pub async fn get_stats(State(state): State<ServerState>) -> Result<Json<StatsResponse>, Error> {

    let rows = state.db.get_tx_stats().await?;    

    let mut tx_stats: Vec<TxStat> = Vec::new();

    // get all tx stats
    for row in rows {
        let tx = TxStat::try_from(&row)?;
        tx_stats.push(tx);
    };


    // get last block info
    let block_row = state.db.get_last_block().await?;
    let block = BlockInfo::try_from(&block_row)?;

    Ok(Json(StatsResponse{
        tx_stats: tx_stats,
        chain_id: block.header.chain_id,
        time: block.header.time,
        height: block.header.height
    }))
}
