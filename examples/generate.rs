// Generate tests vectors blocks
use std::fs::File;
use std::io::Write;
use tendermint::block::Height;
use tendermint_rpc::{self, Client, HttpClient};

const URL: &str = "http://194.163.180.253:26657";
const CURRENT_HEIGHT: u32 = 1;

#[tokio::main]
async fn main() {
    let client = HttpClient::new(URL).unwrap();
    let mut current_height = CURRENT_HEIGHT;
    let mut f1 = File::create("./tests/blocks_vector.json").unwrap();
    let mut f2 = File::create("./tests/block_results_vector.json").unwrap();

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

        if current_height > CURRENT_HEIGHT + 300 {
            break;
        }

        write!(f1, ",").unwrap();
        write!(f2, ",").unwrap();
    }

    write!(f1, "]").unwrap();
    write!(f2, "]").unwrap();
}
