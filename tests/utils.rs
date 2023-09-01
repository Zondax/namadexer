use std::net::SocketAddr;

use namada_prototype::create_server;
use namada_prototype::Database;
use namada_prototype::Error as NError;
use namada_prototype::ServerConfig;

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
