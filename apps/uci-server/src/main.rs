use anyhow::Result;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;
use tracing::{info, error};

mod connection;
mod engine;

use connection::handle_connection;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(true)
        .with_level(true)
        .init();

    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).await?;
    info!("🚀 WebSocket server listening on ws://{}", addr);

    loop {
        match listener.accept().await {
            Ok((stream, peer_addr)) => {
                info!("New connection from: {}", peer_addr);
                tokio::spawn(async move {
                    if let Err(e) = handle_client(stream).await {
                        error!("Error handling client {}: {}", peer_addr, e);
                    }
                });
            }
            Err(e) => {
                error!("Error accepting connection: {}", e);
            }
        }
    }
}

async fn handle_client(stream: TcpStream) -> Result<()> {
    let ws_stream = accept_async(stream).await?;
    info!("WebSocket connection established");

    handle_connection(ws_stream).await?;

    info!("WebSocket connection closed");
    Ok(())
}
