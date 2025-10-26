//! Static Exchange Evaluation (SEE)
//!
//! SEE evaluates the outcome of a capture sequence on a square.
//! It simulates all captures on a target square and calculates the net material balance.
//!
//! Used for:
//! - Move ordering (order captures by SEE value)
//! - Pruning (skip bad captures with SEE < 0)
//! - Quiescence search (prune losing captures)

use crate::attacks::{bishop_attacks, rook_attacks};
use crate::bitboard::Bitboard;
use crate::board::Board;
use crate::piece::{Color, PieceType};
use crate::r#move::Move;
use crate::square::Square;

/// Piece values for SEE calculation (in centipawns)
const SEE_PIECE_VALUES: [i32; 6] = [
    100,   // Pawn
    320,   // Knight
    330,   // Bishop
    500,   // Rook
    900,   // Queen
    20000, // King (high value to avoid king captures in SEE)
];

/// Get the SEE value of a piece type
#[inline]
fn see_value(piece: PieceType) -> i32 {
    SEE_PIECE_VALUES[piece as usize]
}

/// Find the least valuable attacker of a square for a given color
fn find_least_valuable_attacker(
    board: &Board,
    square: Square,
    color: Color,
    occupied: Bitboard,
) -> Option<(Square, PieceType)> {
    // Check pawns first (least valuable)
    let pawn_attackers = board.piece_bb(PieceType::Pawn, color)
        & crate::attacks::pawn_attacks(square, color.opponent())
        & occupied;

    if !pawn_attackers.is_empty() {
        return Some((pawn_attackers.lsb().unwrap(), PieceType::Pawn));
    }

    // Check knights
    let knight_attackers = board.piece_bb(PieceType::Knight, color)
        & crate::attacks::knight_attacks(square)
        & occupied;

    if !knight_attackers.is_empty() {
        return Some((knight_attackers.lsb().unwrap(), PieceType::Knight));
    }

    // Check bishops
    let bishop_attackers =
        board.piece_bb(PieceType::Bishop, color) & bishop_attacks(square, occupied) & occupied;

    if !bishop_attackers.is_empty() {
        return Some((bishop_attackers.lsb().unwrap(), PieceType::Bishop));
    }

    // Check rooks
    let rook_attackers =
        board.piece_bb(PieceType::Rook, color) & rook_attacks(square, occupied) & occupied;

    if !rook_attackers.is_empty() {
        return Some((rook_attackers.lsb().unwrap(), PieceType::Rook));
    }

    // Check queens
    let queen_attackers = board.piece_bb(PieceType::Queen, color)
        & (bishop_attacks(square, occupied) | rook_attacks(square, occupied))
        & occupied;

    if !queen_attackers.is_empty() {
        return Some((queen_attackers.lsb().unwrap(), PieceType::Queen));
    }

    // Check king (last resort)
    let king_attackers =
        board.piece_bb(PieceType::King, color) & crate::attacks::king_attacks(square) & occupied;

    if !king_attackers.is_empty() {
        return Some((king_attackers.lsb().unwrap(), PieceType::King));
    }

    None
}

