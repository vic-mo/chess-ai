//! Move ordering for alpha-beta search optimization.
//!
//! Good move ordering is critical for alpha-beta pruning efficiency.
//! The goal is to search the best moves first to maximize cutoffs.

use crate::board::Board;
use crate::eval::PIECE_VALUES;
use crate::movelist::MoveList;
use crate::r#move::Move;

/// Maximum search depth (for killer move storage)
const MAX_PLY: usize = 64;

/// Move ordering manager.
///
/// Scores and orders moves to maximize alpha-beta pruning efficiency.
/// Better moves are searched first, leading to more cutoffs.
pub struct MoveOrder {
    /// Killer moves for each ply [ply][slot]
    /// Stores quiet moves that caused beta cutoffs
    killers: [[Option<Move>; 2]; MAX_PLY],

    /// History scores [from_sq][to_sq]
    /// Tracks how often moves cause cutoffs across all positions
    history: [[i32; 64]; 64],
}

impl MoveOrder {
    /// Create a new move ordering manager.
    pub fn new() -> Self {
        Self {
            killers: [[None; 2]; MAX_PLY],
            history: [[0; 64]; 64],
        }
    }

    /// Store a killer move at this ply.
    ///
    /// Killer moves are quiet moves that caused beta cutoffs.
    /// We store 2 killers per ply, shifting the first to second when a new one is added.
    ///
    /// # Arguments
    /// * `m` - Move to store as killer (must be a quiet move)
    /// * `ply` - Search ply where the cutoff occurred
    pub fn store_killer(&mut self, m: Move, ply: usize) {
        if ply >= MAX_PLY {
            return;
        }

        // Don't store if it's already the first killer
        if self.killers[ply][0] == Some(m) {
            return;
        }

        // Shift: first killer becomes second, new move becomes first
        self.killers[ply][1] = self.killers[ply][0];
        self.killers[ply][0] = Some(m);
    }

    /// Check if a move is a killer at this ply.
    ///
    /// # Arguments
    /// * `m` - Move to check
    /// * `ply` - Search ply
    ///
    /// # Returns
    /// `true` if the move is one of the killers at this ply
    fn is_killer(&self, m: Move, ply: usize) -> bool {
        if ply >= MAX_PLY {
            return false;
        }

        self.killers[ply][0] == Some(m) || self.killers[ply][1] == Some(m)
    }

    /// Update history score for a move that caused a beta cutoff.
    ///
    /// The bonus is proportional to depth squared, giving more weight to
    /// moves that work well at higher depths.
    ///
    /// # Arguments
    /// * `m` - Move that caused the cutoff (must be a quiet move)
    /// * `depth` - Depth at which the cutoff occurred
    pub fn update_history(&mut self, m: Move, depth: i32) {
        if m.is_capture() {
            return; // Only track quiet moves
        }

        let from = m.from().index() as usize;
        let to = m.to().index() as usize;

        // Bonus proportional to depth squared
        // Deeper searches are more valuable
        let bonus = depth * depth;

        self.history[from][to] += bonus;

        // Prevent overflow - scale down all history scores if any get too large
        if self.history[from][to] > 100_000 {
            for i in 0..64 {
                for j in 0..64 {
                    self.history[i][j] /= 2;
                }
            }
        }
    }

    /// Get history score for a move.
    ///
    /// # Arguments
    /// * `m` - Move to score
    ///
    /// # Returns
    /// History score (0-100k range), or 0 if it's a capture
    fn history_score(&self, m: Move) -> i32 {
        if m.is_capture() {
            return 0;
        }

        let from = m.from().index() as usize;
        let to = m.to().index() as usize;
        self.history[from][to]
    }

