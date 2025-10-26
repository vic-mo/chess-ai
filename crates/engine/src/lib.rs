//! # Chess Engine Core
//!
//! A high-performance chess engine core library featuring:
//! - Bitboard representation for efficient board state
//! - Complete move generation for all piece types
//! - Legal move validation with check detection
//! - Zobrist hashing for position tracking
//! - FEN parsing and serialization
//! - 26M+ nodes/second perft performance
//!
//! ## Quick Start
//!
//! ```
//! use engine::board::Board;
//! use engine::io::parse_fen;
//!
//! // Create a board from starting position
//! let board = Board::startpos();
//!
//! // Or parse from FEN
//! let board = parse_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
//!
//! // Generate legal moves
//! let legal_moves = board.generate_legal_moves();
//! println!("Legal moves: {}", legal_moves.len());
//! ```
//!
//! ## Core Modules
//!
//! - [`board`] - Board representation and core logic
//! - [`bitboard`] - Bitboard data structure for efficient square sets
//! - [`square`] - Square representation and coordinate system
//! - [`piece`] - Piece types and colors
//! - [`r#move`] - Move encoding and representation
//! - [`movegen`] - Move generation for all pieces
//! - [`attacks`] - Attack generation and lookup tables
//! - [`zobrist`] - Zobrist hashing for positions
//! - [`io`] - FEN parsing and serialization
//! - [`perft`] - Performance testing and validation

pub mod attacks;
pub mod bitboard;
pub mod board;
pub mod eval;
pub mod io;
#[allow(clippy::module_inception)]
pub mod r#move;
pub mod move_order;
pub mod movegen;
pub mod movelist;
pub mod perft;
pub mod piece;
pub mod search;
pub mod square;
pub mod time;
pub mod tt;
pub mod types;
pub mod uci;
pub mod zobrist;

use board::Board;
use io::{parse_fen, ToFen};
use r#move::Move;
use search::Searcher;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use time::TimeControl;
use types::*;

pub struct EngineImpl {
    pub opts: EngineOptions,
    pub current_fen: String,
    pub current_board: Option<Board>,
    stopped: Arc<AtomicBool>,
    searcher: Searcher,
}

impl Default for EngineImpl {
    fn default() -> Self {
        let opts = EngineOptions {
            hash_size_mb: 64,
            threads: 1,
            contempt: None,
            skill_level: None,
            multi_pv: Some(1),
            use_tablebases: None,
        };
        let tt_size = opts.hash_size_mb as usize;
        let stopped = Arc::new(AtomicBool::new(false));
        Self {
            opts,
            current_fen: "startpos".to_string(),
            current_board: None,
            stopped: Arc::clone(&stopped),
            searcher: Searcher::with_tt_size_and_stop_flag(tt_size, stopped),
        }
    }
}

impl EngineImpl {
    pub fn new_with(opts: EngineOptions) -> Self {
        let tt_size = opts.hash_size_mb as usize;
        let stopped = Arc::new(AtomicBool::new(false));
        Self {
            opts,
            current_fen: "startpos".to_string(),
            current_board: None,
            stopped: Arc::clone(&stopped),
            searcher: Searcher::with_tt_size_and_stop_flag(tt_size, stopped),
        }
    }

    pub fn new_game(&mut self) {
        self.current_fen = "startpos".to_string();
        self.stopped.store(false, Ordering::Relaxed);
    }

    pub fn position(&mut self, fen: &str, _moves: &[String]) {
        self.current_fen = fen.to_string();
        // Handle "startpos" keyword or parse FEN
        self.current_board = if fen == "startpos" {
            Some(Board::startpos())
        } else {
            parse_fen(fen).ok()
        };
        // TODO: apply moves if provided
    }

    pub fn set_option(&mut self, _key: &str, _value: &str) {
        // TODO: parse key/value into opts
    }

