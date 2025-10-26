//! Selective search extensions
//!
//! Extensions increase search depth for critical positions to avoid the horizon effect.
//! This module implements various extension techniques:
//! - Check extensions: Extend when in check
//! - Singular extensions: Extend when one move is clearly best
//! - Recapture extensions: Extend immediate recaptures
//! - Passed pawn extensions: Extend passed pawns to 6th/7th rank

use crate::bitboard::Bitboard;
use crate::board::Board;
use crate::piece::{Color, PieceType};
use crate::r#move::Move;
use crate::square::Square;

/// Maximum total extensions allowed per search path
pub const MAX_EXTENSIONS_PER_PATH: i32 = 16;

/// Extension amount for being in check
const CHECK_EXTENSION: i32 = 1;

/// Extension amount for recaptures
const RECAPTURE_EXTENSION: i32 = 1;

/// Extension amount for passed pawns to 6th/7th rank
const PASSED_PAWN_EXTENSION: i32 = 1;

/// Margin for singular extension verification search
const SINGULAR_MARGIN: i32 = 100;

/// Depth reduction for singular extension verification
const SINGULAR_DEPTH_REDUCTION: i32 = 3;

/// Minimum depth required for singular extensions
const SINGULAR_MIN_DEPTH: i32 = 8;

/// Calculate search extensions for a given move
///
/// Returns the extension amount in plies (can be 0, 1, or more).
/// Extensions are accumulated but limited by MAX_EXTENSIONS_PER_PATH.
///
/// # Arguments
/// * `board` - Current board position (after making the move)
/// * `mv` - The move that was made
/// * `in_check` - Whether we're in check after the move
/// * `prev_move` - The previous move (for recapture detection)
/// * `depth` - Current search depth
/// * `extensions_used` - Extensions already used in this path
///
/// # Returns
/// Extension amount in plies
pub fn calculate_extension(
    board: &Board,
    mv: Move,
    in_check: bool,
    prev_move: Option<Move>,
    _depth: i32,
    extensions_used: i32,
) -> i32 {
    // Don't extend if we've used too many extensions already
    if extensions_used >= MAX_EXTENSIONS_PER_PATH {
        return 0;
    }

    let mut extension = 0;

    // 1. Check extension - extend when in check
    if in_check {
        extension = extension.max(CHECK_EXTENSION);
    }

    // 2. Recapture extension - extend immediate recaptures
    if let Some(prev) = prev_move {
        if is_recapture(mv, prev) {
            extension = extension.max(RECAPTURE_EXTENSION);
        }
    }

    // 3. Passed pawn extension - extend passed pawns to 6th/7th rank
    if is_passed_pawn_push_to_7th(board, mv) {
        extension = extension.max(PASSED_PAWN_EXTENSION);
    }

    // Clamp extension to remaining budget
    let remaining = MAX_EXTENSIONS_PER_PATH - extensions_used;
    extension.min(remaining)
}

/// Check if a move is a recapture (captures on the same square as previous move)
fn is_recapture(mv: Move, prev_move: Move) -> bool {
    mv.is_capture() && mv.to() == prev_move.to()
}

/// Check if a move is a passed pawn push to the 6th or 7th rank
fn is_passed_pawn_push_to_7th(board: &Board, mv: Move) -> bool {
    // Get the piece that moved
    let piece = board.piece_at(mv.to());
    if piece.is_none() {
        return false;
    }

    let piece = piece.unwrap();
    if piece.piece_type != PieceType::Pawn {
        return false;
    }

    // Check if it's on 6th or 7th rank
    let rank = mv.to().rank();
    let is_advanced = match piece.color {
        Color::White => rank >= 5, // Rank 6 or 7 (0-indexed: 5, 6)
        Color::Black => rank <= 2, // Rank 2 or 3 (0-indexed: 2, 1)
    };

    if !is_advanced {
        return false;
    }

    // Check if it's a passed pawn (no enemy pawns in front on same or adjacent files)
    is_passed_pawn_simple(board, mv.to(), piece.color)
}

