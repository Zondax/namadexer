use criterion::{criterion_group, criterion_main, Criterion};

use namada_prototype::{utils::load_checksums, Database, Settings};
use sqlx::postgres::PgPoolOptions;
use sqlx::query;
use sqlx::PgPool;
use std::collections::HashMap;
use std::fs;
use std::time::Duration;
use tendermint::block::Block;
use tendermint::block::Height;

const DATABASE_NAME: &str = "bench_db";

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

    query(&db_query)
        .execute(pool)
        .await
        .expect("Could not create database for benchmarks");
}

async fn create_bench_database(pg_pool: &PgPool) -> Database {
    let config = Settings::new().unwrap();
    let config = config.database_config();

    // lets connect to our default database, so from there
    // we create another database that is going to be used for
    // benches.
    create_db(pg_pool, DATABASE_NAME).await;

    // Now connect to the just created db
    let config = format!(
        "postgres://{}:{}@{}/{}",
        config.user, config.password, config.host, DATABASE_NAME,
    );

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(30))
        .connect(&config)
        .await
        .expect("Could not connect to bench database");

    Database::with_pool(pool)
}

async fn save_blocks_bench(
    db: &Database,
    blocks: impl Iterator<Item = &mut Block>,
    checksums: &HashMap<String, String>,
) {
    for block in blocks {
        // we need to update the block height
        // to avoid collisions between bench runs
        db.save_block(&block, &checksums).await.unwrap();
    }
}

fn configure_criterion() -> Criterion {
    Criterion::default()
        .sample_size(100) // For example, set sample size
        .noise_threshold(0.05) // 5% noise threshold
}

fn criterion_benchmark(c: &mut Criterion) {
    let config = Settings::new().unwrap();
    let config = config.database_config();

    let checksums_map = load_checksums().unwrap();

    let data = fs::read_to_string("./tests/blocks_vector.json").unwrap();
    let mut blocks: Vec<Block> = serde_json::from_str(&data).unwrap();

    let rt = tokio::runtime::Runtime::new().expect("could not create runtime");

    // this database is used to connect to our default database which is used by tests.
    // but we use it here to create an alternative database to use for benches
    let db = rt.block_on(async { Database::new(config).await.unwrap() });

    // destroy bench database if it exists
    rt.block_on(async {
        destroy_db(db.pool(), DATABASE_NAME).await;
    });

    let bench_db = rt.block_on(async {
        let db = create_bench_database(db.pool()).await;

        db.create_tables().await.unwrap();
        db
    });

    let mut block_idx: u32 = 0;

    // start benchmarking here
    c.bench_function("function_name", |b| {
        b.iter(|| {
            rt.block_on(async {
                // this allows us to avoid collisions
                // in the database, due to repeated blocks.
                let iter = blocks.iter_mut().map(|b| {
                    b.header.height = Height::from(block_idx);
                    block_idx += 1;
                    b
                });

                save_blocks_bench(&bench_db, iter, &checksums_map).await
            });
        });
    });
}

// criterion_group!(benches, criterion_benchmark);
criterion_group! {
    name = benches;
    config = configure_criterion();
    targets = criterion_benchmark
}
criterion_main!(benches);
