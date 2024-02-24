use crate::error::Error;
use clap::{ArgAction, Parser};
use config::{Config, File};
use serde::Deserialize;
use std::{env, net::SocketAddr};
use tracing::{debug, instrument};

const ENV_VAR_NAME: &str = "INDEXER_CONFIG_PATH";

pub const SERVER_ADDR: &str = "127.0.0.1";
pub const SERVER_PORT: u16 = 30303;

pub const TENDERMINT_ADDR: &str = "http://127.0.0.1:26657";

pub const JAEGER_HOST: &str = "localhost";
pub const JAEGER_PORT: u16 = 6831;

pub const PROMETHEUS_HOST: &str = "localhost";
pub const PROMETHEUS_PORT: u16 = 9000;

pub const DEFAULT_CHAIN_NAME: &str = "public-testnet-15";

pub const DEFAULT_LOG_FORMAT: &str = "pretty";

#[derive(Debug, Deserialize)]
pub struct IndexerConfig {
    pub tendermint_addr: String,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub serve_at: String,
    pub port: u16,
    pub cors_allow_origins: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub user: String,
    pub password: String,
    pub dbname: String,
    #[serde(default = "default_db_port")]
    pub port: u16,
    // The limit in seconds to wait for a ready database connection
    pub connection_timeout: Option<u64>,
    //  Give the option to skip the index creation
    pub create_index: bool,
}

const fn default_db_port() -> u16 {
    5432
}

#[derive(Debug, Deserialize)]
pub struct JaegerConfig {
    pub enable: bool,
    pub host: String,
    pub port: u16,
}

impl Default for JaegerConfig {
    fn default() -> Self {
        Self {
            enable: false,
            host: JAEGER_HOST.to_string(),
            port: JAEGER_PORT,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct PrometheusConfig {
    pub host: String,
    pub port: u16,
}

impl Default for PrometheusConfig {
    fn default() -> Self {
        Self {
            host: PROMETHEUS_HOST.to_string(),
            port: PROMETHEUS_PORT,
        }
    }
}
impl PrometheusConfig {
    pub fn address(&self) -> Result<SocketAddr, Error> {
        let listen_at = format!("{}:{}", self.host, self.port);
        listen_at.parse().map_err(Error::from)
    }
}

impl Default for IndexerConfig {
    fn default() -> Self {
        Self {
            tendermint_addr: TENDERMINT_ADDR.to_owned(),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            serve_at: SERVER_ADDR.to_owned(),
            port: SERVER_PORT,
            cors_allow_origins: vec![],
        }
    }
}

impl ServerConfig {
    pub fn address(&self) -> Result<SocketAddr, Error> {
        let listen_at = format!("{}:{}", self.serve_at, self.port);
        listen_at.parse().map_err(Error::from)
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_owned(),
            user: "postgres".to_owned(),
            password: "wow".to_owned(),
            dbname: "blockchain".to_owned(),
            port: 5432,
            connection_timeout: None,
            create_index: true,
        }
    }
}

#[derive(Debug, Default, Deserialize, Clone, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum LogFormat {
    Json,
    #[default]
    Pretty,
}

#[derive(Debug, Deserialize, clap::Parser)]
pub struct CliSettings {
    #[clap(long, env, default_value = "")]
    pub log_level: String,
    #[clap(long, env, default_value = DEFAULT_LOG_FORMAT)]
    pub log_format: LogFormat,
    #[clap(long, env, default_value = DEFAULT_CHAIN_NAME)]
    pub chain_name: String,
    #[clap(long, env, default_value = SERVER_ADDR)]
    pub server_serve_at: String,
    #[clap(long, env, default_value_t = SERVER_PORT)]
    pub server_port: u16,
    #[clap(long, env)]
    pub server_cors_allow_origin: Vec<String>,
    #[clap(long, env, default_value = "localhost")]
    pub database_host: String,
    #[clap(long, env, default_value = "postgres")]
    pub database_user: String,
    #[clap(long, env, default_value = "wow")]
    pub database_password: String,
    #[clap(long, env, default_value = "blockchain")]
    pub database_dbname: String,
    #[clap(long, env, default_value_t = 5432)]
    pub database_port: u16,
    #[clap(long, env)]
    pub database_connection_timeout: Option<u64>,
    #[clap(long, env, default_value = "true")]
    pub database_create_index: bool,
    #[clap(long, env, default_value = TENDERMINT_ADDR)]
    pub indexer_tendermint_addr: String,
    #[clap(long, env, action=ArgAction::SetFalse)]
    pub jaeger_enable: bool,
    #[clap(long, env, default_value = JAEGER_HOST)]
    pub jaeger_host: String,
    #[clap(long, env, default_value_t = JAEGER_PORT)]
    pub jaeger_port: u16,
    #[clap(long, env, default_value = PROMETHEUS_HOST)]
    pub prometheus_host: String,
    #[clap(long, env, default_value_t = PROMETHEUS_PORT)]
    pub prometheus_port: u16,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub log_level: String,
    #[serde(default)]
    pub log_format: LogFormat,
    pub chain_name: String,
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub indexer: IndexerConfig,
    pub jaeger: JaegerConfig,
    pub prometheus: PrometheusConfig,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            log_level: Default::default(),
            log_format: Default::default(),
            chain_name: DEFAULT_CHAIN_NAME.to_string(),
            database: Default::default(),
            server: Default::default(),
            indexer: Default::default(),
            jaeger: Default::default(),
            prometheus: Default::default(),
        }
    }
}

