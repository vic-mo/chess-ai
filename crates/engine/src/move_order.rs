//! Move ordering for alpha-beta search optimization.
//!
//! Good move ordering is critical for alpha-beta pruning efficiency.
//! The goal is to search the best moves first to maximize cutoffs.

use crate::board::Board;
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

    /// Score a move for ordering purposes.
    ///
    /// Higher scores are searched first.
    ///
    /// # Ordering Priority
    /// 1. TT move (from transposition table) - ~10M
    /// 2. Captures (MVV-LVA) - ~1M
    /// 3. Killer moves - ~900k
    /// 4. History heuristic - 0-100k
    /// 5. Quiet moves - 0
    ///
    /// # Arguments
    /// * `board` - Current board position
    /// * `m` - Move to score
    /// * `ply` - Current search ply (for killer moves)
    /// * `tt_move` - Best move from transposition table (if any)
    pub fn score_move(&self, _board: &Board, m: Move, _ply: usize, tt_move: Option<Move>) -> i32 {
        // 1. TT move gets highest priority
        if Some(m) == tt_move {
            return 10_000_000;
        }

        // 2. Captures (placeholder - will add MVV-LVA in Session 3)
        if m.is_capture() {
            return 1_000_000;
        }

        // 3. Killer moves (will add in Sessions 4-5)
        // 4. History heuristic (will add in Sessions 6-7)

        // 5. Quiet moves get lowest priority
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
}
