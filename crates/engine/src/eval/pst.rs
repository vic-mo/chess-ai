//! Piece-Square Tables (PST) for positional evaluation.
//!
//! PSTs provide bonuses/penalties for piece placement on specific squares.
//! Separate tables for middlegame and endgame.

use crate::board::Board;
use crate::piece::{Color, PieceType};
use crate::square::Square;

/// Piece-square tables for all piece types.
///
/// Tables are from White's perspective (rank 0 = rank 1, rank 7 = rank 8).
/// For Black, we flip the rank (7 - rank).
#[derive(Debug, Clone)]
pub struct PieceSquareTables {
    /// Middlegame tables [piece_type][square]
    pub mg_tables: [[i32; 64]; 6],
    /// Endgame tables [piece_type][square]
    pub eg_tables: [[i32; 64]; 6],
}

impl PieceSquareTables {
    /// Evaluate position using piece-square tables.
    pub fn evaluate_position(&self, board: &Board, color: Color) -> i32 {
        let mut score = 0;
        let is_eg = crate::eval::material::is_endgame(board);

        for piece_type in PieceType::all() {
            let pieces = board.piece_bb(piece_type, color);

            for sq in pieces {
                let table_sq = if color == Color::White {
                    sq.index() as usize
                } else {
                    // Flip rank for black
                    Square::from_coords(sq.file(), 7 - sq.rank()).index() as usize
                };

                if is_eg {
                    score += self.eg_tables[piece_type.index()][table_sq];
                } else {
                    score += self.mg_tables[piece_type.index()][table_sq];
                }
            }
        }

        score
    }
}