impl From<CliSettings> for Settings {
    fn from(value: CliSettings) -> Self {
        Self {
            log_level: value.log_level,
            log_format: value.log_format,
            chain_name: value.chain_name,
            database: DatabaseConfig {
                host: value.database_host,
                user: value.database_user,
                password: value.database_password,
                dbname: value.database_dbname,
                port: value.database_port,
                connection_timeout: value.database_connection_timeout,
                create_index: value.database_create_index,
            },
            server: ServerConfig {
                serve_at: value.server_serve_at,
                port: value.server_port,
                cors_allow_origins: value.server_cors_allow_origin,
            },
            indexer: IndexerConfig {
                tendermint_addr: value.indexer_tendermint_addr,
            },
            jaeger: JaegerConfig {
                enable: value.jaeger_enable,
                host: value.jaeger_host,
                port: value.jaeger_port,
            },
            prometheus: PrometheusConfig {
                host: value.prometheus_host,
                port: value.prometheus_port,
            },
        }
    }
}

impl Settings {
    #[instrument(level = "debug")]
    pub fn new() -> Result<Self, Error> {
        // Try to read INDEXER_CONFIG_PATH env variable
        // otherwise use default settings.
        if let Ok(path) = env::var(ENV_VAR_NAME) {
            debug!("Reading configuration file from {}", path);

            let config = Config::builder()
                .add_source(File::with_name(&path))
                .build()?;

            let settings: Self = config.try_deserialize().map_err(Error::from)?;

            // verify if chain_name is correct
            if settings.chain_name.contains('.') {
                panic!("chain_name cannot contains '.' (example of valid chain_name 'public-testnet-14')")
            }

            return Ok(settings);
        }

        let cli_settings = CliSettings::parse();
        let settings = Settings::from(cli_settings);

        Ok(settings)
    }

    pub fn server_config(&self) -> &ServerConfig {
        &self.server
    }

    pub fn database_config(&self) -> &DatabaseConfig {
        &self.database
    }

    pub fn indexer_config(&self) -> &IndexerConfig {
        &self.indexer
    }

    pub(crate) fn jaeger_config(&self) -> &JaegerConfig {
        &self.jaeger
    }

    pub fn prometheus_config(&self) -> &PrometheusConfig {
        &self.prometheus
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use config::{Config, File};
    use std::path::PathBuf;

    #[test]
    fn test_settings_deserialization() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("config/Settings.example.toml");

        assert!(
            d.exists(),
            "Settings.example.toml file does not exist at {:?}",
            d
        );

        let config = Config::builder()
            .add_source(File::from(d))
            .build()
            .expect("Failed to build config");

        let _: Settings = config
            .try_deserialize()
            .expect("Failed to deserialize Settings.example.toml into the Settings struct");
    }
}
