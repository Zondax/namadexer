use namada_prototype::setup_logging;
use namada_prototype::start_server;
use namada_prototype::Database;
use namada_prototype::Error;
use namada_prototype::Settings;

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<(), Error> {
    let cfg = Settings::new()?;

    setup_logging(&cfg);

    let db = Database::new(cfg.database_config()).await?;

    // Start JSON server
    start_server(db, cfg.server_config()).await?;

    Ok(())
}
