// Generate tests vectors blocks
use namada::tendermint::block::Height;
use namada::tendermint_rpc::{Client, HttpClient};
use std::fs::File;
use std::io::Write;

#[tokio::main]
async fn main() {
    let client = HttpClient::new("http://194.163.180.253:26657").unwrap();
    let mut current_height: u32 = 1;
    let mut f1 = File::create("./blocks_vector.json").unwrap();
    let mut f2 = File::create("./block_results_vector.json").unwrap();

    write!(f1, "[").unwrap();
    write!(f2, "[").unwrap();

    loop {
        let height = Height::from(current_height);
        let response1 = client.block(height).await;
        let response2 = client.block_results(height).await;

        if let Ok(resp) = response1 {
            let b = serde_json::to_string(&resp.block).unwrap();
            write!(f1, "{}", b).unwrap();
        }

        if let Ok(resp) = response2 {
            let b = serde_json::to_string(&resp).unwrap();
            write!(f2, "{}", b).unwrap();
        }

        current_height += 1;

        if current_height > 300 {
            break;
        }

        write!(f1, ",").unwrap();
        write!(f2, ",").unwrap();
    }

    write!(f1, "]").unwrap();
    write!(f2, "]").unwrap();

}
