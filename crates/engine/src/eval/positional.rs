//! Positional evaluation factors beyond material and PST.

use crate::board::Board;
use crate::movegen;
use crate::piece::Color;

/// Mobility bonus per legal move (in centipawns).
const MOBILITY_BONUS: i32 = 2;

/// Evaluate mobility (number of pseudo-legal moves).
///
/// More mobility generally indicates better piece activity and control.
pub fn evaluate_mobility(board: &Board, _color: Color) -> i32 {
    let moves = movegen::generate_moves(board);
    (moves.len() as i32) * MOBILITY_BONUS
}

/// Evaluate all positional factors for a color.
pub fn evaluate_positional(board: &Board, color: Color) -> i32 {
    let mut score = 0;

    // Mobility
    score += evaluate_mobility(board, color);

    score
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::parse_fen;

    #[test]
    fn test_mobility_startpos() {
        let board = Board::startpos();

        let white_mobility = evaluate_mobility(&board, Color::White);
        let black_mobility = evaluate_mobility(&board, Color::Black);

        // Starting position is symmetric
        assert_eq!(white_mobility, black_mobility);
        // Should have 20 legal moves each (8 pawns * 2 + 2 knights * 2)
        assert_eq!(white_mobility, 20 * MOBILITY_BONUS);
    }

    #[test]
    fn test_mobility_more_moves() {
        // Position with more piece activity
        let fen = "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2";
        let board = parse_fen(fen).unwrap();

        let white_mobility = evaluate_mobility(&board, Color::White);

        // Should have more moves than starting position
        // (bishop and queen have some moves now)
        assert!(white_mobility > 20 * MOBILITY_BONUS);
    }
}
