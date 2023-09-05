use criterion::{criterion_group, criterion_main, Criterion};

use namadexer::BlockInfo;
use namadexer::{utils::load_checksums, Database};
use sqlx::Row;
use std::convert::TryFrom;

use std::ops::Range;

mod utils;

const DATABASE_NAME: &str = "get_block_db";

async fn tx_ids(db: &Database, block_id: &[u8]) -> Vec<Vec<u8>> {
    let rows = db.get_tx_hashes_block(block_id).await.unwrap();

    rows.iter()
        .map(|row| {
            let tx: Vec<u8> = row.try_get("hash").unwrap();
            tx
        })
        .collect()
}

async fn get_blocks(db: &Database, blocks: Range<u32>) {
    for idx in blocks {
        let row = db
            .block_by_height(idx)
            .await
            .unwrap()
            .expect("block does not exist!!");

        let block_id = BlockInfo::try_from(&row)
            .expect("Failed parsing row -> BlockInfo")
            .block_id;

        _ = tx_ids(db, &block_id).await;
    }
}

async fn prepare_database() -> Database {
    // get helper database to create/destroy a new db used for this specific benchmark
    let helper_db = utils::helper_db().await;

    // destroy bench database if it exists
    utils::destroy_bench_database(helper_db.pool(), DATABASE_NAME).await;

    // a fresh state to ensure best benchmarking results
    let db = utils::create_bench_database(helper_db.pool(), DATABASE_NAME).await;

    db.create_tables().await.unwrap();

    // populate bench database with blocks
    let mut blocks = utils::load_blocks();
    let checksums = load_checksums().unwrap();

    utils::save_blocks(&db, blocks.iter_mut(), &checksums).await;

    db
}

fn configure_criterion() -> Criterion {
    Criterion::default().sample_size(10).noise_threshold(0.05) // 5% noise threshold
}

fn get_blocks_benchmark(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let bench_db = rt.block_on(async { prepare_database().await });

    // start benchmarking here
    c.bench_function("get_block", |b| {
        b.iter(|| {
            // get all blocks per iteration
            rt.block_on(async { get_blocks(&bench_db, 1..301).await });
        });
    });
}

criterion_group! {
    name = get_block_bench;
    config = configure_criterion();
    targets = get_blocks_benchmark
}

criterion_main!(get_block_bench);
