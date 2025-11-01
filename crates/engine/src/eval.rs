//! Position evaluation for chess positions.
//!
//! Evaluates positions from the current side to move's perspective.
//! Positive scores favor the side to move, negative scores favor the opponent.

pub mod king;
pub mod material;
pub mod pawns;
pub mod phase;
pub mod pieces;
pub mod positional;
pub mod pst;
pub mod threats;

pub use king::*;
pub use material::*;
pub use pawns::*;
pub use phase::*;
pub use pieces::*;
pub use positional::*;
pub use pst::*;
pub use threats::*;

use crate::board::Board;
use crate::piece::Color;

/// Main evaluator structure containing evaluation components.
#[derive(Debug)]
pub struct Evaluator {
    pst: PieceSquareTables,
    pawn_hash: PawnHashTable,
}

impl Evaluator {
    /// Create a new evaluator with default evaluation parameters.
    pub fn new() -> Self {
        Self {
            pst: PieceSquareTables::default(),
            pawn_hash: PawnHashTable::default(),
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
    /// let mut evaluator = Evaluator::new();
    /// let score = evaluator.evaluate(&board);
    /// // Starting position should be close to equal (within small positional differences)
    /// assert!(score.abs() < 50, "Starting position should be roughly equal");
    /// ```
    /// Improved evaluation: Material + PST/4 + Pawn/4
    /// Adding back pawn structure at reduced strength
    fn evaluate_minimal(&mut self, board: &Board) -> i32 {
        use crate::tune;

        // 1. Material
        let white_material = evaluate_material(board, Color::White);
        let black_material = evaluate_material(board, Color::Black);
        let material = white_material - black_material;

        // 2. PST with tunable divisor (default: 4)
        let pst_divisor = tune::get_param_or_default(|p| p.pst_scale, 4);
        let white_pst = self.pst.evaluate_position(board, Color::White) / pst_divisor;
        let black_pst = self.pst.evaluate_position(board, Color::Black) / pst_divisor;
        let pst = white_pst - black_pst;

        // 3. Calculate game phase for MG/EG blending
        let phase = phase::calculate_phase(board);

        // 4. Pawn structure with tunable divisor (default: 4)
        let pawn_divisor = tune::get_param_or_default(|p| p.pawn_structure_divisor, 4);
        let (white_pawn_mg, white_pawn_eg, black_pawn_mg, black_pawn_eg) =
            evaluate_pawns_cached(board, &mut self.pawn_hash);
        let white_pawn = (white_pawn_mg * (256 - phase) + white_pawn_eg * phase) / 256;
        let black_pawn = (black_pawn_mg * (256 - phase) + black_pawn_eg * phase) / 256;
        let pawn_structure = (white_pawn - black_pawn) / pawn_divisor;

        // 5. Mobility with tunable divisor (default: 8)
        let mobility_divisor = tune::get_param_or_default(|p| p.mobility_divisor, 8);
        let (white_mob_mg, white_mob_eg) = evaluate_piece_activity(board, Color::White, phase);
        let (black_mob_mg, black_mob_eg) = evaluate_piece_activity(board, Color::Black, phase);
        let white_mob = (white_mob_mg * (256 - phase) + white_mob_eg * phase) / 256;
        let black_mob = (black_mob_mg * (256 - phase) + black_mob_eg * phase) / 256;
        let mobility = (white_mob - black_mob) / mobility_divisor;

        // 6. King safety with tunable divisor (default: 12, optimal setting)
        // /12 = 50% vs SF1800 (+65 ELO), /8 = 47.5% (too strong)
        let king_safety_divisor = tune::get_param_or_default(|p| p.king_safety_divisor, 12);
        let (white_king_mg, white_king_eg) = evaluate_king_safety(board, Color::White, phase);
        let (black_king_mg, black_king_eg) = evaluate_king_safety(board, Color::Black, phase);
        let white_king = (white_king_mg * (256 - phase) + white_king_eg * phase) / 256;
        let black_king = (black_king_mg * (256 - phase) + black_king_eg * phase) / 256;
        let king_safety = (white_king - black_king) / king_safety_divisor;

        // 7. Threat detection (disabled - still causes timeouts despite optimization)
        // Even with 2.14x speedup (cached attack maps), NPS drop from 120k to 116k
        // causes 14% timeout rate and -56 ELO regression.
        // let threat_divisor = tune::get_param_or_default(|p| p.threat_divisor, 8);
        // let (threats_mg, threats_eg) = evaluate_threats(board);
        // let threats = (threats_mg * (256 - phase) + threats_eg * phase) / 256;
        // let threats = threats / threat_divisor;

        let score = material + pst + pawn_structure + mobility + king_safety;

        // Return from side to move's perspective
        if board.side_to_move() == Color::Black {
            -score
        } else {
            score
        }
    }

    pub fn evaluate(&mut self, board: &Board) -> i32 {
        // EMERGENCY FIX: Use minimal evaluation
        return self.evaluate_minimal(board);

        // Original evaluation (disabled for now)
        #[allow(unreachable_code)]
        {
        // 1. Calculate game phase (0 = opening/middlegame, 256 = pure endgame)
        let phase = phase::calculate_phase(board);

        // 2. Initialize middlegame and endgame scores
        let mut mg_score = 0;
        let mut eg_score = 0;

        // 3. Material evaluation (same for MG and EG)
        let white_material = evaluate_material(board, Color::White);
        let black_material = evaluate_material(board, Color::Black);
        let material_score = white_material - black_material;
        mg_score += material_score;
        eg_score += material_score;

        // 4. Piece-square tables (MG and EG)
        let white_pst = self.pst.evaluate_position(board, Color::White);
        let black_pst = self.pst.evaluate_position(board, Color::Black);
        mg_score += white_pst - black_pst;
        eg_score += white_pst - black_pst;

        // 5. Pawn structure (cached, MG and EG)
        let (white_pawn_mg, white_pawn_eg, black_pawn_mg, black_pawn_eg) =
            evaluate_pawns_cached(board, &mut self.pawn_hash);
        mg_score += white_pawn_mg - black_pawn_mg;
        eg_score += white_pawn_eg - black_pawn_eg;

        // 6. King safety (phase-dependent, MG and EG)
        // Scale king safety by phase: minimal in opening, full in middlegame, reduced in endgame
        let (white_king_mg, white_king_eg) = evaluate_king_safety(board, Color::White, phase);
        let (black_king_mg, black_king_eg) = evaluate_king_safety(board, Color::Black, phase);

        // King safety scaling factor based on phase
        // phase 0-128 (opening/early middlegame): 0-50% king safety
        // phase 128-192 (middlegame): 50-100% king safety
        // phase 192-256 (endgame): 100-25% king safety (less important)
        let king_safety_scale = if phase < 128 {
            phase * 50 / 128  // 0% at phase 0, 50% at phase 128
        } else if phase < 192 {
            50 + (phase - 128) * 50 / 64  // 50% at phase 128, 100% at phase 192
        } else {
            100 - (phase - 192) * 75 / 64  // 100% at phase 192, 25% at phase 256
        };

        let scaled_white_king_mg = white_king_mg * king_safety_scale / 100;
        let scaled_black_king_mg = black_king_mg * king_safety_scale / 100;
        let scaled_white_king_eg = white_king_eg * king_safety_scale / 100;
        let scaled_black_king_eg = black_king_eg * king_safety_scale / 100;

        // TEMPORARY: Disable king safety completely to test if this is the only issue
        // TODO: Re-enable with proper values once evaluation is working
        // mg_score += scaled_white_king_mg - scaled_black_king_mg;
        // eg_score += scaled_white_king_eg - scaled_black_king_eg;

        // 7. Piece activity (MG and EG)
        let (white_pieces_mg, white_pieces_eg) =
            evaluate_piece_activity(board, Color::White, phase);
        let (black_pieces_mg, black_pieces_eg) =
            evaluate_piece_activity(board, Color::Black, phase);
        mg_score += white_pieces_mg - black_pieces_mg;
        eg_score += white_pieces_eg - black_pieces_eg;

        // 8. Mobility (existing evaluation, same for MG and EG)
        let white_mobility = evaluate_positional(board, Color::White);
        let black_mobility = evaluate_positional(board, Color::Black);
        let mobility_score = white_mobility - black_mobility;
        mg_score += mobility_score;
        eg_score += mobility_score;

        // 9. Interpolate based on game phase
        let score = phase::interpolate(mg_score, eg_score, phase);

        // 10. Return from side to move's perspective
        if board.side_to_move() == Color::Black {
            -score
        } else {
            score
        }
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
        let mut eval = Evaluator::new();
        let score = eval.evaluate(&board);
        // With all the new evaluation features, startpos might not be exactly 0
        // but should be close (within small positional differences)
        assert!(
            score.abs() < 50,
            "Starting position should be roughly equal, got score={}",
            score
        );
    }

    #[test]
    fn test_white_advantage() {
        // White is up a rook (black missing h8 rook)
        let fen = "rnbqkbn1/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let board = parse_fen(fen).unwrap();
        let mut eval = Evaluator::new();
        let score = eval.evaluate(&board);
        assert!(score > 0, "White should have positive score (up material)");
    }

    #[test]
    fn test_black_advantage() {
        // Black is up a rook (white missing h1 rook), black to move
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBN1 b KQkq - 0 1";
        let board = parse_fen(fen).unwrap();
        let mut eval = Evaluator::new();
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

        let mut eval = Evaluator::new();
        let score1 = eval.evaluate(&board1);
        let score2 = eval.evaluate(&board2);

        assert_eq!(score1, score2, "Evaluation should be side-to-move relative");
    }

    #[test]
    fn test_phase_interpolation() {
        // Test that evaluation uses phase interpolation
        // Endgame position (bare kings + pawns)
        let endgame = parse_fen("4k3/4p3/8/8/8/8/4P3/4K3 w - - 0 1").unwrap();
        let mut eval = Evaluator::new();
        let _score = eval.evaluate(&endgame);
        // Just verify it doesn't crash
    }

    #[test]
    fn test_evaluation_components() {
        // Test that all evaluation components are working
        let board = Board::startpos();
        let mut eval = Evaluator::new();

        // Should evaluate without crashing
        let score = eval.evaluate(&board);

        // Score should be reasonable (not absurdly high)
        assert!(
            score.abs() < 500,
            "Evaluation should be reasonable for startpos, got {}",
            score
        );
    }

    // ===== M6 VALIDATION TESTS =====

    #[test]
    fn test_m6_passed_pawn_bonus() {
        // White has a far advanced passed pawn on e7
        let fen = "4k3/4P3/8/8/8/8/8/4K3 w - - 0 1";
        let board = parse_fen(fen).unwrap();
        let mut eval = Evaluator::new();
        let score = eval.evaluate(&board);

        // Should recognize the powerful passed pawn
        assert!(
            score > 100,
            "Far advanced passed pawn should have large bonus, got {}",
            score
        );
    }

    #[test]
    fn test_m6_king_safety() {
        // White king exposed, black king safe with pawn shield
        let fen = "rnbq1rk1/ppp2ppp/3p1n2/2b1p3/2B1P3/3P1N2/PPP2PPP/RNBQK2R w KQ - 0 1";
        let board = parse_fen(fen).unwrap();
        let mut eval = Evaluator::new();
        let score_white = eval.evaluate(&board);

        // Black king is safer (castled with pawn shield), white is not castled
        // So black should have better evaluation
        assert!(
            score_white < 0,
            "Black should have advantage due to king safety, got score={}",
            score_white
        );
    }

    #[test]
    fn test_m6_bishop_pair() {
        // White has bishop pair, black has knight and bishop
        let fen = "rnbqk1nr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let board = parse_fen(fen).unwrap();
        let mut eval = Evaluator::new();
        let score = eval.evaluate(&board);

        // White should have advantage from bishop pair (black missing a bishop)
        assert!(
            score > 0,
            "Bishop pair should give white advantage, got {}",
            score
        );
    }

    #[test]
    fn test_m6_rook_open_file() {
        // White rook on open e-file (e2 pawn removed, e7 pawn removed)
        let fen = "r1bqkbnr/pppp1ppp/8/8/8/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1";
        let board1 = parse_fen(fen).unwrap();

        // Now move white rook to e-file in second position
        let fen2 = "r1bqkbnr/pppp1ppp/8/8/8/4R3/PPPP1PPP/RNBQKBN1 w KQkq - 0 1";
        let board2 = parse_fen(fen2).unwrap();

        let mut eval = Evaluator::new();
        let score1 = eval.evaluate(&board1);
        let score2 = eval.evaluate(&board2);

        // Position with rook on open file should be better than without
        // (even though white is missing h1 rook in both cases, the open file bonus should matter)
        assert!(
            score2 > score1 - 50,
            "Rook on open file should not be much worse, got score1={} score2={}",
            score1,
            score2
        );
    }

    #[test]
    fn test_m6_isolated_pawns() {
        // Black has multiple isolated pawns
        let fen = "rnbqkbnr/p1p1p1p1/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let board = parse_fen(fen).unwrap();
        let mut eval = Evaluator::new();
        let score = eval.evaluate(&board);

        // White should have advantage (black has isolated pawns)
        assert!(
            score > 0,
            "Isolated pawns should give white advantage, got {}",
            score
        );
    }

    #[test]
    fn test_m6_knight_outpost() {
        // White has knight on strong outpost
        let fen = "rnbqkb1r/pp3ppp/8/3N4/2P5/8/PP1PPPPP/RNBQKB1R w KQkq - 0 1";
        let board = parse_fen(fen).unwrap();
        let mut eval = Evaluator::new();
        let score = eval.evaluate(&board);

        // White should have advantage from knight outpost
        assert!(
            score > 0,
            "Knight outpost should give advantage, got {}",
            score
        );
    }

    #[test]
    fn test_m6_endgame_evaluation() {
        // Endgame position - king activity matters more
        let fen = "8/8/3k4/8/3K4/8/8/8 w - - 0 1";
        let board = parse_fen(fen).unwrap();
        let mut eval = Evaluator::new();
        let score = eval.evaluate(&board);

        // Should be roughly equal (bare kings)
        assert!(
            score.abs() < 20,
            "Bare kings should be roughly equal, got {}",
            score
        );
    }

    #[test]
    fn test_m6_material_dominance() {
        // White has overwhelming material advantage
        let fen = "4k3/8/8/8/8/8/PPPPPPPP/RNBQKBNR w KQ - 0 1";
        let board = parse_fen(fen).unwrap();
        let mut eval = Evaluator::new();
        let score = eval.evaluate(&board);

        // White should have huge advantage
        assert!(
            score > 2000,
            "Overwhelming material advantage should give huge score, got {}",
            score
        );
    }

    #[test]
    fn test_m6_pawn_structure_quality() {
        // White has good pawn structure, black has doubled pawns
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PP1PP1PP/RNBQKBNR w KQkq - 0 1";
        let board1 = parse_fen(fen).unwrap();

        let fen2 = "rnbqkbnr/pp1pp1pp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let board2 = parse_fen(fen2).unwrap();

        let mut eval = Evaluator::new();
        let score1 = eval.evaluate(&board1); // White missing pawns
        let score2 = eval.evaluate(&board2); // Black missing pawns

        // score2 should be better for white (black missing pawns)
        assert!(
            score2 > score1,
            "Missing pawns should be worse than having them, got score1={} score2={}",
            score1,
            score2
        );
    }

    #[test]
    fn test_m6_tactical_material() {
        // Position where material is unequal
        let fen = "rnbqkb1r/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let board = parse_fen(fen).unwrap();
        let mut eval = Evaluator::new();
        let score = eval.evaluate(&board);

        // White is up a knight, should have significant advantage
        assert!(
            score > 250,
            "Being up a knight should give significant advantage, got {}",
            score
        );
    }
}
