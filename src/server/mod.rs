use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};

#[cfg(feature = "prometheus")]
use axum_prometheus::{PrometheusMetricLayerBuilder, AXUM_HTTP_REQUESTS_DURATION_SECONDS};
use futures_util::{Future, TryFutureExt};
#[cfg(feature = "prometheus")]
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder};
use sqlx::Row;
use std::{collections::HashMap, net::SocketAddr};
use tracing::{info, instrument};

use crate::config::ServerConfig;
use crate::database::Database;
use crate::error::Error;
use crate::utils::load_checksums;

pub mod blocks;
pub mod tx;
pub use blocks::BlockInfo;
pub use tx::TxInfo;
mod utils;
pub mod shielded;
pub(crate) use utils::{from_hex, serialize_hex};

pub const HTTP_DURATION_SECONDS_BUCKETS: &[f64; 11] = &[
    0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
];

#[derive(Clone)]
struct ServerState {
    db: Database,
    checksums_map: HashMap<String, String>,
}

async fn get_tx_hashes(
    state: &ServerState,
    block: &mut BlockInfo,
    hash: &[u8],
) -> Result<(), Error> {
    let rows = state.db.get_tx_hashes_block(hash).await?;

    let mut tx_hashes: Vec<Vec<u8>> = vec![];
    for row in rows.iter() {
        let hash: Vec<u8> = row.try_get("hash")?;
        tx_hashes.push(hash);
    }

    block.tx_hashes = tx_hashes;

    Ok(())
}

async fn get_block_by_height(
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

async fn get_block_by_hash(
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

async fn get_last_block(State(state): State<ServerState>) -> Result<Json<BlockInfo>, Error> {
    let row = state.db.get_last_block().await?;

    let mut block = BlockInfo::try_from(&row)?;

    let block_id: Vec<u8> = row.try_get("block_id")?;
    get_tx_hashes(&state, &mut block, &block_id).await?;

    Ok(Json(block))
}

async fn get_tx_by_hash(
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
async fn get_shielded_tx(
    State(state): State<ServerState>,
) -> Result<Json<shielded::ShieldedAssetsResponse>, Error> {
    let rows = state.db.get_shielded_tx().await?;

    let shielded_assests_response = shielded::ShieldedAssetsResponse::try_from(&rows)?;

    Ok(Json(shielded_assests_response))
}

fn server_routes(state: ServerState) -> Router<()> {
    Router::new()
        .route("/block/height/:block_height", get(get_block_by_height))
        .route("/block/hash/:block_hash", get(get_block_by_hash))
        .route("/block/last", get(get_last_block))
        .route("/tx/:tx_hash", get(get_tx_by_hash))
        .route("/shielded", get(get_shielded_tx))
        .with_state(state)
}

/// Returns a http server as a future so it needs to be pulled to start processing
/// incoming requests. The server address is also returned.
///
/// # Arguments
///
/// `db` The database for storing indexed data
///
/// `config` The server [configuration](ServerConfig) to use.
///
pub fn create_server(
    db: Database,
    config: &ServerConfig,
) -> Result<(SocketAddr, impl Future<Output = Result<(), Error>>), Error> {
    info!("Starting JSON server");

    let checksums_map = load_checksums()?;

    // JSON API server
    // we move the handler creation here so we propagate errors gracefully
    #[cfg(feature = "prometheus")]
    let prometheus_handle = {
        let builder = PrometheusBuilder::new().set_buckets_for_metric(
            Matcher::Full(AXUM_HTTP_REQUESTS_DURATION_SECONDS.to_string()),
            HTTP_DURATION_SECONDS_BUCKETS,
        )?;

        builder.install_recorder()?
    };

    #[cfg(feature = "prometheus")]
    let (prometheus_layer, metric_handle) = PrometheusMetricLayerBuilder::new()
        .with_prefix("server-metrics")
        .with_metrics_from_fn(|| prometheus_handle)
        .build_pair();

    let routes = server_routes(ServerState { db, checksums_map });

    #[cfg(feature = "prometheus")]
    let routes = routes
        .route("/metrics", get(|| async move { metric_handle.render() }))
        .layer(prometheus_layer);

    info!("server URL : {}:{}", &config.serve_at, &config.port);

    let server = axum::Server::try_bind(&config.address()?)
        .map_err(|e| Error::Generic(Box::new(e)))?
        .serve(routes.into_make_service());

    let local_addr = server.local_addr();

    Ok((local_addr, server.map_err(|e| Error::Generic(Box::new(e)))))
}

/// Starts a http server that listen for blocks and transactions requests.
///
/// # Arguments
///
/// `db` The database for storing indexed data
///
/// `config` The server [configuration](ServerConfig) to use.
///
/// Note:
/// This function starts a server blocking current thread, returning only
/// if server gets close or an error happens.
#[instrument(level = "trace", skip(db, config))]
pub async fn start_server(db: Database, config: &ServerConfig) -> Result<(), Error> {
    let (_, server) = create_server(db, config)?;

    server.await
}