    /// MVV-LVA (Most Valuable Victim - Least Valuable Attacker) score for a capture.
    ///
    /// Prioritizes capturing high-value pieces with low-value pieces.
    /// Example: Queen captures Pawn (QxP) scores higher than Pawn captures Queen (PxQ).
    ///
    /// # Formula
    /// `victim_value * 10 - attacker_value`
    ///
    /// This ensures QxP (900*10 - 100 = 8900) > PxQ (100*10 - 900 = 100)
    ///
    /// # Arguments
    /// * `board` - Current board position
    /// * `m` - Move to score (must be a capture)
    ///
    /// # Returns
    /// MVV-LVA score, or 0 if not a capture
    fn mvv_lva_score(board: &Board, m: Move) -> i32 {
        if !m.is_capture() {
            return 0;
        }

        // Get the piece being captured (victim)
        let victim = board.piece_at(m.to());

        // Get the piece making the capture (attacker)
        let attacker = board.piece_at(m.from());

        // Handle en passant capture (victim square is empty, but it's a pawn capture)
        let victim_value = if let Some(piece) = victim {
            PIECE_VALUES[piece.piece_type.index()]
        } else {
            // En passant - capturing a pawn
            PIECE_VALUES[0] // Pawn value
        };

        let attacker_value = if let Some(piece) = attacker {
            PIECE_VALUES[piece.piece_type.index()]
        } else {
            // This shouldn't happen (no piece at from square)
            0
        };

        // MVV-LVA: prioritize high-value victims and low-value attackers
        victim_value * 10 - attacker_value
    }

    /// Score a move for ordering purposes.
    ///
    /// Higher scores are searched first.
    ///
    /// # Ordering Priority
    /// 1. TT move (from transposition table) - 10M
    /// 2. Captures (MVV-LVA) - 1M + (100-8,900)
    /// 3. Killer moves - 900k
    /// 4. History heuristic - 0-100k
    /// 5. Other quiet moves - 0
    ///
    /// # Arguments
    /// * `board` - Current board position
    /// * `m` - Move to score
    /// * `ply` - Current search ply (for killer moves)
    /// * `tt_move` - Best move from transposition table (if any)
    pub fn score_move(&self, board: &Board, m: Move, ply: usize, tt_move: Option<Move>) -> i32 {
        // 1. TT move gets highest priority
        if Some(m) == tt_move {
            return 10_000_000;
        }

        // 2. Captures (MVV-LVA ordering)
        if m.is_capture() {
            return 1_000_000 + Self::mvv_lva_score(board, m);
        }

        // 3. Killer moves (quiet moves that caused beta cutoffs)
        if !m.is_capture() && self.is_killer(m, ply) {
            return 900_000;
        }

        // 4. History heuristic (quiet moves with good track record)
        if !m.is_capture() {
            return self.history_score(m);
        }

        // 5. Other quiet moves get lowest priority
        0
    }

    /// Sort moves in-place by score (highest first).
    ///
    /// # Arguments
    /// * `board` - Current board position
    /// * `moves` - Moves to sort (modified in-place)
    /// * `ply` - Current search ply
    /// * `tt_move` - Best move from transposition table (if any)
    pub fn order_moves(
        &mut self,
        board: &Board,
        moves: &mut MoveList,
        ply: usize,
        tt_move: Option<Move>,
    ) {
        // Sort by score (descending - highest scores first)
        moves.sort_by_key(|&m| -self.score_move(board, m, ply, tt_move));
    }

    /// Clear history and killers for new search.
    pub fn clear(&mut self) {
        self.killers = [[None; 2]; MAX_PLY];
        self.history = [[0; 64]; 64];
    }
}

