//! Threat detection and evaluation
//!
//! Evaluates tactical threats in a position:
//! - Hanging pieces (undefended)
//! - Pieces attacked by lower value pieces
//! - Weak pieces under attack

use crate::board::Board;
use crate::piece::{Color, PieceType};
use crate::square::Square;
use crate::bitboard::Bitboard;

/// Piece values for threat evaluation (in centipawns)
const PIECE_VALUES: [i32; 6] = [
    100,  // Pawn
    320,  // Knight
    330,  // Bishop
    500,  // Rook
    900,  // Queen
    0,    // King (not used for hanging)
];

/// Get the value of a piece type
#[inline]
fn piece_value(piece_type: PieceType) -> i32 {
    match piece_type {
        PieceType::Pawn => PIECE_VALUES[0],
        PieceType::Knight => PIECE_VALUES[1],
        PieceType::Bishop => PIECE_VALUES[2],
        PieceType::Rook => PIECE_VALUES[3],
        PieceType::Queen => PIECE_VALUES[4],
        PieceType::King => PIECE_VALUES[5],
    }
}

/// Compute all squares attacked by a color
/// This is computed once and reused to avoid repeated is_square_attacked calls
fn compute_attacks(board: &Board, color: Color) -> Bitboard {
    use crate::attacks::{
        bishop_attacks, king_attacks, knight_attacks, pawn_attacks, rook_attacks,
    };

    let mut attacks = Bitboard::EMPTY;
    let occupied = board.occupied();

    // Pawn attacks
    let pawns = board.piece_bb(PieceType::Pawn, color);
    for sq in pawns {
        attacks |= pawn_attacks(sq, color);
    }

    // Knight attacks
    let knights = board.piece_bb(PieceType::Knight, color);
    for sq in knights {
        attacks |= knight_attacks(sq);
    }

    // Bishop, rook, and queen attacks
    let bishops = board.piece_bb(PieceType::Bishop, color);
    let rooks = board.piece_bb(PieceType::Rook, color);
    let queens = board.piece_bb(PieceType::Queen, color);

    // Diagonal attacks (bishops + queens)
    let diagonal_attackers = bishops | queens;
    for sq in diagonal_attackers {
        attacks |= bishop_attacks(sq, occupied);
    }

    // Orthogonal attacks (rooks + queens)
    let orthogonal_attackers = rooks | queens;
    for sq in orthogonal_attackers {
        attacks |= rook_attacks(sq, occupied);
    }

    // King attacks
    let king = board.piece_bb(PieceType::King, color);
    for sq in king {
        attacks |= king_attacks(sq);
    }

    attacks
}

/// Evaluate threats in the position
///
/// Returns (mg_score, eg_score) from white's perspective
pub fn evaluate_threats(board: &Board) -> (i32, i32) {
    // Compute attack maps once for both colors
    let white_attacks = compute_attacks(board, Color::White);
    let black_attacks = compute_attacks(board, Color::Black);

    // Evaluate threats for both sides using cached attacks
    let (white_mg, white_eg) = evaluate_threats_for_side(board, Color::White, &white_attacks, &black_attacks);
    let (black_mg, black_eg) = evaluate_threats_for_side(board, Color::Black, &black_attacks, &white_attacks);

    let mg_score = white_mg - black_mg;
    let eg_score = white_eg - black_eg;

    (mg_score, eg_score)
}

/// Evaluate threats for one side using precomputed attack maps
fn evaluate_threats_for_side(
    board: &Board,
    color: Color,
    our_attacks: &Bitboard,
    enemy_attacks: &Bitboard,
) -> (i32, i32) {
    let mut mg_score = 0;
    let mut eg_score = 0;

    let enemy_color = color.opponent();

    // Check all enemy pieces for threats
    for piece_type in [
        PieceType::Queen,
        PieceType::Rook,
        PieceType::Bishop,
        PieceType::Knight,
        PieceType::Pawn,
    ] {
        let pieces = board.piece_bb(piece_type, enemy_color);

        for sq in pieces {
            let sq_bb = Bitboard::from_square(sq);

            // Check if piece is defended by enemy
            let is_defended = !(*enemy_attacks & sq_bb).is_empty();

            // Check if we attack this piece
            let is_attacked_by_us = !(*our_attacks & sq_bb).is_empty();

            // Hanging piece: attacked by us but not defended
            if is_attacked_by_us && !is_defended {
                let value = piece_value(piece_type);
                mg_score += value / 2; // Half the piece value as bonus
                eg_score += value / 2;
            }

            // Weak piece: attacked by us even if defended
            // (gives bonus for pieces we can attack with lower value pieces)
            if is_attacked_by_us {
                // Simplified: assume we can attack with pawn (lowest value)
                // More accurate would be to find actual lowest attacker
                let attacker_value = piece_value(PieceType::Pawn);
                let piece_val = piece_value(piece_type);

                if attacker_value < piece_val {
                    let bonus = (piece_val - attacker_value) / 8; // Smaller bonus (divided by 8 instead of 4)
                    mg_score += bonus;
                    eg_score += bonus;
                }
            }
        }
    }

    (mg_score, eg_score)
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::parse_fen;

    #[test]
    fn test_hanging_piece() {
        // Position with a hanging knight on e5
        let board = parse_fen("rnbqkb1r/pppp1ppp/8/4n3/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();

        let (mg, eg) = evaluate_threats(&board);

        // White should get bonus for threatening the hanging knight
        assert!(mg > 0, "Should detect hanging knight");
        assert!(eg > 0, "Should detect hanging knight in endgame");
    }

    #[test]
    fn test_defended_piece() {
        // Position with defended pieces
        let board = parse_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();

        let (mg, eg) = evaluate_threats(&board);

        // No threats in starting position
        assert_eq!(mg, 0, "No threats in starting position");
        assert_eq!(eg, 0, "No threats in starting position");
    }

    #[test]
    fn test_attacked_by_pawn() {
        // Position where pawn attacks knight
        let board = parse_fen("rnbqkb1r/pppp1ppp/8/4n3/3P4/8/PPP1PPPP/RNBQKBNR w KQkq - 0 1").unwrap();

        let (mg, eg) = evaluate_threats(&board);

        // White pawn threatens black knight (worth more)
        assert!(mg > 0, "Pawn should threaten knight");
        assert!(eg > 0, "Pawn should threaten knight in endgame");
    }
}
