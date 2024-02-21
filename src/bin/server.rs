use namadexer::setup_logging;
use namadexer::start_server;
use namadexer::Database;
use namadexer::Error;
use namadexer::Settings;

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<(), Error> {
    let cfg = Settings::new()?;

    setup_logging(&cfg);

    let db = Database::new(cfg.database_config(), cfg.chain_name.as_str()).await?;

    // Start JSON server
    start_server(db, cfg.server_config()).await?;

    Ok(())
}