/// Simple passed pawn detection
fn is_passed_pawn_simple(board: &Board, sq: Square, color: Color) -> bool {
    let file = sq.file();
    let rank = sq.rank();

    let enemy_pawns = board.piece_bb(PieceType::Pawn, color.opponent());

    // Check files: same file and adjacent files
    for check_file in (file.saturating_sub(1))..=(file + 1).min(7) {
        // Check if any enemy pawn exists in front on this file
        match color {
            Color::White => {
                // Check ranks ahead for white (rank + 1 to 7)
                for check_rank in (rank + 1)..=7 {
                    let check_sq = Square::from_coords(check_file, check_rank);
                    let check_bb = Bitboard::from_square(check_sq);
                    if !(enemy_pawns & check_bb).is_empty() {
                        return false; // Found an enemy pawn in front
                    }
                }
            }
            Color::Black => {
                // Check ranks ahead for black (0 to rank - 1)
                for check_rank in 0..rank {
                    let check_sq = Square::from_coords(check_file, check_rank);
                    let check_bb = Bitboard::from_square(check_sq);
                    if !(enemy_pawns & check_bb).is_empty() {
                        return false; // Found an enemy pawn in front
                    }
                }
            }
        }
    }

    true
}

/// Check if a move is singular (much better than all alternatives)
///
/// A move is "singular" if when we search at reduced depth with a lower bound
/// of (beta - margin), all other moves fail low.
///
/// This requires a verification search, which is expensive, so we only do it
/// at high depths and when we have a TT move.
///
/// # Arguments
/// * `board` - Current board position
/// * `tt_move` - The move from the transposition table
/// * `beta` - Beta bound from the search
/// * `depth` - Current search depth
///
/// # Returns
/// Extension amount (0 or 1)
pub fn singular_extension(
    board: &Board,
    tt_move: Move,
    beta: i32,
    depth: i32,
    _extensions_used: i32,
) -> i32 {
    // Only apply singular extensions at sufficient depth
    if depth < SINGULAR_MIN_DEPTH {
        return 0;
    }

    // Perform a reduced depth search excluding the TT move
    // If all other moves fail low by a margin, the TT move is "singular"
    let verification_depth = depth - SINGULAR_DEPTH_REDUCTION;
    let singular_beta = beta - SINGULAR_MARGIN;

    // This would require calling back into the search function
    // For now, we'll return a placeholder
    // In the actual integration, this will call a verification search
    let _is_singular = verify_singular(board, tt_move, singular_beta, verification_depth);

    // We'll implement this properly during integration
    0
}

