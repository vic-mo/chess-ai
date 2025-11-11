use anyhow::Result;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};
use tracing::{debug, error, info, warn};

use engine::types::SearchLimit;

use crate::engine::EngineManager;

#[derive(Debug, Deserialize)]
struct ClientMessage {
    #[serde(rename = "type")]
    msg_type: String,
    #[serde(default)]
    id: String,
    #[serde(default)]
    fen: String,
    #[serde(default)]
    limit: Option<SearchLimit>,
    #[serde(default)]
    uci_move: String,
}

#[derive(Debug, Serialize)]
pub struct ServerMessage {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub id: String,
    pub payload: serde_json::Value,
}

pub async fn handle_connection(ws_stream: WebSocketStream<TcpStream>) -> Result<()> {
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    // Channel for sending messages to WebSocket
    let (tx, mut rx) = mpsc::unbounded_channel::<ServerMessage>();

    // Spawn task to forward messages from channel to WebSocket
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Ok(json) = serde_json::to_string(&msg) {
                if let Err(e) = ws_sender.send(Message::Text(json)).await {
                    error!("Failed to send WebSocket message: {}", e);
                    break;
                }
            }
        }
    });

    // Create engine manager for this connection
    let mut engine = EngineManager::new(tx.clone());

    // Process incoming messages
    while let Some(msg) = ws_receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                info!("📨 Received message: {}", text);

                match serde_json::from_str::<ClientMessage>(&text) {
                    Ok(client_msg) => {
                        info!("✅ Parsed message type: {}", client_msg.msg_type);
                        if let Err(e) = handle_client_message(&mut engine, client_msg, tx.clone()).await {
                            error!("Error handling message: {}", e);
                        }
                    }
                    Err(e) => {
                        warn!("Failed to parse message: {}", e);
                    }
                }
            }
            Ok(Message::Close(_)) => {
                info!("Client closed connection");
                break;
            }
            Ok(Message::Ping(_)) => {
                debug!("Received ping, sending pong");
                // Pong is handled automatically by tokio-tungstenite
            }
            Ok(_) => {
                // Ignore binary, pong, and frame messages
            }
            Err(e) => {
                error!("WebSocket error: {}", e);
                break;
            }
        }
    }

    // Stop engine and wait for send task to complete
    engine.stop();
    drop(tx);
    let _ = send_task.await;

    Ok(())
}

async fn handle_client_message(
    engine: &mut EngineManager,
    msg: ClientMessage,
    tx: mpsc::UnboundedSender<ServerMessage>,
) -> Result<()> {
    match msg.msg_type.as_str() {
        "analyze" => {
            let id = msg.id.clone();
            let fen = msg.fen;
            let limit = msg.limit.ok_or_else(|| anyhow::anyhow!("Missing limit"))?;

            info!("Analyzing position: {} with limit {:?}", fen, limit);

            // Analyze position
            engine.analyze(id, fen, limit, tx)?;
        }
        "stop" => {
            info!("Stopping analysis: {}", msg.id);
            engine.stop();
        }
        "validateMove" => {
            info!("🔍 Validating move: {} on {}", msg.uci_move, msg.fen);
            let is_legal = engine.is_move_legal(&msg.fen, &msg.uci_move);
            info!("✓ Move valid: {}", is_legal);
            let response = ServerMessage {
                msg_type: "moveValidation".to_string(),
                id: msg.id,
                payload: serde_json::json!({ "valid": is_legal }),
            };
            tx.send(response)?;
        }
        "makeMove" => {
            info!("♟️ Making move: {} on {}", msg.uci_move, msg.fen);
            match engine.make_move(&msg.fen, &msg.uci_move) {
                Ok(new_fen) => {
                    info!("✓ Move applied, new FEN: {}", new_fen);
                    let response = ServerMessage {
                        msg_type: "newPosition".to_string(),
                        id: msg.id,
                        payload: serde_json::json!({ "fen": new_fen }),
                    };
                    tx.send(response)?;
                }
                Err(e) => {
                    error!("✗ Move failed: {}", e);
                    let response = ServerMessage {
                        msg_type: "error".to_string(),
                        id: msg.id,
                        payload: serde_json::json!({ "error": e.to_string() }),
                    };
                    tx.send(response)?;
                }
            }
        }
        "legalMoves" => {
            info!("📋 Getting legal moves for: {}", msg.fen);
            let moves = engine.legal_moves(&msg.fen);
            info!("✓ Found {} legal moves", moves.len());
            let response = ServerMessage {
                msg_type: "legalMoves".to_string(),
                id: msg.id,
                payload: serde_json::json!({ "moves": moves }),
            };
            tx.send(response)?;
        }
        "gameStatus" => {
            info!("🎮 Checking game status for: {}", msg.fen);
            let (is_over, status) = engine.is_game_over(&msg.fen);
            info!("✓ Game over: {}, status: {:?}", is_over, status);
            let response = ServerMessage {
                msg_type: "gameStatus".to_string(),
                id: msg.id,
                payload: serde_json::json!({
                    "isOver": is_over,
                    "status": status,
                }),
            };
            tx.send(response)?;
        }
        "ping" => {
            debug!("Received ping, sending pong");
            let response = ServerMessage {
                msg_type: "pong".to_string(),
                id: msg.id,
                payload: serde_json::json!({}),
            };
            tx.send(response)?;
        }
        _ => {
            warn!("Unknown message type: {}", msg.msg_type);
        }
    }

    Ok(())
}
