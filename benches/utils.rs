use namadexer::{Database, Settings};
use sqlx::postgres::PgPoolOptions;
use sqlx::query;
use sqlx::PgPool;
use std::collections::HashMap;
use std::fs;
use std::time::Duration;
use tendermint::block::Block;

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

pub async fn create_bench_database(pg_pool: &PgPool, name: &str) -> Database {
    let config = Settings::new().unwrap();
    let config = config.database_config();

    // lets connect to our default database, so from there
    // we create another database that is going to be used for
    // benches.
    create_db(pg_pool, name).await;

    // Now connect to the just created db
    let config = format!(
        "postgres://{}:{}@{}/{}",
        config.user, config.password, config.host, name,
    );

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(30))
        .connect(&config)
        .await
        .expect("Could not connect to bench database");

    Database::with_pool(pool)
}

pub async fn destroy_bench_database(pg_pool: &PgPool, name: &str) {
    destroy_db(pg_pool, name).await
}

pub fn load_blocks() -> Vec<Block> {
    let data = fs::read_to_string("./tests/blocks_vector.json").unwrap();
    serde_json::from_str(&data).unwrap()
}

pub async fn helper_db() -> Database {
    let config = Settings::new().unwrap();
    let config = config.database_config();
    Database::new(config).await.unwrap()
}

pub async fn save_blocks(
    db: &Database,
    blocks: impl Iterator<Item = &mut Block>,
    checksums: &HashMap<String, String>,
) {
    for block in blocks {
        db.save_block(&block, &checksums).await.unwrap();
    }
}