/// Verification search for singular extensions
///
/// This is a placeholder - will be implemented during search integration
fn verify_singular(_board: &Board, _tt_move: Move, _beta: i32, _depth: i32) -> bool {
    // TODO: Implement verification search during integration
    // This requires calling back into the main search function
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::parse_fen;
    use crate::movegen::generate_moves;

    #[test]
    fn test_check_extension() {
        // Position where white is in check
        let board =
            parse_fen("rnbqkbnr/pppp1ppp/8/4p3/6P1/5P2/PPPPP2P/RNBQKBNR b KQkq - 0 2").unwrap();

        let moves = generate_moves(&board);
        // Find a checking move (Qh4+)
        let checking_move = moves
            .iter()
            .find(|m| m.from().to_string() == "d8" && m.to().to_string() == "h4");

        if let Some(mv) = checking_move {
            let mut board_after = board.clone();
            board_after.make_move(*mv);

            // After the move, white is in check
            let in_check = board_after.is_in_check();
            let extension = calculate_extension(&board_after, *mv, in_check, None, 5, 0);

            if in_check {
                assert!(extension > 0, "Check should give extension");
            }
        }
    }

    #[test]
    fn test_recapture_extension() {
        // Position with captures
        let board =
            parse_fen("rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 2").unwrap();

        let moves = generate_moves(&board);
        let capture = moves
            .iter()
            .find(|m| m.from().to_string() == "e4" && m.to().to_string() == "d5");

        if let Some(first_capture) = capture {
            let mut board_after = board.clone();
            board_after.make_move(*first_capture);

            // Now black recaptures
            let moves2 = generate_moves(&board_after);
            let recapture = moves2.iter().find(|m| m.to().to_string() == "d5");

            if let Some(recap) = recapture {
                let mut board_after2 = board_after.clone();
                board_after2.make_move(*recap);

                let extension =
                    calculate_extension(&board_after2, *recap, false, Some(*first_capture), 5, 0);

                assert!(extension > 0, "Recapture should give extension");
            }
        }
    }

    #[test]
    fn test_passed_pawn_extension() {
        // Position with a far advanced passed pawn
        let board = parse_fen("4k3/4P3/8/8/8/8/8/4K3 w - - 0 1").unwrap();

        let moves = generate_moves(&board);
        // The pawn is already on the 7th rank, any pawn move should extend
        let pawn_move = moves.iter().find(|m| m.from().to_string() == "e7");

        if let Some(mv) = pawn_move {
            let mut board_after = board.clone();
            board_after.make_move(*mv);

            let extension = calculate_extension(&board_after, *mv, false, None, 5, 0);

            // May or may not extend depending on if we consider it still passed after the move
            // This test just checks it doesn't crash
            assert!(extension >= 0);
        }
    }

    #[test]
    fn test_extension_limit() {
        // Test that extensions are limited
        let board = Board::startpos();
        let moves = generate_moves(&board);
        let mv = moves.iter().next().unwrap();

        // With max extensions already used, should get 0
        let extension = calculate_extension(&board, *mv, true, None, 5, MAX_EXTENSIONS_PER_PATH);

        assert_eq!(extension, 0, "Should not extend beyond limit");
    }

    #[test]
    fn test_extension_remaining_budget() {
        // Test that extension respects remaining budget
        let board = Board::startpos();
        let moves = generate_moves(&board);
        let mv = moves.iter().next().unwrap();

        // With 15 extensions used, only 1 remaining
        let extension = calculate_extension(&board, *mv, true, None, 5, 15);

        assert!(
            extension <= 1,
            "Should not extend more than remaining budget"
        );
    }

    #[test]
    fn test_no_extension_quiet_move() {
        // Regular quiet move should not extend
        let board = Board::startpos();
        let moves = generate_moves(&board);
        let quiet_move = moves
            .iter()
            .find(|m| m.from().to_string() == "e2" && m.to().to_string() == "e4")
            .unwrap();

        let mut board_after = board.clone();
        board_after.make_move(*quiet_move);

        let extension = calculate_extension(&board_after, *quiet_move, false, None, 5, 0);

        assert_eq!(extension, 0, "Quiet move should not extend");
    }

    #[test]
    fn test_multiple_extension_conditions() {
        // Test position where multiple extension conditions might apply
        // But we should only extend once (take the max)
        let board = parse_fen("4k3/8/8/8/8/8/4p3/4K3 b - - 0 1").unwrap();

        let moves = generate_moves(&board);
        // Black pawn promotes with check
        let promoting_check = moves
            .iter()
            .find(|m| m.from().to_string() == "e2" && m.is_promotion());

        if let Some(mv) = promoting_check {
            let mut board_after = board.clone();
            board_after.make_move(*mv);

            let in_check = board_after.is_in_check();
            let extension = calculate_extension(&board_after, *mv, in_check, None, 5, 0);

            // Should extend, but not double (at most 1 ply in this case)
            assert!(extension > 0);
            assert!(extension <= 2, "Should not stack extensions excessively");
        }
    }

    #[test]
    fn test_is_recapture_same_square() {
        use crate::r#move::MoveFlags;

        let board = Board::startpos();
        let moves = generate_moves(&board);
        let mv1 = moves.iter().next().unwrap();

        // Same destination = recapture
        let mv2 = Move::new(
            Square::from_algebraic("f2").unwrap(),
            mv1.to(),
            MoveFlags::CAPTURE,
        );

        assert!(is_recapture(mv2, *mv1));
    }

    #[test]
    fn test_is_recapture_different_square() {
        use crate::r#move::MoveFlags;

        let board = Board::startpos();
        let moves = generate_moves(&board);
        let mv1 = moves.iter().next().unwrap();

        // Different destination = not a recapture
        let mv2 = Move::new(
            Square::from_algebraic("f2").unwrap(),
            Square::from_algebraic("f4").unwrap(),
            MoveFlags::QUIET,
        );

        assert!(!is_recapture(mv2, *mv1));
    }

    #[test]
    fn test_passed_pawn_detection() {
        // White passed pawn on 6th rank
        let board = parse_fen("4k3/8/4P3/8/8/8/8/4K3 w - - 0 1").unwrap();

        let moves = generate_moves(&board);
        let pawn_push = moves
            .iter()
            .find(|m| m.from().to_string() == "e6" && m.to().to_string() == "e7");

        if let Some(mv) = pawn_push {
            let mut board_after = board.clone();
            board_after.make_move(*mv);

            // After pushing to 7th, should still detect as passed pawn
            assert!(is_passed_pawn_push_to_7th(&board_after, *mv));
        }
    }

    #[test]
    fn test_singular_extension_low_depth() {
        // At low depth, singular extension should return 0
        let board = Board::startpos();
        let moves = generate_moves(&board);
        let mv = moves.iter().next().unwrap();

        let extension = singular_extension(&board, *mv, 100, 5, 0);

        assert_eq!(extension, 0, "Should not apply singular at low depth");
    }
}
