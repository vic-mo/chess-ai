//! Simple opening book for avoiding bad openings and following sound principles
//!
//! This is a minimal opening book that stores (position_hash, best_move) pairs
//! to guide the engine in the opening phase.

use crate::board::Board;
use crate::r#move::Move;
use std::collections::HashMap;

pub struct OpeningBook {
    positions: HashMap<u64, Vec<Move>>,
}

impl OpeningBook {
    pub fn new() -> Self {
        let mut book = OpeningBook {
            positions: HashMap::new(),
        };
        book.populate_basic_openings();
        book
    }

    /// Add a position and its book moves
    fn add_position(&mut self, fen: &str, moves: Vec<&str>) {
        use crate::io::parse_fen;
        use crate::movegen::generate_moves;

        let board = match parse_fen(fen) {
            Ok(b) => b,
            Err(_) => return,
        };

        let legal_moves = generate_moves(&board);
        let mut book_moves = Vec::new();

        for move_str in moves {
            // Find the move in legal moves
            for m in legal_moves.iter() {
                let from_str = m.from().to_string();
                let to_str = m.to().to_string();
                let move_uci = format!("{}{}", from_str, to_str);

                if move_uci == *move_str {
                    book_moves.push(*m);
                    break;
                }
            }
        }

        if !book_moves.is_empty() {
            self.positions.insert(board.hash(), book_moves);
        }
    }

