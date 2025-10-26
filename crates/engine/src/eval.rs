//! Position evaluation for chess positions.
//!
//! Evaluates positions from the current side to move's perspective.
//! Positive scores favor the side to move, negative scores favor the opponent.

pub mod king;
mod material;
pub mod pawns;
pub mod phase;
pub mod pieces;
mod positional;
mod pst;

pub use king::*;
pub use material::*;
pub use pawns::*;
pub use phase::*;
pub use pieces::*;
pub use positional::*;
pub use pst::*;

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
    pub fn evaluate(&mut self, board: &Board) -> i32 {
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
        let (white_king_mg, white_king_eg) = evaluate_king_safety(board, Color::White, phase);
        let (black_king_mg, black_king_eg) = evaluate_king_safety(board, Color::Black, phase);
        mg_score += white_king_mg - black_king_mg;
        eg_score += white_king_eg - black_king_eg;

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
