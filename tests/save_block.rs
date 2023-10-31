mod utils;

#[cfg(test)]
mod save_block {
    use namadexer::utils::load_checksums;
    use std::fs;
    use tendermint::block::Block;

    use crate::utils::{create_test_db, destroy_test_db, helper_db, TESTING_DB_NAME};

    #[tokio::test]
    async fn save_block() {
        let helper_db = helper_db().await;

        destroy_test_db(helper_db.pool(), TESTING_DB_NAME).await;

        // now create a fresh database for tests
        let db = create_test_db(helper_db.pool(), TESTING_DB_NAME).await;

        let checksums_map = load_checksums().unwrap();

        let data = fs::read_to_string("./tests/blocks_vector.json").unwrap();
        let blocks: Vec<Block> = serde_json::from_str(&data).unwrap();

        db.create_tables().await.unwrap();

        for block in blocks {
            db.save_block(&block, &checksums_map).await.unwrap();
        }

        // assert!(db.create_indexes().await.is_ok());
        db.create_indexes()
            .await
            .expect("Something went wrong creating database indexes");
    }
}
