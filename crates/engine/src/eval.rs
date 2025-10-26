//! Position evaluation for chess positions.
//!
//! Evaluates positions from the current side to move's perspective.
//! Positive scores favor the side to move, negative scores favor the opponent.

mod material;
pub mod phase;
mod positional;
mod pst;

pub use material::*;
pub use phase::*;
pub use positional::*;
pub use pst::*;

use crate::board::Board;
use crate::piece::Color;

/// Main evaluator structure containing evaluation components.
#[derive(Debug, Clone)]
pub struct Evaluator {
    pst: PieceSquareTables,
}

impl Evaluator {
    /// Create a new evaluator with default evaluation parameters.
    pub fn new() -> Self {
        Self {
            pst: PieceSquareTables::default(),
        }
    }

    /// Evaluate a position from the current side to move's perspective.
    ///
    /// Returns a score in centipawns (1 pawn = 100 centipawns).
    /// Positive scores favor the side to move.
    ///
    /// # Example
    /// ```
    /// use engine::board::Board;
    /// use engine::eval::Evaluator;
    ///
    /// let board = Board::startpos();
    /// let evaluator = Evaluator::new();
    /// let score = evaluator.evaluate(&board);
    /// assert_eq!(score, 0); // Starting position is equal
    /// ```
    pub fn evaluate(&self, board: &Board) -> i32 {
        let mut score = 0;

        // Material + PST
        score += self.evaluate_material_and_pst(board);

        // Positional factors
        score += self.evaluate_positional_factors(board);

        // Return score from side to move's perspective
        score
    }

    /// Evaluate positional factors.
    fn evaluate_positional_factors(&self, board: &Board) -> i32 {
        let white_pos = evaluate_positional(board, Color::White);
        let black_pos = evaluate_positional(board, Color::Black);

        let score = white_pos - black_pos;

        // Flip if black to move
        if board.side_to_move() == Color::Black {
            -score
        } else {
            score
        }
    }

    /// Evaluate material and piece-square tables.
    fn evaluate_material_and_pst(&self, board: &Board) -> i32 {
        let mut score = 0;

        // Evaluate for white
        let white_score = evaluate_material(board, Color::White)
            + self.pst.evaluate_position(board, Color::White);

        // Evaluate for black
        let black_score = evaluate_material(board, Color::Black)
            + self.pst.evaluate_position(board, Color::Black);

        // Net score from white's perspective
        score += white_score - black_score;

        // Flip if black to move
        if board.side_to_move() == Color::Black {
            -score
        } else {
            score
        }
    }
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::parse_fen;

    #[test]
    fn test_startpos_equal() {
        let board = Board::startpos();
        let eval = Evaluator::new();
        let score = eval.evaluate(&board);
        assert_eq!(score, 0, "Starting position should be equal");
    }

    #[test]
    fn test_white_advantage() {
        // White is up a rook (black missing h8 rook)
        let fen = "rnbqkbn1/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let board = parse_fen(fen).unwrap();
        let eval = Evaluator::new();
        let score = eval.evaluate(&board);
        assert!(score > 0, "White should have positive score (up material)");
    }

    #[test]
    fn test_black_advantage() {
        // Black is up a rook (white missing h1 rook), black to move
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBN1 b KQkq - 0 1";
        let board = parse_fen(fen).unwrap();
        let eval = Evaluator::new();
        let score = eval.evaluate(&board);
        assert!(
            score > 0,
            "Black to move should have positive score (up material)"
        );
    }

    #[test]
    fn test_symmetry() {
        // Test that evaluation is symmetric for identical positions
        let fen1 = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let fen2 = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1";

        let board1 = parse_fen(fen1).unwrap();
        let board2 = parse_fen(fen2).unwrap();

        let eval = Evaluator::new();
        let score1 = eval.evaluate(&board1);
        let score2 = eval.evaluate(&board2);

        assert_eq!(score1, score2, "Evaluation should be side-to-move relative");
    }
}
