use namadexer::{create_server, Database, Error as NError, ServerConfig, Settings};
use sqlx::query;
use sqlx::PgPool;
use std::net::SocketAddr;

pub const TESTING_DB_NAME: &str = "testingdb";
const NETWORK: &str = "testnet";

// start a server and return its address
pub fn start_server(db: Database) -> Result<SocketAddr, NError> {
    let config = ServerConfig {
        serve_at: "127.0.0.1".to_string(),
        // lets use default port 0, here the operating system will assign a free port dinamically
        // this ensure there would not be conflicts with other server instances started by other
        // tests
        port: 0,
        cors_allow_origins: vec![],
    };

    let (socket, server) = create_server(db, &config)?;

    tokio::spawn(server);

    Ok(socket)
}

async fn create_db(pool: &PgPool, name: &str) {
    // now create bench database
    let db_query = format!("CREATE DATABASE {}", name);

    query(&db_query)
        .execute(pool)
        .await
        .expect("Could not create database for benchmarks");
}

async fn destroy_db(pool: &PgPool, name: &str) {
    let db_query = format!("DROP DATABASE {}", name);

    _ = query(&db_query).execute(pool).await
}

pub async fn create_test_db(pg_pool: &PgPool, name: &str) -> Database {
    create_db(pg_pool, name).await;

    // now connect to the just created db
    let mut config = Settings::new().unwrap();
    config.database.dbname = name.to_string();

    let config = config.database_config();

    Database::new(config, NETWORK).await.unwrap()
}

pub async fn destroy_test_db(pg_pool: &PgPool, name: &str) {
    destroy_db(pg_pool, name).await
}

// Returns a database that is used for creating the db used for testing,
// here we connect to postgres database.
pub async fn helper_db() -> Database {
    let mut config = Settings::new().unwrap();
    // Connect to default postgres database
    config.database.dbname = "postgres".to_string();

    let config = config.database_config();

    Database::new(config, NETWORK).await.unwrap()
}

pub async fn testing_db() -> Database {
    let mut config = Settings::new().unwrap();
    config.database.dbname = TESTING_DB_NAME.to_string();

    let config = config.database_config();
    Database::new(config, NETWORK).await.unwrap()
}
