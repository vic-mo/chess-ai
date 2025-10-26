//! Game phase calculation for smooth middlegame to endgame transition.
//!
//! The phase value ranges from 0 (opening/middlegame) to 256 (pure endgame).
//! This allows for smooth interpolation between middlegame and endgame evaluation.

use crate::board::Board;
use crate::piece::{Color, PieceType};

/// Maximum phase value (pure endgame).
pub const MAX_PHASE: i32 = 256;

/// Total material phase value at the start of the game.
/// Knight=1, Bishop=1, Rook=2, Queen=4
/// Total = 4 Knights + 4 Bishops + 4 Rooks + 2 Queens = 4+4+8+8 = 24
const TOTAL_PHASE: i32 = 24;

/// Material phase weights for each piece type.
/// Pawns don't contribute to phase (they appear throughout the game).
const KNIGHT_PHASE: i32 = 1;
const BISHOP_PHASE: i32 = 1;
const ROOK_PHASE: i32 = 2;
const QUEEN_PHASE: i32 = 4;

/// Calculate the current game phase based on remaining material.
///
/// Returns a value from 0 (opening/middlegame with all pieces) to 256 (endgame with few pieces).
///
/// # Examples
///
/// ```
/// use engine::board::Board;
/// use engine::eval::phase::calculate_phase;
///
/// let board = Board::startpos();
/// let phase = calculate_phase(&board);
/// assert_eq!(phase, 0); // Starting position = phase 0
/// ```
pub fn calculate_phase(board: &Board) -> i32 {
    let mut phase = 0;

    // Count material for both sides
    for color in [Color::White, Color::Black] {
        let knights = board.piece_bb(PieceType::Knight, color).count();
        let bishops = board.piece_bb(PieceType::Bishop, color).count();
        let rooks = board.piece_bb(PieceType::Rook, color).count();
        let queens = board.piece_bb(PieceType::Queen, color).count();

        phase += knights as i32 * KNIGHT_PHASE;
        phase += bishops as i32 * BISHOP_PHASE;
        phase += rooks as i32 * ROOK_PHASE;
        phase += queens as i32 * QUEEN_PHASE;
    }

    // Convert to phase value (higher phase = more endgame)
    // phase = 256 - (current_material * 256 / TOTAL_PHASE)
    let phase_value = MAX_PHASE - (phase * MAX_PHASE / TOTAL_PHASE);

    // Clamp to valid range [0, 256]
    phase_value.clamp(0, MAX_PHASE)
}

