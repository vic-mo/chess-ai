use anyhow::Result;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tracing::{debug, info};

use engine::EngineImpl;
use engine::types::{BestMove, EngineOptions, SearchInfo, SearchLimit};

use crate::connection::ServerMessage;

pub struct EngineManager {
    engine: Arc<Mutex<EngineImpl>>,
    stop_flag: Arc<AtomicBool>,
    _tx: mpsc::UnboundedSender<ServerMessage>,
    current_id: Option<String>,
}

impl EngineManager {
    pub fn new(tx: mpsc::UnboundedSender<ServerMessage>) -> Self {
        // Create engine with 16MB transposition table
        let opts = EngineOptions {
            hash_size_mb: 16,
            threads: 1,
            contempt: None,
            skill_level: None,
            multi_pv: None,
            use_tablebases: None,
        };
        let engine_impl = EngineImpl::new_with(opts);
        let stop_flag = engine_impl.stop_flag();
        let engine = Arc::new(Mutex::new(engine_impl));

        Self {
            engine,
            stop_flag,
            _tx: tx,
            current_id: None,
        }
    }

    pub fn analyze(
        &mut self,
        id: String,
        fen: String,
        limit: SearchLimit,
        tx: mpsc::UnboundedSender<ServerMessage>,
    ) -> Result<()> {
        // Store current analysis ID
        self.current_id = Some(id.clone());

        // Clone Arc for the thread
        let engine = Arc::clone(&self.engine);
        let callback_id = id.clone();
        let callback_tx = tx.clone();

        // Start analysis in background thread
        std::thread::spawn(move || {
            use tracing::info;

            info!("Analysis thread started for id: {}", callback_id);

            // Lock engine and set position
            {
                let mut eng = engine.lock().unwrap();
                eng.position(&fen, &[]);
                info!("Position set for id: {}", callback_id);
            }

            // Analyze with callback
            info!("Starting analysis for id: {}", callback_id);
            let mut best: BestMove = {
                let mut eng = engine.lock().unwrap();
                eng.analyze(limit, |info: SearchInfo| {
                    // Send SearchInfo to WebSocket
                    let msg = ServerMessage {
                        msg_type: "searchInfo".to_string(),
                        id: callback_id.clone(),
                        payload: serde_json::to_value(&info).unwrap(),
                    };

                    if let Err(e) = callback_tx.send(msg) {
                        debug!("Failed to send SearchInfo (client disconnected?): {}", e);
                    } else {
                        info!("Sent SearchInfo depth {} for id: {}", info.depth, callback_id);
                    }
                })
            };

            info!("Analysis complete for id: {}, best move: {}", callback_id, best.best);

            // Set the id on the BestMove
            best.id = callback_id.clone();

            // Send BestMove to WebSocket
            let msg = ServerMessage {
                msg_type: "bestMove".to_string(),
                id: callback_id.clone(),
                payload: serde_json::to_value(&best).unwrap(),
            };

            if let Err(e) = callback_tx.send(msg) {
                debug!("Failed to send BestMove (client disconnected?): {}", e);
            } else {
                info!("Sent BestMove for id: {}", callback_id);
            }
        });

        Ok(())
    }

    pub fn stop(&mut self) {
        // Set stop flag directly without locking (non-blocking)
        self.stop_flag.store(true, Ordering::Relaxed);
        self.current_id = None;
    }

    /// Validate if a UCI move is legal in the given position
    pub fn is_move_legal(&self, fen: &str, uci_move: &str) -> bool {
        self.engine.lock().unwrap().is_move_legal(fen, uci_move)
    }

    /// Apply a UCI move and return the new FEN
    pub fn make_move(&self, fen: &str, uci_move: &str) -> Result<String> {
        self.engine
            .lock()
            .unwrap()
            .make_move(fen, uci_move)
            .map_err(|e| anyhow::anyhow!(e))
    }

    /// Get all legal moves for a position as UCI strings
    pub fn legal_moves(&self, fen: &str) -> Vec<String> {
        self.engine.lock().unwrap().legal_moves(fen)
    }

    /// Check if position is game over (checkmate, stalemate)
    pub fn is_game_over(&self, fen: &str) -> (bool, Option<String>) {
        self.engine.lock().unwrap().is_game_over(fen)
    }
}
