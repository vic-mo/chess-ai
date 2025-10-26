//! Material evaluation - counts piece values.

use crate::board::Board;
use crate::piece::{Color, PieceType};

/// Piece values in centipawns (1 pawn = 100).
///
/// Values based on standard piece values used in most engines.
pub const PIECE_VALUES: [i32; 6] = [
    100,    // Pawn
    320,    // Knight
    330,    // Bishop
    500,    // Rook
    900,    // Queen
    20_000, // King (effectively infinite)
];

/// Get the value of a piece type in centipawns.
#[inline]
pub fn piece_value(piece_type: PieceType) -> i32 {
    PIECE_VALUES[piece_type.index()]
}

/// Evaluate material for a given color.
///
/// Sums up the values of all pieces for the given color.
pub fn evaluate_material(board: &Board, color: Color) -> i32 {
    let mut score = 0;

    // Iterate through all piece types
    for piece_type in PieceType::all() {
        let bitboard = board.piece_bb(piece_type, color);
        let count = bitboard.count() as i32;
        score += count * piece_value(piece_type);
    }

    score
}

/// Count total material on the board (for both sides).
pub fn total_material(board: &Board) -> i32 {
    evaluate_material(board, Color::White) + evaluate_material(board, Color::Black)
}

/// Determine if we're in the endgame based on material.
///
/// Endgame detection: both queens traded OR material low enough.
pub fn is_endgame(board: &Board) -> bool {
    // No queens on board
    let queens = board.piece_bb(PieceType::Queen, Color::White)
        | board.piece_bb(PieceType::Queen, Color::Black);
    let no_queens = queens.is_empty();

    if no_queens {
        return true;
    }

    // Low material (less than 2 rooks + 2 minors per side average)
    let total = total_material(board);
    let avg_per_side = total / 2;

    // Rough threshold: less than 2R + 2N = 2*500 + 2*320 = 1640 per side
    avg_per_side < 1_700
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::parse_fen;

    #[test]
    fn test_piece_values() {
        assert_eq!(piece_value(PieceType::Pawn), 100);
        assert_eq!(piece_value(PieceType::Knight), 320);
        assert_eq!(piece_value(PieceType::Bishop), 330);
        assert_eq!(piece_value(PieceType::Rook), 500);
        assert_eq!(piece_value(PieceType::Queen), 900);
        assert_eq!(piece_value(PieceType::King), 20_000);
    }

    #[test]
    fn test_startpos_material() {
        let board = Board::startpos();
        let white_mat = evaluate_material(&board, Color::White);
        let black_mat = evaluate_material(&board, Color::Black);

        // Each side: 8P + 2N + 2B + 2R + Q + K
        // = 8*100 + 2*320 + 2*330 + 2*500 + 900 + 20000
        // = 800 + 640 + 660 + 1000 + 900 + 20000 = 24000
        assert_eq!(white_mat, 24_000);
        assert_eq!(black_mat, 24_000);
    }

    #[test]
    fn test_material_imbalance() {
        // White missing a rook
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBN1 w KQkq - 0 1";
        let board = parse_fen(fen).unwrap();

        let white_mat = evaluate_material(&board, Color::White);
        let black_mat = evaluate_material(&board, Color::Black);

        assert_eq!(black_mat - white_mat, 500, "Black should be up a rook");
    }

    #[test]
    fn test_is_endgame() {
        // Starting position - not endgame
        let board = Board::startpos();
        assert!(!is_endgame(&board));

        // Queens traded
        let fen = "rnb1kbnr/pppppppp/8/8/8/8/PPPPPPPP/RNB1KBNR w KQkq - 0 1";
        let board = parse_fen(fen).unwrap();
        assert!(is_endgame(&board));

        // King and pawn endgame
        let fen = "8/4k3/8/8/8/8/4K3/8 w - - 0 1";
        let board = parse_fen(fen).unwrap();
        assert!(is_endgame(&board));
    }

    #[test]
    fn test_total_material() {
        let board = Board::startpos();
        let total = total_material(&board);
        assert_eq!(total, 48_000); // Both sides
    }
}