/// Interpolate between middlegame and endgame scores based on phase.
///
/// # Arguments
/// * `mg_score` - Middlegame score
/// * `eg_score` - Endgame score
/// * `phase` - Current game phase (0 = middlegame, 256 = endgame)
///
/// # Returns
/// Interpolated score
///
/// # Examples
///
/// ```
/// use engine::eval::phase::interpolate;
///
/// // Pure middlegame (phase 0)
/// assert_eq!(interpolate(100, 200, 0), 100);
///
/// // Pure endgame (phase 256)
/// assert_eq!(interpolate(100, 200, 256), 200);
///
/// // Halfway (phase 128)
/// assert_eq!(interpolate(100, 200, 128), 150);
/// ```
pub fn interpolate(mg_score: i32, eg_score: i32, phase: i32) -> i32 {
    // Linear interpolation: mg_score * (256 - phase) + eg_score * phase
    // Divided by 256 for normalization
    ((mg_score * (MAX_PHASE - phase)) + (eg_score * phase)) / MAX_PHASE
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::parse_fen;

    #[test]
    fn test_phase_startpos() {
        // Starting position should be phase 0 (all pieces present)
        let board = Board::startpos();
        let phase = calculate_phase(&board);
        assert_eq!(phase, 0);
    }

    #[test]
    fn test_phase_bare_kings() {
        // Bare kings should be phase 256 (pure endgame)
        let board = parse_fen("8/8/8/4k3/4K3/8/8/8 w - - 0 1").unwrap();
        let phase = calculate_phase(&board);
        assert_eq!(phase, 256);
    }

    #[test]
    fn test_phase_kqk() {
        // K+Q vs K should be near endgame
        // 1 Queen = 4 phase points out of 24 total
        // phase = 256 - (4 * 256 / 24) = 256 - 42 = 214
        let board = parse_fen("8/8/8/4k3/4K3/8/8/4Q3 w - - 0 1").unwrap();
        let phase = calculate_phase(&board);
        assert_eq!(phase, 214); // Close to endgame
    }

    #[test]
    fn test_phase_krk() {
        // K+R vs K
        // 1 Rook = 2 phase points out of 24 total
        // phase = 256 - (2 * 256 / 24) = 256 - 21 = 235
        let board = parse_fen("8/8/8/4k3/4K3/8/8/4R3 w - - 0 1").unwrap();
        let phase = calculate_phase(&board);
        assert_eq!(phase, 235); // Very close to endgame
    }

    #[test]
    fn test_phase_middlegame() {
        // Middlegame position with some trades
        // Remove 2 minor pieces (2 phase points)
        // phase = 256 - (22 * 256 / 24) = 256 - 234 = 22
        let board =
            parse_fen("r1bqkb1r/pppp1ppp/2n2n2/4p3/4P3/3P1N2/PPP2PPP/RNBQKB1R w KQkq - 0 1")
                .unwrap();
        let phase = calculate_phase(&board);
        // Should be in early middlegame (low phase value)
        assert!(
            phase < 64,
            "Phase should be < 64 in middlegame, got {}",
            phase
        );
    }

    #[test]
    fn test_phase_symmetry() {
        // Phase should be same regardless of side to move
        let fen = "r1bqkb1r/pppp1ppp/2n2n2/4p3/4P3/3P1N2/PPP2PPP/RNBQKB1R w KQkq - 0 1";
        let board_white = parse_fen(fen).unwrap();

        let fen_black = "r1bqkb1r/pppp1ppp/2n2n2/4p3/4P3/3P1N2/PPP2PPP/RNBQKB1R b KQkq - 0 1";
        let board_black = parse_fen(fen_black).unwrap();

        assert_eq!(calculate_phase(&board_white), calculate_phase(&board_black));
    }

    #[test]
    fn test_phase_boundaries() {
        // Test that phase is always within [0, 256]
        let positions = vec![
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", // startpos
            "8/8/8/4k3/4K3/8/8/8 w - - 0 1",                            // bare kings
            "rnbqkbnr/8/8/8/8/8/8/RNBQKBNR w - - 0 1",                  // no pawns
            "4k3/pppppppp/8/8/8/8/PPPPPPPP/4K3 w - - 0 1",              // only pawns
        ];

        for fen in positions {
            let board = parse_fen(fen).unwrap();
            let phase = calculate_phase(&board);
            assert!(
                (0..=256).contains(&phase),
                "Phase {} out of bounds for FEN: {}",
                phase,
                fen
            );
        }
    }

    #[test]
    fn test_interpolate_pure_middlegame() {
        // Phase 0 = pure middlegame, should return mg_score
        assert_eq!(interpolate(100, 200, 0), 100);
        assert_eq!(interpolate(-50, 75, 0), -50);
    }

    #[test]
    fn test_interpolate_pure_endgame() {
        // Phase 256 = pure endgame, should return eg_score
        assert_eq!(interpolate(100, 200, 256), 200);
        assert_eq!(interpolate(-50, 75, 256), 75);
    }

    #[test]
    fn test_interpolate_halfway() {
        // Phase 128 = halfway, should return average
        assert_eq!(interpolate(100, 200, 128), 150);
        assert_eq!(interpolate(0, 100, 128), 50);
        assert_eq!(interpolate(-100, 100, 128), 0);
    }

    #[test]
    fn test_interpolate_quarter() {
        // Phase 64 = 1/4 into endgame
        // mg_score * (256 - 64) / 256 + eg_score * 64 / 256
        // = mg_score * 192/256 + eg_score * 64/256
        // = mg_score * 0.75 + eg_score * 0.25
        assert_eq!(interpolate(100, 200, 64), 125);
        assert_eq!(interpolate(0, 100, 64), 25);
    }

    #[test]
    fn test_interpolate_three_quarters() {
        // Phase 192 = 3/4 into endgame
        // mg_score * 64/256 + eg_score * 192/256
        // = mg_score * 0.25 + eg_score * 0.75
        assert_eq!(interpolate(100, 200, 192), 175);
        assert_eq!(interpolate(0, 100, 192), 75);
    }
}
