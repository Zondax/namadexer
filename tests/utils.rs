use std::net::SocketAddr;

use namadexer::create_server;
use namadexer::Database;
use namadexer::Error as NError;
use namadexer::ServerConfig;

// start a server and return its address
pub fn start_server(db: Database) -> Result<SocketAddr, NError> {
    let config = ServerConfig {
        serve_at: "127.0.0.1".to_string(),
        // lets use default port 0, here the operating system will assign a free port dinamically
        // this ensure there would not be conflicts with other server instances started by other
        // tests
        port: 0,
    };

    let (socket, server) = create_server(db, &config)?;

    tokio::spawn(async move { server.await });

    Ok(socket)
}
