use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use std::error::Error as StdError;
use std::num::{ParseFloatError, ParseIntError};
use thiserror::Error as ThisError;
use tokio::task::JoinError;

use config::ConfigError;
use opentelemetry_api::metrics::MetricsError;
use tendermint::Error as TError;
use tendermint_rpc::endpoint::block_results;
use tendermint_rpc::Error as TRpcError;
use tokio::sync::mpsc::error::SendError;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("Invalid Block data")]
    InvalidBlockData,
    #[error("Invalid Transaction data (reason: {0})")]
    InvalidTxData(String),
    #[error("Tendermint error: {0}")]
    TendermintError(#[from] TError),
    #[error("Tendermint rpc_error: {0}")]
    TendermintRpcError(#[from] TRpcError),
    #[error("Configuration file error: {0}")]
    Config(#[from] ConfigError),
    #[error("Configuration error: {0}")]
    IO(#[from] std::io::Error),
    #[error("Address parsing error: {0}")]
    AddrError(#[from] std::net::AddrParseError),
    #[error("Server error: {0}")]
    ServerError(#[from] axum::Error),
    #[error("Hex error: {0}")]
    HexError(#[from] hex::FromHexError),
    #[error("Database error: {0}")]
    DB(#[from] sqlx::Error),
    #[error("std::env error: {0}")]
    EnvError(#[from] std::env::VarError),
    #[error("Tokio channel SendError")]
    SendError,
    #[error("tokio_error: {0}")]
    JoinError(#[from] JoinError),
    #[error("openetelemetry error: {0}")]
    MetricsError(#[from] MetricsError),

    #[cfg(feature = "prometheus")]
    #[error("PrometheusBuilder error: {0}")]
    PrometheusBuilderError(#[from] metrics_exporter_prometheus::BuildError),

    #[error("serde_json error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("Invalid checksum data")]
    InvalidChecksum,
    #[error("Unknow error: {0}")]
    Generic(Box<dyn StdError + Send>),
    #[error("ParseInt error")]
    ParseIntError(#[from] ParseIntError),
    #[error("ParseFloat error")]
    ParseFloatError(#[from] ParseFloatError),
    #[error("Timeout error")]
    Timeout(#[from] tokio::time::error::Elapsed),
}

impl From<SendError<(tendermint::Block, block_results::Response)>> for Error {
    fn from(_: SendError<(tendermint::Block, block_results::Response)>) -> Self {
        Self::SendError
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status = match self {
            Error::InvalidBlockData => StatusCode::EXPECTATION_FAILED,
            Error::InvalidTxData(_) => StatusCode::EXPECTATION_FAILED,
            Error::DB(_) => StatusCode::NOT_FOUND,
            Error::HexError(_) => StatusCode::BAD_REQUEST,
            Error::TendermintError(_) => StatusCode::EXPECTATION_FAILED,
            // errors bellow should not happen in the http handler context
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = Json(json!({
            "Error": self.to_string(),
        }));

        (status, body).into_response()
    }
}