    /// Populate with basic sound opening principles
    fn populate_basic_openings(&mut self) {
        // Starting position - classical opening moves
        self.add_position(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            vec!["e2e4", "d2d4", "g1f3", "c2c4"],
        );

        // After 1.e4 - good responses
        self.add_position(
            "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1",
            vec!["e7e5", "c7c5", "e7e6", "c7c6"],
        );

        // After 1.d4 - good responses
        self.add_position(
            "rnbqkbnr/pppppppp/8/8/3P4/8/PPP1PPPP/RNBQKBNR b KQkq d3 0 1",
            vec!["d7d5", "g8f6", "e7e6"],
        );

        // After 1.Nf3 - good responses
        self.add_position(
            "rnbqkbnr/pppppppp/8/8/8/5N2/PPPPPPPP/RNBQKB1R b KQkq - 1 1",
            vec!["d7d5", "g8f6", "c7c5"],
        );

        // After 1.Nf3 d5 - AVOID Ng5!
        self.add_position(
            "rnbqkbnr/ppp1pppp/8/3p4/8/5N2/PPPPPPPP/RNBQKB1R w KQkq d6 0 2",
            vec!["d2d4", "c2c4", "g2g3", "b1c3"],
        );

        // After 1.e4 e5 2.Nf3 - develop pieces
        self.add_position(
            "rnbqkbnr/pppp1ppp/8/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2",
            vec!["b8c6", "g8f6"],
        );

        // After 1.e4 e5 2.Nf3 Nc6 - classical development
        self.add_position(
            "r1bqkbnr/pppp1ppp/2n5/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 2 3",
            vec!["f1b5", "f1c4", "d2d4"],
        );

        // After 1.d4 d5 2.c4 - Queen's Gambit
        self.add_position(
            "rnbqkbnr/ppp1pppp/8/3p4/2PP4/8/PP2PPPP/RNBQKBNR b KQkq c3 0 2",
            vec!["e7e6", "c7c6", "d5c4"],
        );

        // After 1.e4 c5 - Sicilian Defense
        self.add_position(
            "rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq c6 0 2",
            vec!["g1f3", "b1c3"],
        );

        // === EXPANDED OPENING BOOK ===

        // === KING'S PAWN OPENINGS (1.e4) ===

        // Ruy Lopez: 1.e4 e5 2.Nf3 Nc6 3.Bb5
        self.add_position(
            "r1bqkbnr/pppp1ppp/2n5/1B2p3/4P3/5N2/PPPP1PPP/RNBQK2R b KQkq - 3 3",
            vec!["a7a6", "g8f6", "f8c5"],
        );

        // Italian Game: 1.e4 e5 2.Nf3 Nc6 3.Bc4
        self.add_position(
            "r1bqkbnr/pppp1ppp/2n5/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R b KQkq - 3 3",
            vec!["f8c5", "g8f6", "f8e7"],
        );

        // Scotch Game: 1.e4 e5 2.Nf3 Nc6 3.d4
        self.add_position(
            "r1bqkbnr/pppp1ppp/2n5/4p3/3PP3/5N2/PPP2PPP/RNBQKB1R b KQkq d3 0 3",
            vec!["e5d4", "g8f6"],
        );

        // French Defense: 1.e4 e6 2.d4
        self.add_position(
            "rnbqkbnr/pppp1ppp/4p3/8/3PP3/8/PPP2PPP/RNBQKBNR b KQkq d3 0 2",
            vec!["d7d5"],
        );

        // French: 1.e4 e6 2.d4 d5 3.Nc3
        self.add_position(
            "rnbqkbnr/ppp2ppp/4p3/3p4/3PP3/2N5/PPP2PPP/R1BQKBNR b KQkq - 1 3",
            vec!["f8b4", "g8f6", "d5e4"],
        );

        // Caro-Kann: 1.e4 c6 2.d4
        self.add_position(
            "rnbqkbnr/pp1ppppp/2p5/8/3PP3/8/PPP2PPP/RNBQKBNR b KQkq d3 0 2",
            vec!["d7d5"],
        );

        // Caro-Kann: 1.e4 c6 2.d4 d5 3.Nc3
        self.add_position(
            "rnbqkbnr/pp2pppp/2p5/3p4/3PP3/2N5/PPP2PPP/R1BQKBNR b KQkq - 1 3",
            vec!["d5e4", "g8f6"],
        );

        // Sicilian: 1.e4 c5 2.Nf3 d6
        self.add_position(
            "rnbqkbnr/pp2pppp/3p4/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 0 3",
            vec!["d2d4", "f1c4"],
        );

        // Sicilian: 1.e4 c5 2.Nf3 d6 3.d4 cxd4
        self.add_position(
            "rnbqkbnr/pp2pppp/3p4/8/3pP3/5N2/PPP2PPP/RNBQKB1R w KQkq - 0 4",
            vec!["f3d4", "d1d4"],
        );

        // Sicilian: 1.e4 c5 2.Nf3 Nc6
        self.add_position(
            "r1bqkbnr/pp1ppppp/2n5/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 2 3",
            vec!["d2d4", "f1b5"],
        );

        // === QUEEN'S PAWN OPENINGS (1.d4) ===

        // Queen's Gambit: 1.d4 d5 2.c4 e6
        self.add_position(
            "rnbqkbnr/ppp2ppp/4p3/3p4/2PP4/8/PP2PPPP/RNBQKBNR w KQkq - 0 3",
            vec!["b1c3", "g1f3"],
        );

        // Queen's Gambit: 1.d4 d5 2.c4 c6
        self.add_position(
            "rnbqkbnr/pp2pppp/2p5/3p4/2PP4/8/PP2PPPP/RNBQKBNR w KQkq - 0 3",
            vec!["g1f3", "b1c3"],
        );

        // Queen's Gambit Accepted: 1.d4 d5 2.c4 dxc4
        self.add_position(
            "rnbqkbnr/ppp1pppp/8/8/2pP4/8/PP2PPPP/RNBQKBNR w KQkq - 0 3",
            vec!["g1f3", "e2e3"],
        );

        // King's Indian: 1.d4 Nf6 2.c4
        self.add_position(
            "rnbqkb1r/pppppppp/5n2/8/2PP4/8/PP2PPPP/RNBQKBNR b KQkq c3 0 2",
            vec!["g7g6", "e7e6", "d7d5"],
        );

        // King's Indian: 1.d4 Nf6 2.c4 g6 3.Nc3
        self.add_position(
            "rnbqkb1r/pppppp1p/5np1/8/2PP4/2N5/PP2PPPP/R1BQKBNR b KQkq - 1 3",
            vec!["f8g7", "d7d5"],
        );

        // Nimzo-Indian: 1.d4 Nf6 2.c4 e6 3.Nc3 Bb4
        self.add_position(
            "rnbqk2r/pppp1ppp/4pn2/8/1bPP4/2N5/PP2PPPP/R1BQKBNR w KQkq - 2 4",
            vec!["e2e3", "d1c2", "g1f3"],
        );

        // Grunfeld: 1.d4 Nf6 2.c4 g6 3.Nc3 d5
        self.add_position(
            "rnbqkb1r/ppp1pp1p/5np1/3p4/2PP4/2N5/PP2PPPP/R1BQKBNR w KQkq d6 0 4",
            vec!["c4d5", "g1f3"],
        );

        // London System: 1.d4 d5 2.Bf4
        self.add_position(
            "rnbqkbnr/ppp1pppp/8/3p4/3P1B2/8/PPP1PPPP/RN1QKBNR b KQkq - 1 2",
            vec!["g8f6", "c7c5", "e7e6"],
        );

        // London: 1.d4 Nf6 2.Bf4
        self.add_position(
            "rnbqkb1r/pppppppp/5n2/8/3P1B2/8/PPP1PPPP/RN1QKBNR b KQkq - 1 2",
            vec!["c7c5", "e7e6", "d7d5"],
        );

        // === FLANK OPENINGS ===

        // English: 1.c4 e5
        self.add_position(
            "rnbqkbnr/pppp1ppp/8/4p3/2P5/8/PP1PPPPP/RNBQKBNR w KQkq e6 0 2",
            vec!["b1c3", "g1f3"],
        );

        // English: 1.c4 c5
        self.add_position(
            "rnbqkbnr/pp1ppppp/8/2p5/2P5/8/PP1PPPPP/RNBQKBNR w KQkq c6 0 2",
            vec!["g1f3", "b1c3", "g2g3"],
        );

        // English: 1.c4 Nf6
        self.add_position(
            "rnbqkb1r/pppppppp/5n2/8/2P5/8/PP1PPPPP/RNBQKBNR w KQkq - 1 2",
            vec!["g1f3", "b1c3"],
        );

        // Reti: 1.Nf3 d5 2.c4
        self.add_position(
            "rnbqkbnr/ppp1pppp/8/3p4/2P5/5N2/PP1PPPPP/RNBQKB1R b KQkq c3 0 2",
            vec!["d5c4", "e7e6", "g8f6"],
        );

        // Reti: 1.Nf3 Nf6 2.c4
        self.add_position(
            "rnbqkb1r/pppppppp/5n2/8/2P5/5N2/PP1PPPPP/RNBQKB1R b KQkq c3 0 2",
            vec!["e7e6", "g7g6", "c7c5"],
        );

        // === ADDITIONAL KEY POSITIONS ===

        // After 1.e4 e5 2.Nf3 Nf6 (Petrov)
        self.add_position(
            "rnbqkb1r/pppp1ppp/5n2/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 2 3",
            vec!["f3e5", "d2d4"],
        );

        // After 1.d4 d5 2.c4 e6 3.Nc3 Nf6
        self.add_position(
            "rnbqkb1r/ppp2ppp/4pn2/3p4/2PP4/2N5/PP2PPPP/R1BQKBNR w KQkq - 1 4",
            vec!["g1f3", "c1g5"],
        );

        // After 1.e4 e5 2.Nf3 Nc6 3.Bb5 a6 4.Ba4
        self.add_position(
            "r1bqkbnr/1ppp1ppp/p1n5/4p3/B3P3/5N2/PPPP1PPP/RNBQK2R b KQkq - 4 4",
            vec!["g8f6", "f8c5"],
        );

        // After 1.e4 e5 2.Nf3 Nc6 3.Bc4 Bc5 (Italian)
        self.add_position(
            "r1bqk1nr/pppp1ppp/2n5/2b1p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4",
            vec!["c2c3", "d2d3", "e1g1"],
        );
    }

