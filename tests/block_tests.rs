mod utils;

use utils::{start_server, testing_db};

#[cfg(test)]
mod block_tests {
    use namadexer::BlockInfo;

    use super::*;

    #[tokio::test]
    async fn block_by_id() {
        let db = testing_db().await;

        // start a testing server an gives back the server address
        let addr = start_server(db).unwrap();

        let address = format!("http://{}:{}", addr.ip(), addr.port());
        let hc = httpc_test::new_client(address).expect("Server not running?");
        let response = hc
            .do_get("/block/height/1")
            .await
            .expect("Block does not exist");

        response.print().await.unwrap();

        let header = response.json_body_as::<BlockInfo>().unwrap();

        let hash_str = hex::encode(&header.block_id.0);

        // now retrieve same block but by hash:
        let new_header = hc
            .do_get(&format!("/block/hash/{hash_str}"))
            .await
            .expect("Block does not exist")
            .json_body_as::<BlockInfo>()
            .unwrap();

        assert_eq!(header, new_header);
    }

    #[tokio::test]
    async fn last_block() {
        let db = testing_db().await;

        // start a testing server an gives back the server address
        let addr = start_server(db).unwrap();

        let address = format!("http://{}:{}", addr.ip(), addr.port());
        let hc = httpc_test::new_client(address).expect("Server not running?");
        let response = hc
            .do_get("/block/last")
            .await
            .expect("Block does not exist");

        response.print().await.unwrap();

        let header = response.json_body_as::<BlockInfo>().unwrap();

        let height = header.header.height.value();

        // our testing database contains 300 blocks
        assert_eq!(300, height);
    }

    #[tokio::test]
    async fn block_with_tx() {
        let db = testing_db().await;
        // start server

        // start a testing server an gives back the server address
        let addr = start_server(db).unwrap();

        let address = format!("http://{}:{}", addr.ip(), addr.port());
        let hc = httpc_test::new_client(address).expect("Server not running?");
        let response = hc
            .do_get("/block/hash/2b719da000dd1caef4d401dbf9908610ce9059d052654887886957d8d1256d1a")
            .await
            .expect("Block does not exist");

        response.print().await.unwrap();

        let header = response.json_body_as::<BlockInfo>().unwrap();

        assert_eq!(header.tx_hashes.len(), 197)
    }
}