    pub fn analyze<F>(&mut self, limit: SearchLimit, mut info_sink: F) -> BestMove
    where
        F: FnMut(SearchInfo),
    {
        self.stopped.store(false, Ordering::Relaxed);

        // Parse board from FEN
        let board = match &self.current_board {
            Some(b) => b.clone(),
            None => {
                // Handle "startpos" keyword or parse FEN
                if self.current_fen == "startpos" {
                    Board::startpos()
                } else {
                    match parse_fen(&self.current_fen) {
                        Ok(b) => b,
                        Err(_e) => {
                            return BestMove {
                                id: String::new(),
                                best: "0000".to_string(), // Invalid move to signal error
                                ponder: None,
                            };
                        }
                    }
                }
            }
        };

        // Convert SearchLimit to (max_depth, TimeControl)
        let (max_depth, time_control) = match limit {
            SearchLimit::Depth { depth } => (depth, TimeControl::Depth { depth }),
            SearchLimit::Nodes { nodes } => (search::MAX_DEPTH, TimeControl::Nodes { nodes }),
            SearchLimit::Time { move_time_ms } => {
                (search::MAX_DEPTH, TimeControl::MoveTime { millis: move_time_ms })
            }
            SearchLimit::Infinite => (search::MAX_DEPTH, TimeControl::Infinite),
        };

        // Call the real search engine with callback
        let result = self.searcher.search_with_limit_callback(
            &board,
            max_depth,
            time_control,
            |mut info| {
                // ID will be set by caller if needed, leave empty here
                info.id = String::new();
                info_sink(info);
            },
        );

        // Convert result to BestMove
        let best_move_str = Self::move_to_string(&result.best_move);
        let ponder_move_str = result.pv.get(1).map(|m| Self::move_to_string(m));

        BestMove {
            id: String::new(), // ID is added by the caller (WASM bridge, server, etc.)
            best: best_move_str,
            ponder: ponder_move_str,
        }
    }

    /// Convert Move to UCI string (e.g., "e2e4", "e7e8q")
    fn move_to_string(mv: &Move) -> String {
        format!("{}", mv)
    }

    pub fn stop(&self) {
        self.stopped.store(true, Ordering::Relaxed);
        self.searcher.stop();
    }

    /// Get a clone of the stop flag for external control.
    pub fn stop_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.stopped)
    }

    /// Validate if a UCI move is legal in the given position
    pub fn is_move_legal(&self, fen: &str, uci_move: &str) -> bool {
        let board = match parse_fen(fen) {
            Ok(b) => b,
            Err(_) => return false,
        };

        let legal_moves = board.generate_legal_moves();
        let result = legal_moves.iter().any(|m| m.to_uci() == uci_move);
        result
    }

    /// Apply a UCI move and return the new FEN
    pub fn make_move(&mut self, fen: &str, uci_move: &str) -> Result<String, String> {
        let mut board = parse_fen(fen).map_err(|e| format!("Invalid FEN: {:?}", e))?;

        let legal_moves = board.generate_legal_moves();
        let mv = legal_moves
            .iter()
            .find(|m| m.to_uci() == uci_move)
            .ok_or_else(|| format!("Illegal move: {}", uci_move))?;

        board.make_move(*mv);
        Ok(board.to_fen())
    }

    /// Get all legal moves for a position as UCI strings
    pub fn legal_moves(&self, fen: &str) -> Vec<String> {
        match parse_fen(fen) {
            Ok(board) => board
                .generate_legal_moves()
                .iter()
                .map(|m| m.to_uci())
                .collect(),
            Err(_) => vec![],
        }
    }

    /// Check if position is game over (checkmate, stalemate)
    pub fn is_game_over(&self, fen: &str) -> (bool, Option<String>) {
        match parse_fen(fen) {
            Ok(board) => {
                let legal_moves = board.generate_legal_moves();
                if legal_moves.len() == 0 {
                    if board.is_in_check() {
                        (true, Some("checkmate".to_string()))
                    } else {
                        (true, Some("stalemate".to_string()))
                    }
                } else {
                    (false, None)
                }
            }
            Err(_) => (false, None),
        }
    }
}
