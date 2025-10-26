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
pub mod movegen;
pub mod movelist;
pub mod perft;
pub mod piece;
pub mod search;
pub mod square;
pub mod tt;
pub mod types;
pub mod zobrist;

use types::*;

pub struct EngineImpl {
    pub opts: EngineOptions,
    pub current_fen: String,
    stopped: bool,
}

impl Default for EngineImpl {
    fn default() -> Self {
        Self {
            opts: EngineOptions {
                hash_size_mb: 64,
                threads: 1,
                contempt: None,
                skill_level: None,
                multi_pv: Some(1),
                use_tablebases: None,
            },
            current_fen: "startpos".to_string(),
            stopped: false,
        }
    }
}

impl EngineImpl {
    pub fn new_with(opts: EngineOptions) -> Self {
        Self {
            opts,
            ..Default::default()
        }
    }

    pub fn new_game(&mut self) {
        self.current_fen = "startpos".to_string();
        self.stopped = false;
    }

    pub fn position(&mut self, fen: &str, _moves: &[String]) {
        self.current_fen = fen.to_string();
    }

    pub fn set_option(&mut self, _key: &str, _value: &str) {
        // TODO: parse key/value into opts
    }

    pub fn analyze<F>(&mut self, limit: SearchLimit, mut info_sink: F) -> BestMove
    where
        F: FnMut(SearchInfo),
    {
        self.stopped = false;
        // Dummy iterative deepening loop for scaffold
        let mut nodes = 0u64;
        for depth in 1..=match limit {
            SearchLimit::Depth { depth } => depth,
            _ => 6,
        } {
            if self.stopped {
                break;
            }
            nodes += (depth as u64) * 1000;
            let info = SearchInfo {
                id: "scaffold".into(),
                depth,
                seldepth: Some(depth + 2),
                nodes,
                nps: 1_200_000,
                time_ms: depth as u64 * 50,
                score: Score::Cp {
                    value: depth as i32 * 10,
                },
                pv: vec!["e2e4".into(), "e7e5".into()],
                hashfull: Some(10 * depth),
                tb_hits: None,
            };
            info_sink(info);
        }
        BestMove {
            id: "scaffold".into(),
            best: "e2e4".into(),
            ponder: Some("e7e5".into()),
        }
    }

    pub fn stop(&mut self) {
        self.stopped = true;
    }
}