impl Default for PieceSquareTables {
    fn default() -> Self {
        Self {
            mg_tables: [
                // Pawn middlegame
                [
                    0, 0, 0, 0, 0, 0, 0, 0, // Rank 1
                    5, 10, 10, -20, -20, 10, 10, 5, // Rank 2
                    5, -5, -10, 0, 0, -10, -5, 5, // Rank 3
                    0, 0, 0, 20, 20, 0, 0, 0, // Rank 4
                    5, 5, 10, 25, 25, 10, 5, 5, // Rank 5
                    10, 10, 20, 30, 30, 20, 10, 10, // Rank 6
                    50, 50, 50, 50, 50, 50, 50, 50, // Rank 7
                    0, 0, 0, 0, 0, 0, 0, 0, // Rank 8
                ],
                // Knight middlegame
                [
                    -50, -40, -30, -30, -30, -30, -40, -50, -40, -20, 0, 5, 5, 0, -20, -40, -30, 5,
                    10, 15, 15, 10, 5, -30, -30, 0, 15, 20, 20, 15, 0, -30, -30, 5, 15, 20, 20, 15,
                    5, -30, -30, 0, 10, 15, 15, 10, 0, -30, -40, -20, 0, 0, 0, 0, -20, -40, -50,
                    -40, -30, -30, -30, -30, -40, -50,
                ],
                // Bishop middlegame
                [
                    -20, -10, -10, -10, -10, -10, -10, -20, -10, 5, 0, 0, 0, 0, 5, -10, -10, 10,
                    10, 10, 10, 10, 10, -10, -10, 0, 10, 10, 10, 10, 0, -10, -10, 5, 5, 10, 10, 5,
                    5, -10, -10, 0, 5, 10, 10, 5, 0, -10, -10, 0, 0, 0, 0, 0, 0, -10, -20, -10,
                    -10, -10, -10, -10, -10, -20,
                ],
                // Rook middlegame
                [
                    0, 0, 0, 5, 5, 0, 0, 0, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5,
                    0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, 5,
                    10, 10, 10, 10, 10, 10, 5, 0, 0, 0, 0, 0, 0, 0, 0,
                ],
                // Queen middlegame
                [
                    -20, -10, -10, -5, -5, -10, -10, -20, -10, 0, 5, 0, 0, 0, 0, -10, -10, 5, 5, 5,
                    5, 5, 0, -10, 0, 0, 5, 5, 5, 5, 0, -5, -5, 0, 5, 5, 5, 5, 0, -5, -10, 0, 5, 5,
                    5, 5, 0, -10, -10, 0, 0, 0, 0, 0, 0, -10, -20, -10, -10, -5, -5, -10, -10, -20,
                ],
                // King middlegame (focus on safety)
                [
                    20, 30, 10, 0, 0, 10, 30, 20, 20, 20, 0, 0, 0, 0, 20, 20, -10, -20, -20, -20,
                    -20, -20, -20, -10, -20, -30, -30, -40, -40, -30, -30, -20, -30, -40, -40, -50,
                    -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30, -30, -40, -40, -50,
                    -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30,
                ],
            ],
            eg_tables: [
                // Pawn endgame (passed pawns more valuable)
                [
                    0, 0, 0, 0, 0, 0, 0, 0, // Rank 1
                    10, 10, 10, 10, 10, 10, 10, 10, // Rank 2
                    20, 20, 20, 20, 20, 20, 20, 20, // Rank 3
                    30, 30, 30, 30, 30, 30, 30, 30, // Rank 4
                    40, 40, 40, 40, 40, 40, 40, 40, // Rank 5
                    50, 50, 50, 50, 50, 50, 50, 50, // Rank 6
                    70, 70, 70, 70, 70, 70, 70, 70, // Rank 7
                    0, 0, 0, 0, 0, 0, 0, 0, // Rank 8
                ],
                // Knight endgame (less valuable)
                [
                    -50, -40, -30, -30, -30, -30, -40, -50, -40, -20, 0, 0, 0, 0, -20, -40, -30, 0,
                    10, 15, 15, 10, 0, -30, -30, 5, 15, 20, 20, 15, 5, -30, -30, 0, 15, 20, 20, 15,
                    0, -30, -30, 5, 10, 15, 15, 10, 5, -30, -40, -20, 0, 5, 5, 0, -20, -40, -50,
                    -40, -30, -30, -30, -30, -40, -50,
                ],
                // Bishop endgame
                [
                    -20, -10, -10, -10, -10, -10, -10, -20, -10, 0, 0, 0, 0, 0, 0, -10, -10, 0, 5,
                    10, 10, 5, 0, -10, -10, 0, 10, 10, 10, 10, 0, -10, -10, 0, 10, 10, 10, 10, 0,
                    -10, -10, 0, 5, 10, 10, 5, 0, -10, -10, 0, 0, 0, 0, 0, 0, -10, -20, -10, -10,
                    -10, -10, -10, -10, -20,
                ],
                // Rook endgame
                [
                    0, 0, 0, 0, 0, 0, 0, 0, 5, 10, 10, 10, 10, 10, 10, 5, -5, 0, 0, 0, 0, 0, 0, -5,
                    -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5,
                    -5, 0, 0, 0, 0, 0, 0, -5, 0, 0, 0, 0, 0, 0, 0, 0,
                ],
                // Queen endgame
                [
                    -20, -10, -10, -5, -5, -10, -10, -20, -10, 0, 0, 0, 0, 0, 0, -10, -10, 0, 5, 5,
                    5, 5, 0, -10, -5, 0, 5, 5, 5, 5, 0, -5, 0, 0, 5, 5, 5, 5, 0, 0, -10, 5, 5, 5,
                    5, 5, 0, -10, -10, 0, 5, 0, 0, 0, 0, -10, -20, -10, -10, -5, -5, -10, -10, -20,
                ],
                // King endgame (active king)
                [
                    -50, -30, -30, -30, -30, -30, -30, -50, -30, -30, 0, 0, 0, 0, -30, -30, -30,
                    -10, 20, 30, 30, 20, -10, -30, -30, -10, 30, 40, 40, 30, -10, -30, -30, -10,
                    30, 40, 40, 30, -10, -30, -30, -10, 20, 30, 30, 20, -10, -30, -30, -20, -10, 0,
                    0, -10, -20, -30, -50, -40, -30, -20, -20, -30, -40, -50,
                ],
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::parse_fen;

    #[test]
    fn test_pst_startpos() {
        let board = Board::startpos();
        let pst = PieceSquareTables::default();

        let white_score = pst.evaluate_position(&board, Color::White);
        let black_score = pst.evaluate_position(&board, Color::Black);

        // Starting position is symmetric, so scores should be equal
        assert_eq!(
            white_score, black_score,
            "PST should be symmetric for starting position"
        );
    }

    #[test]
    fn test_pst_central_pawn() {
        // Pawn on e4 should have bonus
        let fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1";
        let board = parse_fen(fen).unwrap();
        let pst = PieceSquareTables::default();

        let white_score = pst.evaluate_position(&board, Color::White);
        let black_score = pst.evaluate_position(&board, Color::Black);

        // White should have slightly better PST score due to central pawn
        assert!(
            white_score > black_score,
            "White central pawn should have PST bonus"
        );
    }

    #[test]
    fn test_pst_color_flip() {
        // Test that PST correctly flips for black pieces
        let pst = PieceSquareTables::default();

        // White pawn on e4 (rank 3, file 4)
        let sq_white = Square::E4;
        let idx_white = sq_white.index();

        // Black pawn on e5 (rank 4, file 4)
        let sq_black = Square::E5;
        // Black uses flipped rank
        let idx_black = Square::from_coords(sq_black.file(), 7 - sq_black.rank()).index();

        // Should have similar values (mirror positions)
        let white_value = pst.mg_tables[PieceType::Pawn.index()][idx_white as usize];
        let black_value = pst.mg_tables[PieceType::Pawn.index()][idx_black as usize];

        assert_eq!(
            white_value, black_value,
            "Mirrored positions should have same PST value"
        );
    }
}
