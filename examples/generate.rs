// Generate tests vectors blocks
use namada::tendermint::block::Height;
use namada::tendermint_rpc::{Client, HttpClient};
use std::fs::File;
use std::io::Write;

#[tokio::main]
async fn main() {
    let client = HttpClient::new("http://194.163.180.253:26657").unwrap();
    let mut current_height: u32 = 1;
    let mut f = File::create("./blocks_vector.json").unwrap();

    write!(f, "[").unwrap();

    loop {
        let height = Height::from(current_height);
        let response = client.block(height).await;

        if let Ok(resp) = response {
            let b = serde_json::to_string(&resp.block).unwrap();
            write!(f, "{}", b).unwrap();
        }

        current_height += 1;

        if current_height > 300 {
            break;
        }

        write!(f, ",").unwrap();
    }

    write!(f, "]").unwrap();
}