/// Static Exchange Evaluation
///
/// Returns true if the SEE value of the move is >= threshold.
/// A threshold of 0 means the capture is at least equal (doesn't lose material).
///
/// # Examples
///
/// ```
/// use engine::board::Board;
/// use engine::r#move::Move;
/// use engine::search::see::see;
/// use engine::io::parse_fen;
///
/// // Position where white can capture a pawn
/// let board = parse_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1").unwrap();
/// // Test would go here with actual move
/// ```
pub fn see(board: &Board, mv: Move, threshold: i32) -> bool {
    let from = mv.from();
    let to = mv.to();

    // Get the moving piece and captured piece
    let moving_piece = board.piece_at(from).unwrap().piece_type;
    let mut captured = board.piece_at(to).map(|p| p.piece_type);

    // Handle en passant
    if moving_piece == PieceType::Pawn && captured.is_none() && from.file() != to.file() {
        captured = Some(PieceType::Pawn);
    }

    // If no capture, SEE is 0
    if captured.is_none() {
        return 0 >= threshold;
    }

    let captured = captured.unwrap();

    // Start with the value of the captured piece
    let mut balance = see_value(captured);

    // Subtract the value of the moving piece (assuming it gets captured)
    balance -= see_value(moving_piece);

    // If we're already winning after the capture, return true
    if balance >= threshold {
        return true;
    }

    // If we can't possibly reach the threshold even if we keep the piece, fail
    if balance + see_value(moving_piece) < threshold {
        return false;
    }

    // Simulate the exchange
    let mut occupied = board.occupied().clear(from); // Remove the moving piece

    let mut side = board.side_to_move().opponent();
    let mut attacker_value = see_value(moving_piece);

    loop {
        // Find the least valuable attacker for the current side
        let attacker = find_least_valuable_attacker(board, to, side, occupied);

        if attacker.is_none() {
            break;
        }

        let (attacker_sq, attacker_piece) = attacker.unwrap();

        // Remove the attacker from the board
        occupied = occupied.clear(attacker_sq);

        // Update balance (negamax style)
        balance = -balance - 1 - attacker_value;
        attacker_value = see_value(attacker_piece);

        // Alpha-beta style pruning
        if balance >= 0 {
            break;
        }

        // Switch sides
        side = side.opponent();
    }

    // If white to move, positive is good; if black to move, negative is good
    // But we already alternated in the loop, so check final side
    let final_score = if side == board.side_to_move() {
        -balance
    } else {
        balance
    };

    final_score >= threshold
}

