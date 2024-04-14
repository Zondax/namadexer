use namadexer::setup_logging;
use namadexer::start_indexing;
use namadexer::Database;
use namadexer::Error;

use tracing::info;

#[cfg(feature = "prometheus")]
use namadexer::PrometheusConfig;

use namadexer::Settings;

#[cfg(feature = "prometheus")]
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder};

// Block/Evidences/Transactions insterts duration buckets to
// "scale" metrics in prometheus.
// we could tweek this
pub const DB_SAVE_DATA_DURATION_MS_BUCKETS: &[f64; 26] = &[
    0.005 * 1000.0,
    0.01 * 1000.0,
    0.25 * 1000.0,
    0.5 * 1000.0,
    1.0 * 1000.0,
    1.5 * 1000.0,
    2.0 * 1000.0,
    2.5 * 1000.0,
    3.5 * 1000.0,
    4.0 * 1000.0,
    4.5 * 1000.0,
    5.0 * 1000.0,
    5.5 * 1000.0,
    6.0 * 1000.0,
    6.5 * 1000.0,
    7.0 * 1000.0,
    8.0 * 1000.0,
    10.0 * 1000.0,
    15.0 * 1000.0,
    20.0 * 1000.0,
    22.5 * 1000.0,
    25.0 * 1000.0,
    30.0 * 1000.0,
    40.0 * 1000.0,
    50.0 * 1000.0,
    60.0 * 1000.0,
];

pub const DB_SAVE_DATA_BATCH_SIZE_BUCKETS: &[f64; 10] =
    &[1.0, 2.0, 5.0, 10.0, 25.0, 50.0, 100.0, 200.0, 300.0, 400.0];

pub const GET_BLOCK_DURATION_SECONDS_BUCKETS: &[f64; 11] = &[
    0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
];

#[cfg(feature = "prometheus")]
async fn start_metrics_server(cfg: &PrometheusConfig) -> Result<(), Error> {
    let address = cfg.address()?;

    PrometheusBuilder::new()
        .with_http_listener(address)
        .set_buckets_for_metric(
            Matcher::Prefix("db_save_duration_".to_string()),
            DB_SAVE_DATA_DURATION_MS_BUCKETS,
        )?
        .set_buckets_for_metric(
            Matcher::Prefix("db_save_batch_size_".to_string()),
            DB_SAVE_DATA_BATCH_SIZE_BUCKETS,
        )?
        .set_buckets_for_metric(
            Matcher::Prefix(namadexer::INDEXER_GET_BLOCK_DURATION.to_string()),
            GET_BLOCK_DURATION_SECONDS_BUCKETS,
        )?
        .install()
        .map_err(Error::from)
}

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<(), Error> {
    let cfg = Settings::new()?;

    setup_logging(&cfg);

    info!("Starting database connection");

    let db = Database::new(cfg.database_config(), cfg.chain_name.as_str()).await?;
    info!("Creating tables");
    db.create_tables().await?;

    // start metrics service
    #[cfg(feature = "prometheus")]
    start_metrics_server(cfg.prometheus_config()).await?;

    let network = db.network.clone();

    info!("Starting indexer");
    start_indexing(
        db,
        cfg.indexer_config(),
        network.as_str(),
        cfg.database.create_index,
    )
    .await
}
