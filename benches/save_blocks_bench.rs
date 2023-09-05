use criterion::{criterion_group, criterion_main, Criterion};

use namada_prototype::utils::load_checksums;
use tendermint::block::Height;

mod utils;

const DATABASE_NAME: &str = "save_blocks_db";

fn configure_criterion() -> Criterion {
    Criterion::default().sample_size(10).noise_threshold(0.05) // 5% noise threshold
}

fn save_blocks_benchmark(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let save_blocks_db = rt.block_on(async {
        // get helper database to create/destroy a new db used for this specific benchmark
        let helper_db = utils::helper_db().await;

        // destroy bench database if it exists
        utils::destroy_bench_database(helper_db.pool(), DATABASE_NAME).await;

        // a fresh state to ensure best benchmarking results
        let db = utils::create_bench_database(helper_db.pool(), DATABASE_NAME).await;

        db.create_tables().await.unwrap();
        db
    });

    let mut block_idx: u32 = 1;

    // load testing data
    let mut blocks = utils::load_blocks();
    let checksums_map = load_checksums().unwrap();

    // start benchmarking here
    c.bench_function("save_block", |b| {
        b.iter(|| {
            rt.block_on(async {
                // this allows us to avoid collisions
                // in the database, due to repeated blocks.
                let iter = blocks.iter_mut().map(|b| {
                    b.header.height = Height::from(block_idx);
                    block_idx += 1;
                    b
                });

                utils::save_blocks(&save_blocks_db, iter, &checksums_map).await
            });
        });
    });
}

criterion_group! {
    name = save_blocks_benches;
    config = configure_criterion();
    targets = save_blocks_benchmark
}
criterion_main!(save_blocks_benches);