    /// Probe the book for a move
    pub fn probe(&self, board: &Board) -> Option<Move> {
        if let Some(moves) = self.positions.get(&board.hash()) {
            if !moves.is_empty() {
                // Return first move (could randomize later)
                return Some(moves[0]);
            }
        }
        None
    }

    /// Check if a position is in the book
    pub fn contains(&self, board: &Board) -> bool {
        self.positions.contains_key(&board.hash())
    }

    /// Get number of positions in book
    pub fn size(&self) -> usize {
        self.positions.len()
    }
}

impl Default for OpeningBook {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::parse_fen;

    #[test]
    fn test_book_creation() {
        let book = OpeningBook::new();
        assert!(book.size() > 0, "Book should have positions");
    }

    #[test]
    fn test_startpos_in_book() {
        let book = OpeningBook::new();
        let board = Board::startpos();
        assert!(book.contains(&board), "Starting position should be in book");
    }

    #[test]
    fn test_probe_startpos() {
        let book = OpeningBook::new();
        let board = Board::startpos();
        let mv = book.probe(&board);
        assert!(mv.is_some(), "Should find a move for starting position");
    }

    #[test]
    fn test_avoid_ng5() {
        let book = OpeningBook::new();
        // After 1.Nf3 d5, should NOT suggest Ng5
        let board = parse_fen("rnbqkbnr/ppp1pppp/8/3p4/8/5N2/PPPPPPPP/RNBQKB1R w KQkq d6 0 2").unwrap();

        if let Some(mv) = book.probe(&board) {
            let from_str = mv.from().to_string();
            let to_str = mv.to().to_string();
            let move_uci = format!("{}{}", from_str, to_str);
            assert_ne!(move_uci, "g1g5", "Book should not suggest Ng5");
        }
    }
}
