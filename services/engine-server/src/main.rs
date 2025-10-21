use std::{collections::HashMap, sync::Arc, time::Duration};

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use engine::{
    types::{BestMove, EngineOptions, Score, SearchInfo},
    EngineImpl,
};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    sessions: Arc<Mutex<HashMap<String, broadcast::Sender<String>>>>,
}

#[derive(Deserialize)]
struct AnalyzeRequestBody {
    id: Option<String>,
    fen: String,
    #[allow(dead_code)]
    limit: Option<serde_json::Value>,
}

#[derive(Serialize)]
struct AnalyzeResponse {
    id: String,
}

#[tokio::main]
async fn main() {
    let state = AppState {
        sessions: Arc::new(Mutex::new(HashMap::new())),
    };
    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/analyze", post(start_analyze))
        .route("/stop", post(stop_analyze))
        .route("/streams/:id", get(ws_stream))
        .with_state(state);
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("engine-server listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn start_analyze(
    State(state): State<AppState>,
    Json(body): Json<AnalyzeRequestBody>,
) -> impl IntoResponse {
    let id = body.id.unwrap_or_else(|| Uuid::new_v4().to_string());
    let (tx, _rx) = broadcast::channel::<String>(16);
    state.sessions.lock().insert(id.clone(), tx.clone());

    // Spawn a task that simulates iterative deepening and sends SearchInfo JSON lines
    let id_for_task = id.clone();
    tokio::spawn(async move {
        let mut eng = EngineImpl::new_with(EngineOptions {
            hash_size_mb: 64,
            threads: 1,
            contempt: None,
            skill_level: None,
            multi_pv: Some(1),
            use_tablebases: None,
        });
        eng.position(&body.fen, &[]);
        for depth in 1..=6u32 {
            let info = SearchInfo {
                id: id_for_task.clone(),
                depth,
                seldepth: Some(depth + 1),
                nodes: depth as u64 * 10_000,
                nps: 1_000_000,
                time_ms: depth as u64 * 80,
                score: Score::Cp {
                    value: depth as i32 * 12,
                },
                pv: vec!["e2e4".into(), "e7e5".into()],
                hashfull: Some(20 * depth),
                tb_hits: None,
            };
            let line = serde_json::to_string(&serde_json::json!({
                "type": "searchInfo",
                "payload": info
            }))
            .unwrap();
            let _ = tx.send(line);
            tokio::time::sleep(Duration::from_millis(150)).await;
        }
        let best = BestMove {
            id: id_for_task.clone(),
            best: "e2e4".into(),
            ponder: Some("e7e5".into()),
        };
        let line = serde_json::to_string(&serde_json::json!({
            "type": "bestMove",
            "payload": best
        }))
        .unwrap();
        let _ = tx.send(line);
    });

    Json(AnalyzeResponse { id })
}

#[derive(Deserialize)]
struct StopBody {
    id: String,
}

async fn stop_analyze(
    State(state): State<AppState>,
    Json(body): Json<StopBody>,
) -> impl IntoResponse {
    let mut sessions = state.sessions.lock();
    if let Some(tx) = sessions.remove(&body.id) {
        let line = serde_json::to_string(&serde_json::json!({
            "type": "error",
            "payload": {
                "id": body.id,
                "message": "stopped"
            }
        }))
        .unwrap();
        let _ = tx.send(line);
    }
    "ok"
}

async fn ws_stream(
    State(state): State<AppState>,
    Path(id): Path<String>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws(state, id, socket))
}

async fn handle_ws(state: AppState, id: String, mut socket: WebSocket) {
    let rx = {
        let sessions = state.sessions.lock();
        sessions.get(&id).map(|tx| tx.subscribe())
    };

    let mut rx = match rx {
        Some(rx) => rx,
        None => {
            let _ = socket
                .send(Message::Text(
                    r#"{"type":"error","payload":{"id":"none","message":"invalid id"}}"#.into(),
                ))
                .await;
            return;
        }
    };

    // Forward broadcast to WS
    while let Ok(line) = rx.recv().await {
        if socket.send(Message::Text(line)).await.is_err() {
            break;
        }
    }
}