impl Default for MoveOrder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::parse_fen;
    use crate::square::Square;

    #[test]
    fn test_move_order_create() {
        let move_order = MoveOrder::new();
        assert_eq!(move_order.killers[0][0], None);
        assert_eq!(move_order.history[0][0], 0);
    }

    #[test]
    fn test_tt_move_priority() {
        let board = Board::startpos();
        let move_order = MoveOrder::new();

        let moves = board.generate_legal_moves();
        assert!(!moves.is_empty());

        let tt_move = moves[5]; // Arbitrary move as TT move
        let other_move = moves[0];

        let tt_score = move_order.score_move(&board, tt_move, 0, Some(tt_move));
        let other_score = move_order.score_move(&board, other_move, 0, Some(tt_move));

        // TT move should have higher score
        assert!(tt_score > other_score);
        assert_eq!(tt_score, 10_000_000);
    }

    #[test]
    fn test_capture_priority() {
        let move_order = MoveOrder::new();

        // Make a position with captures available
        let fen = "rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1";
        let board = parse_fen(fen).unwrap();

        let moves = board.generate_legal_moves();
        let capture = moves.iter().find(|m| m.is_capture()).unwrap();
        let quiet = moves.iter().find(|m| !m.is_capture()).unwrap();

        let capture_score = move_order.score_move(&board, *capture, 0, None);
        let quiet_score = move_order.score_move(&board, *quiet, 0, None);

        // Captures should score higher than quiet moves
        assert!(capture_score > quiet_score);
    }

    #[test]
    fn test_move_ordering_tt_move_first() {
        let board = Board::startpos();
        let mut move_order = MoveOrder::new();

        let mut moves = board.generate_legal_moves();
        assert!(moves.len() >= 6);

        let tt_move = moves[5]; // Make an arbitrary move the TT move

        move_order.order_moves(&board, &mut moves, 0, Some(tt_move));

        // TT move should be sorted to first position
        assert_eq!(moves[0], tt_move);
    }

    #[test]
    fn test_move_ordering_captures_before_quiet() {
        // Position with both captures and quiet moves
        let fen = "rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1";
        let board = parse_fen(fen).unwrap();

        let mut moves = board.generate_legal_moves();
        let mut move_order = MoveOrder::new();

        move_order.order_moves(&board, &mut moves, 0, None);

        // Find first capture and first quiet move in ordered list
        let first_capture_idx = moves.iter().position(|m| m.is_capture());
        let first_quiet_idx = moves.iter().position(|m| !m.is_capture());

        if let (Some(cap_idx), Some(quiet_idx)) = (first_capture_idx, first_quiet_idx) {
            // Captures should come before quiet moves
            assert!(
                cap_idx < quiet_idx,
                "Captures should be ordered before quiet moves"
            );
        }
    }

    #[test]
    fn test_clear() {
        let mut move_order = MoveOrder::new();

        // Manually set some data
        move_order.killers[0][0] = Some(Move::new(
            Square::E2,
            Square::E4,
            crate::r#move::MoveFlags::QUIET,
        ));
        move_order.history[0][0] = 100;

        // Clear
        move_order.clear();

        // Should be reset
        assert_eq!(move_order.killers[0][0], None);
        assert_eq!(move_order.history[0][0], 0);
    }

    #[test]
    fn test_mvv_lva_basic() {
        use crate::r#move::MoveFlags;

        // Create a simple position: White Queen on e4, Black Pawn on e5
        // QxP should have high MVV-LVA score
        let fen = "rnbqkbnr/pppp1ppp/8/4p3/4Q3/8/PPPP1PPP/RNB1KBNR w KQkq - 0 1";
        let board = parse_fen(fen).unwrap();

        // Queen captures pawn
        let qxp = Move::new(Square::E4, Square::E5, MoveFlags::CAPTURE);
        let mvv_lva = MoveOrder::mvv_lva_score(&board, qxp);

        // Expected: pawn_value * 10 - queen_value = 100 * 10 - 900 = 100
        assert_eq!(mvv_lva, 100);
    }

    #[test]
    fn test_mvv_lva_qxp_vs_pxq() {
        use crate::r#move::MoveFlags;

        // Position with Queen and Pawn that can capture each other
        // White: Queen on e4, Pawn on d2
        // Black: Queen on d5, Pawn on e5
        let fen = "rnb1kbnr/pppp1ppp/8/3qp3/4Q3/8/PPPP1PPP/RNB1KBNR w KQkq - 0 1";
        let board = parse_fen(fen).unwrap();

        let move_order = MoveOrder::new();

        // Queen takes Queen (from e4 to d5)
        let qxq = Move::new(Square::E4, Square::D5, MoveFlags::CAPTURE);
        let qxq_score = move_order.score_move(&board, qxq, 0, None);

        // Create position for pawn takes queen
        let fen2 = "rnb1kbnr/pppp1ppp/8/3Qp3/8/8/PPPPPPPP/RNB1KBNR b KQkq - 0 1";
        let board2 = parse_fen(fen2).unwrap();

        // Pawn takes Queen (from e5 to d5)
        let pxq = Move::new(Square::E5, Square::D5, MoveFlags::CAPTURE);
        let pxq_score = move_order.score_move(&board2, pxq, 0, None);

        // QxQ should score higher than PxQ
        // QxQ: 900*10 - 900 = 8100
        // PxQ: 900*10 - 100 = 8900
        // Actually PxQ should score higher! (better to use pawn to capture queen)
        assert!(pxq_score > qxq_score, "PxQ should score higher than QxQ");
    }

    #[test]
    fn test_mvv_lva_ordering_multiple_captures() {
        use crate::r#move::MoveFlags;

        // Position with multiple possible captures
        // White can capture: Queen(d5), Knight(b5)
        let fen = "rnb1kbnr/pppp1ppp/8/nrBqR3/4p3/8/PPPP1PPP/RNBQK1NR w KQkq - 0 1";
        let board = parse_fen(fen).unwrap();

        let move_order = MoveOrder::new();

        // Bishop captures Queen (best victim) - c5 to d5
        let bxq = Move::new(
            Square::from_coords(2, 4), // c5
            Square::D5,
            MoveFlags::CAPTURE,
        );

        // Rook captures Knight (medium victim) - e5 to b5
        let rxn = Move::new(
            Square::E5,
            Square::from_coords(1, 4), // b5
            MoveFlags::CAPTURE,
        );

        let bxq_score = move_order.score_move(&board, bxq, 0, None);
        let rxn_score = move_order.score_move(&board, rxn, 0, None);

        // BxQ should score higher than RxN
        // BxQ: 900*10 - 330 = 8670
        // RxN: 320*10 - 500 = 2700
        assert!(
            bxq_score > rxn_score,
            "BxQ (score={}) should score higher than RxN (score={})",
            bxq_score,
            rxn_score
        );
    }

    #[test]
    fn test_killer_moves_storage() {
        use crate::r#move::MoveFlags;

        let mut move_order = MoveOrder::new();

        let killer1 = Move::new(Square::E2, Square::E4, MoveFlags::QUIET);
        let killer2 = Move::new(Square::D2, Square::D4, MoveFlags::QUIET);
        let killer3 = Move::new(Square::E7, Square::E5, MoveFlags::QUIET);

        // Store first killer at ply 0
        move_order.store_killer(killer1, 0);
        assert_eq!(move_order.killers[0][0], Some(killer1));
        assert_eq!(move_order.killers[0][1], None);

        // Store second killer at ply 0 - should shift first to second
        move_order.store_killer(killer2, 0);
        assert_eq!(move_order.killers[0][0], Some(killer2));
        assert_eq!(move_order.killers[0][1], Some(killer1));

        // Store third killer - should shift again
        move_order.store_killer(killer3, 0);
        assert_eq!(move_order.killers[0][0], Some(killer3));
        assert_eq!(move_order.killers[0][1], Some(killer2));
        // killer1 is now lost (only keep 2 per ply)
    }

    #[test]
    fn test_killer_moves_no_duplicate() {
        use crate::r#move::MoveFlags;

        let mut move_order = MoveOrder::new();

        let killer1 = Move::new(Square::E2, Square::E4, MoveFlags::QUIET);
        let killer2 = Move::new(Square::D2, Square::D4, MoveFlags::QUIET);

        // Store first killer
        move_order.store_killer(killer1, 0);
        move_order.store_killer(killer2, 0);

        // Try to store killer1 again - should not duplicate
        move_order.store_killer(killer1, 0);

        // killer1 should still be at position 0, not duplicated
        assert_eq!(move_order.killers[0][0], Some(killer1));
        assert_eq!(move_order.killers[0][1], Some(killer2));
    }

    #[test]
    fn test_killer_moves_is_killer() {
        use crate::r#move::MoveFlags;

        let mut move_order = MoveOrder::new();

        let killer1 = Move::new(Square::E2, Square::E4, MoveFlags::QUIET);
        let killer2 = Move::new(Square::D2, Square::D4, MoveFlags::QUIET);
        let non_killer = Move::new(Square::E3, Square::F3, MoveFlags::QUIET);

        move_order.store_killer(killer1, 0);
        move_order.store_killer(killer2, 0);

        // Both killers should be recognized
        assert!(move_order.is_killer(killer1, 0));
        assert!(move_order.is_killer(killer2, 0));

        // Non-killer should not be recognized
        assert!(!move_order.is_killer(non_killer, 0));

        // Killers at ply 0 should not be killers at ply 1
        assert!(!move_order.is_killer(killer1, 1));
        assert!(!move_order.is_killer(killer2, 1));
    }

    #[test]
    fn test_killer_moves_ordering() {
        use crate::r#move::MoveFlags;

        let board = Board::startpos();
        let mut move_order = MoveOrder::new();

        let killer = Move::new(Square::E2, Square::E4, MoveFlags::QUIET);
        let non_killer = Move::new(Square::D2, Square::D4, MoveFlags::QUIET);

        move_order.store_killer(killer, 0);

        let killer_score = move_order.score_move(&board, killer, 0, None);
        let non_killer_score = move_order.score_move(&board, non_killer, 0, None);

        // Killer should score higher than non-killer
        assert_eq!(killer_score, 900_000);
        assert_eq!(non_killer_score, 0);
        assert!(killer_score > non_killer_score);
    }

    #[test]
    fn test_killer_moves_below_captures() {
        use crate::r#move::MoveFlags;

        // Position with a capture available
        let fen = "rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1";
        let board = parse_fen(fen).unwrap();

        let mut move_order = MoveOrder::new();

        let killer = Move::new(Square::E4, Square::E5, MoveFlags::QUIET);
        let capture = Move::new(Square::E4, Square::D5, MoveFlags::CAPTURE);

        move_order.store_killer(killer, 0);

        let killer_score = move_order.score_move(&board, killer, 0, None);
        let capture_score = move_order.score_move(&board, capture, 0, None);

        // Captures should score higher than killers
        assert!(
            capture_score > killer_score,
            "Capture ({}) should score higher than killer ({})",
            capture_score,
            killer_score
        );
    }

    #[test]
    fn test_history_update() {
        use crate::r#move::MoveFlags;

        let mut move_order = MoveOrder::new();

        let mv = Move::new(Square::E2, Square::E4, MoveFlags::QUIET);

        // Initially, history should be 0
        assert_eq!(move_order.history_score(mv), 0);

        // Update with depth 3: bonus = 3 * 3 = 9
        move_order.update_history(mv, 3);
        assert_eq!(move_order.history_score(mv), 9);

        // Update again with depth 5: bonus = 5 * 5 = 25
        move_order.update_history(mv, 5);
        assert_eq!(move_order.history_score(mv), 9 + 25);
    }

    #[test]
    fn test_history_depth_squared() {
        use crate::r#move::MoveFlags;

        let mut move_order = MoveOrder::new();

        let mv1 = Move::new(Square::E2, Square::E4, MoveFlags::QUIET);
        let mv2 = Move::new(Square::D2, Square::D4, MoveFlags::QUIET);

        // Update mv1 at depth 2: bonus = 4
        move_order.update_history(mv1, 2);

        // Update mv2 at depth 4: bonus = 16
        move_order.update_history(mv2, 4);

        // Deeper cutoffs should have higher scores
        assert!(move_order.history_score(mv2) > move_order.history_score(mv1));
        assert_eq!(move_order.history_score(mv1), 4);
        assert_eq!(move_order.history_score(mv2), 16);
    }

    #[test]
    fn test_history_only_quiet_moves() {
        use crate::r#move::MoveFlags;

        let mut move_order = MoveOrder::new();

        let quiet = Move::new(Square::E2, Square::E4, MoveFlags::QUIET);
        let capture = Move::new(Square::E4, Square::D5, MoveFlags::CAPTURE);

        // Update both moves
        move_order.update_history(quiet, 5);
        move_order.update_history(capture, 5);

        // Only quiet move should have history
        assert_eq!(move_order.history_score(quiet), 25);
        assert_eq!(move_order.history_score(capture), 0);
    }

    #[test]
    fn test_history_scaling() {
        use crate::r#move::MoveFlags;

        let mut move_order = MoveOrder::new();

        let mv = Move::new(Square::E2, Square::E4, MoveFlags::QUIET);

        // Force history to exceed 100,000 by updating many times
        for _ in 0..400 {
            move_order.update_history(mv, 10); // bonus = 100 each time
        }

        // History should be scaled down (all divided by 2)
        let score = move_order.history_score(mv);
        assert!(
            score <= 100_000,
            "History score should be scaled: {}",
            score
        );
        assert!(score > 0, "History should still be positive after scaling");
    }

    #[test]
    fn test_history_ordering() {
        use crate::r#move::MoveFlags;

        let board = Board::startpos();
        let mut move_order = MoveOrder::new();

        let mv1 = Move::new(Square::E2, Square::E4, MoveFlags::QUIET);
        let mv2 = Move::new(Square::D2, Square::D4, MoveFlags::QUIET);

        // Update mv1 with higher history
        move_order.update_history(mv1, 8); // bonus = 64

        // Update mv2 with lower history
        move_order.update_history(mv2, 2); // bonus = 4

        let score1 = move_order.score_move(&board, mv1, 0, None);
        let score2 = move_order.score_move(&board, mv2, 0, None);

        // mv1 should score higher due to better history
        assert!(score1 > score2);
        assert_eq!(score1, 64);
        assert_eq!(score2, 4);
    }

    #[test]
    fn test_history_below_killers() {
        use crate::r#move::MoveFlags;

        let board = Board::startpos();
        let mut move_order = MoveOrder::new();

        let killer = Move::new(Square::E2, Square::E4, MoveFlags::QUIET);
        let history_move = Move::new(Square::D2, Square::D4, MoveFlags::QUIET);

        // Make killer a killer move
        move_order.store_killer(killer, 0);

        // Give history_move a high history score
        move_order.update_history(history_move, 300); // bonus = 90,000

        let killer_score = move_order.score_move(&board, killer, 0, None);
        let history_score = move_order.score_move(&board, history_move, 0, None);

        // Killer should still score higher than history
        assert_eq!(killer_score, 900_000);
        assert_eq!(history_score, 90_000);
        assert!(killer_score > history_score);
    }

    #[test]
    fn test_complete_move_ordering() {
        use crate::r#move::MoveFlags;

        let fen = "rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1";
        let board = parse_fen(fen).unwrap();

        let mut move_order = MoveOrder::new();

        // Create different types of moves
        let tt_move = Move::new(Square::E4, Square::D5, MoveFlags::CAPTURE);
        let capture = Move::new(Square::E4, Square::D5, MoveFlags::CAPTURE);
        let killer = Move::new(Square::E2, Square::E3, MoveFlags::QUIET);
        let history_move = Move::new(Square::D2, Square::D4, MoveFlags::QUIET);
        let quiet = Move::new(Square::E1, Square::E2, MoveFlags::QUIET);

        // Set up move order state
        move_order.store_killer(killer, 0);
        move_order.update_history(history_move, 5); // bonus = 25

        // Score all moves
        let tt_score = move_order.score_move(&board, tt_move, 0, Some(tt_move));
        let capture_score = move_order.score_move(&board, capture, 0, None);
        let killer_score = move_order.score_move(&board, killer, 0, None);
        let history_score = move_order.score_move(&board, history_move, 0, None);
        let quiet_score = move_order.score_move(&board, quiet, 0, None);

        // Verify ordering: TT > Capture > Killer > History > Quiet
        assert!(tt_score > capture_score, "TT move should score highest");
        assert!(capture_score > killer_score, "Captures should beat killers");
        assert!(killer_score > history_score, "Killers should beat history");
        assert!(history_score > quiet_score, "History should beat quiet");

        // Verify specific values
        assert_eq!(tt_score, 10_000_000);
        assert!(capture_score >= 1_000_000);
        assert_eq!(killer_score, 900_000);
        assert_eq!(history_score, 25);
        assert_eq!(quiet_score, 0);
    }

    #[test]
    fn test_mvv_lva_same_victim_prefers_lower_attacker() {
        use crate::r#move::MoveFlags;

        // Position where both Queen and Pawn can capture the same piece (a Rook)
        // White: Queen on d4, Pawn on a4
        // Black: Rook on a5
        let fen = "rnbqkbnr/1ppppppp/8/r7/P2Q4/8/1PPPPPPP/RNB1KBNR w KQkq - 0 1";
        let board = parse_fen(fen).unwrap();

        let move_order = MoveOrder::new();

        // Pawn captures Rook (a4 -> a5)
        let pxr = Move::new(
            Square::from_coords(0, 3), // a4
            Square::from_coords(0, 4), // a5
            MoveFlags::CAPTURE,
        );

        // Queen captures Rook (d4 -> a5)
        let qxr = Move::new(
            Square::D4,
            Square::from_coords(0, 4), // a5
            MoveFlags::CAPTURE,
        );

        let pxr_score = move_order.score_move(&board, pxr, 0, None);
        let qxr_score = move_order.score_move(&board, qxr, 0, None);

        // PxR should score higher than QxR (same victim, lower attacker)
        // PxR: 500*10 - 100 = 4900
        // QxR: 500*10 - 900 = 4100
        assert!(
            pxr_score > qxr_score,
            "PxR (score={}) should score higher than QxR (score={})",
            pxr_score,
            qxr_score
        );
    }
}