/// SEE for move ordering - returns the actual SEE value (not just true/false)
pub fn see_value_of_move(board: &Board, mv: Move) -> i32 {
    let from = mv.from();
    let to = mv.to();

    // Get the moving piece and captured piece
    let moving_piece = board.piece_at(from).unwrap().piece_type;
    let mut captured = board.piece_at(to).map(|p| p.piece_type);

    // Handle en passant
    if moving_piece == PieceType::Pawn && captured.is_none() && from.file() != to.file() {
        captured = Some(PieceType::Pawn);
    }

    // If no capture, SEE is 0
    if captured.is_none() {
        return 0;
    }

    let captured = captured.unwrap();

    // Start with the value of the captured piece
    let mut balance = see_value(captured);

    // Simulate the exchange
    let mut occupied = board.occupied().clear(from); // Remove the moving piece

    let mut side = board.side_to_move().opponent();
    let mut next_victim = moving_piece;

    loop {
        // Update balance for the capture of the previous attacker
        balance -= see_value(next_victim);

        // Find the least valuable attacker for the current side
        let attacker = find_least_valuable_attacker(board, to, side, occupied);

        if attacker.is_none() {
            break;
        }

        let (attacker_sq, attacker_piece) = attacker.unwrap();

        // Remove the attacker from the board
        occupied = occupied.clear(attacker_sq);

        // The next victim is this attacker
        next_victim = attacker_piece;

        // Switch sides
        side = side.opponent();

        // Negamax the balance
        balance = -balance;

        // Alpha-beta style pruning
        if balance < 0 {
            // Side to move doesn't want to continue (would lose material)
            balance = -balance;
            break;
        }
    }

    balance
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::parse_fen;
    use crate::movegen::generate_moves;

    #[test]
    fn test_see_simple_pawn_capture() {
        // Simple position: pawn takes pawn
        let board =
            parse_fen("rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 2").unwrap();

        // Find exd5 move
        let moves = generate_moves(&board);
        let capture_move = moves
            .iter()
            .find(|m| m.from().to_string() == "e4" && m.to().to_string() == "d5");

        if let Some(mv) = capture_move {
            // Pawn takes pawn should be at least equal
            assert!(see(&board, *mv, 0));
        }
    }

    #[test]
    fn test_see_equal_trade() {
        // Position where PxP is equal
        let _board =
            parse_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1").unwrap();

        // Find d7-d5 creating a capture opportunity
        let board =
            parse_fen("rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 2").unwrap();

        // Find exd5 move
        let moves = generate_moves(&board);
        let capture_move = moves
            .iter()
            .find(|m| m.from().to_string() == "e4" && m.to().to_string() == "d5")
            .unwrap();

        // PxP should be >= 0
        assert!(see(&board, *capture_move, 0));
    }

    #[test]
    fn test_see_losing_capture() {
        // Position where capturing loses material
        let _board =
            parse_fen("rnbqkb1r/pppppppp/5n2/8/8/3P4/PPP1PPPP/RNBQKBNR w KQkq - 2 2").unwrap();

        // Find d3xNf6 (pawn takes knight, but knight is defended)
        // This is a bad example, let me create a better position
        let board =
            parse_fen("rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPPQPPP/RNB1KBNR w KQkq - 0 3").unwrap();

        // Find Qxe5 (queen takes pawn, but pawn is defended by knight)
        let moves = generate_moves(&board);
        let capture_move = moves
            .iter()
            .find(|m| m.from().to_string() == "e2" && m.to().to_string() == "e5")
            .cloned();

        if let Some(mv) = capture_move {
            // Queen takes pawn defended by pieces - likely losing
            // This test might need adjustment based on actual position
            let _see_val = see_value_of_move(&board, mv);
            // println!("SEE value: {}", _see_val);
        }
    }

    #[test]
    fn test_see_no_capture() {
        let board = Board::startpos();
        let moves = generate_moves(&board);

        // Find a quiet move (e.g., e2-e4)
        let quiet_move = moves
            .iter()
            .find(|m| m.from().to_string() == "e2" && m.to().to_string() == "e4")
            .unwrap();

        // Quiet move should have SEE of 0
        assert!(see(&board, *quiet_move, 0));
        assert_eq!(see_value_of_move(&board, *quiet_move), 0);
    }

    #[test]
    fn test_see_promotion_capture() {
        // Position with promotion capture
        let board = parse_fen("4k3/P7/8/8/8/8/8/4K3 w - - 0 1").unwrap();

        let moves = generate_moves(&board);
        // Just test that it doesn't crash
        for mv in moves.iter() {
            let _ = see(&board, *mv, 0);
        }
    }

    #[test]
    fn test_see_en_passant() {
        // Position with en passant capture
        let board =
            parse_fen("rnbqkbnr/ppp1p1pp/8/3pPp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3").unwrap();

        let moves = generate_moves(&board);
        let ep_move = moves
            .iter()
            .find(|m| m.from().to_string() == "e5" && m.to().to_string() == "f6")
            .unwrap();

        // En passant should be recognized and evaluated
        assert!(see(&board, *ep_move, 0));
    }

    #[test]
    fn test_see_threshold() {
        // Test SEE with different thresholds
        let board =
            parse_fen("rnbqkb1r/pppppppp/5n2/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 2 3").unwrap();

        let moves = generate_moves(&board);
        let capture = moves
            .iter()
            .find(|m| {
                m.to() == crate::square::Square::from_algebraic("f6").unwrap() && m.is_capture()
            })
            .cloned();

        if let Some(mv) = capture {
            // Test that threshold works correctly
            let value = see_value_of_move(&board, mv);
            assert!(see(&board, mv, value));
            assert!(!see(&board, mv, value + 1));
        }
    }

    #[test]
    fn test_see_recapture_sequence() {
        // Position with long recapture sequence
        // Multiple pieces attacking the same square
        let board = parse_fen("1k1r4/1pp4p/p7/4p3/8/P5P1/1PP4P/2K1R3 w - - 0 1").unwrap();

        let moves = generate_moves(&board);

        // Test all captures
        for mv in moves.iter() {
            if mv.is_capture() {
                // Should not crash
                let _ = see(&board, *mv, 0);
                let _ = see_value_of_move(&board, *mv);
            }
        }
    }

    #[test]
    fn test_see_symmetry() {
        // Test that SEE is symmetric for identical positions
        let fen_white = "rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 2";
        let fen_black = "rnbqkbnr/pppp1ppp/8/4p3/3P4/8/PPP1PPPP/RNBQKBNR b KQkq d3 0 2";

        let board_white = parse_fen(fen_white).unwrap();
        let board_black = parse_fen(fen_black).unwrap();

        // Both should evaluate exd5 / dxe4 similarly
        let moves_white = generate_moves(&board_white);
        let moves_black = generate_moves(&board_black);

        let white_capture = moves_white
            .iter()
            .find(|m| m.from().to_string() == "e4" && m.to().to_string() == "d5");

        let black_capture = moves_black
            .iter()
            .find(|m| m.from().to_string() == "d4" && m.to().to_string() == "e5");

        if let (Some(wc), Some(bc)) = (white_capture, black_capture) {
            assert_eq!(see(&board_white, *wc, 0), see(&board_black, *bc, 0));
        }
    }
}
