mod config;
pub mod database;
mod error;
mod indexer;
pub mod server;
pub mod tables;
mod telemetry;
pub mod utils;

pub use crate::config::{IndexerConfig, JaegerConfig, PrometheusConfig, ServerConfig, Settings};
pub use database::Database;
pub use error::Error;
pub use indexer::start_indexing;
pub use server::{create_server, start_server, BlockInfo};
pub use telemetry::{get_subscriber, init_subscriber, setup_logging};

pub const INDEXER_GET_BLOCK_DURATION: &str = "indexer_get_block_duration";
const DB_SAVE_BLOCK_COUNTER: &str = "db_save_block_count";
const DB_SAVE_BLOCK_DURATION: &str = "db_save_block_duration";
const DB_SAVE_TXS_DURATION: &str = "db_save_transactions_duration";
const DB_SAVE_EVDS_DURATION: &str = "db_save_evidences_duration";

pub const MASP_ADDR: &str =
    "atest1v4ehgw36xaryysfsx5unvve4g5my2vjz89p52sjxxgenzd348yuyyv3hg3pnjs35g5unvde4ca36y5";
