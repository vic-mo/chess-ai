//! Advanced history heuristics for move ordering
//!
//! This module implements several advanced history tracking mechanisms:
//! - Countermove heuristic: Best refutation for each move
//! - Continuation history: Move pairs that work well together
//! - Capture history: Separate history for captures

use crate::piece::PieceType;
use crate::r#move::Move;

/// Countermove table tracks the best refutation for each move
///
/// When a move fails high, we record what move caused the cutoff.
/// Later, when the opponent makes that move again, we try the countermove first.
#[derive(Clone)]
pub struct CountermoveTable {
    /// [from_square][to_square] -> countermove
    table: [[Option<Move>; 64]; 64],
}

impl CountermoveTable {
    pub fn new() -> Self {
        Self {
            table: [[None; 64]; 64],
        }
    }

    /// Store a countermove for a given move
    ///
    /// # Arguments
    /// * `prev_move` - The move we're responding to
    /// * `countermove` - The move that refuted it
    pub fn store(&mut self, prev_move: Move, countermove: Move) {
        let from = prev_move.from().index() as usize;
        let to = prev_move.to().index() as usize;
        self.table[from][to] = Some(countermove);
    }

    /// Get the countermove for a given move
    ///
    /// # Arguments
    /// * `prev_move` - The move to get the countermove for
    ///
    /// # Returns
    /// The countermove, or None if there isn't one
    pub fn get(&self, prev_move: Move) -> Option<Move> {
        let from = prev_move.from().index() as usize;
        let to = prev_move.to().index() as usize;
        self.table[from][to]
    }

    /// Clear all countermoves
    pub fn clear(&mut self) {
        self.table = [[None; 64]; 64];
    }
}

impl Default for CountermoveTable {
    fn default() -> Self {
        Self::new()
    }
}

/// Continuation history tracks move pairs
///
/// Scores moves based on what move was played before them.
/// This captures tactical and positional patterns.
#[derive(Clone)]
pub struct ContinuationHistory {
    /// [from1][to1][from2][to2] -> score
    /// Indexed by: previous move's from/to, current move's from/to
    table: Box<[[[[i16; 64]; 64]; 64]; 64]>,
}

impl ContinuationHistory {
    pub fn new() -> Self {
        // Allocate on heap directly to avoid stack overflow
        // Create with vec and convert to box
        Self {
            table: unsafe {
                let layout = std::alloc::Layout::new::<[[[[i16; 64]; 64]; 64]; 64]>();
                let ptr = std::alloc::alloc_zeroed(layout) as *mut [[[[i16; 64]; 64]; 64]; 64];
                Box::from_raw(ptr)
            },
        }
    }

    /// Update continuation history for a move pair
    ///
    /// # Arguments
    /// * `prev_move` - The previous move
    /// * `current_move` - The current move that caused a cutoff
    /// * `depth` - Depth of the cutoff
    pub fn update(&mut self, prev_move: Move, current_move: Move, depth: i32) {
        let from1 = prev_move.from().index() as usize;
        let to1 = prev_move.to().index() as usize;
        let from2 = current_move.from().index() as usize;
        let to2 = current_move.to().index() as usize;

        // Bonus proportional to depth squared, but capped to avoid overflow
        let bonus = (depth * depth).min(400) as i16;

        // Update with saturation arithmetic
        self.table[from1][to1][from2][to2] = self.table[from1][to1][from2][to2]
            .saturating_add(bonus)
            .min(16000);

        // Age down if getting too large
        if self.table[from1][to1][from2][to2] > 16000 {
            self.age_down();
        }
    }

    /// Get continuation history score for a move pair
    ///
    /// # Arguments
    /// * `prev_move` - The previous move
    /// * `current_move` - The current move
    ///
    /// # Returns
    /// Continuation history score (0-16000)
    pub fn get(&self, prev_move: Move, current_move: Move) -> i32 {
        let from1 = prev_move.from().index() as usize;
        let to1 = prev_move.to().index() as usize;
        let from2 = current_move.from().index() as usize;
        let to2 = current_move.to().index() as usize;

        self.table[from1][to1][from2][to2] as i32
    }

    /// Age down all scores (divide by 2)
    fn age_down(&mut self) {
        for i in 0..64 {
            for j in 0..64 {
                for k in 0..64 {
                    for l in 0..64 {
                        self.table[i][j][k][l] /= 2;
                    }
                }
            }
        }
    }

    /// Clear all continuation history
    pub fn clear(&mut self) {
        // Zero out in place to avoid stack overflow
        for i in 0..64 {
            for j in 0..64 {
                for k in 0..64 {
                    for l in 0..64 {
                        self.table[i][j][k][l] = 0;
                    }
                }
            }
        }
    }
}

impl Default for ContinuationHistory {
    fn default() -> Self {
        Self::new()
    }
}

/// Capture history tracks which captures are good
///
/// Separate from regular history because captures have different characteristics.
/// Indexed by [from_square][to_square][captured_piece_type].
#[derive(Clone)]
pub struct CaptureHistory {
    /// [from][to][captured_piece] -> score
    table: [[[i16; 6]; 64]; 64],
}

impl CaptureHistory {
    pub fn new() -> Self {
        Self {
            table: [[[0; 6]; 64]; 64],
        }
    }

    /// Update capture history for a capture that caused a cutoff
    ///
    /// # Arguments
    /// * `mv` - The capture move
    /// * `captured` - The piece type that was captured
    /// * `depth` - Depth of the cutoff
    pub fn update(&mut self, mv: Move, captured: PieceType, depth: i32) {
        let from = mv.from().index() as usize;
        let to = mv.to().index() as usize;
        let piece_idx = captured as usize;

        // Bonus proportional to depth squared
        let bonus = (depth * depth).min(400) as i16;

        // Update with saturation
        self.table[from][to][piece_idx] = self.table[from][to][piece_idx]
            .saturating_add(bonus)
            .min(16000);

        // Age down if too large
        if self.table[from][to][piece_idx] > 16000 {
            self.age_down();
        }
    }

    /// Get capture history score
    ///
    /// # Arguments
    /// * `mv` - The capture move
    /// * `captured` - The piece type being captured
    ///
    /// # Returns
    /// Capture history score (0-16000)
    pub fn get(&self, mv: Move, captured: PieceType) -> i32 {
        let from = mv.from().index() as usize;
        let to = mv.to().index() as usize;
        let piece_idx = captured as usize;

        self.table[from][to][piece_idx] as i32
    }

    /// Age down all scores (divide by 2)
    fn age_down(&mut self) {
        for i in 0..64 {
            for j in 0..64 {
                for k in 0..6 {
                    self.table[i][j][k] /= 2;
                }
            }
        }
    }

    /// Clear all capture history
    pub fn clear(&mut self) {
        self.table = [[[0; 6]; 64]; 64];
    }
}

impl Default for CaptureHistory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::r#move::MoveFlags;
    use crate::square::Square;

    #[test]
    fn test_countermove_basic() {
        let mut cm = CountermoveTable::new();

        let prev = Move::new(Square::E2, Square::E4, MoveFlags::QUIET);
        let counter = Move::new(Square::E7, Square::E5, MoveFlags::QUIET);

        // Initially no countermove
        assert_eq!(cm.get(prev), None);

        // Store countermove
        cm.store(prev, counter);
        assert_eq!(cm.get(prev), Some(counter));

        // Different move should have no countermove
        let other = Move::new(Square::D2, Square::D4, MoveFlags::QUIET);
        assert_eq!(cm.get(other), None);
    }

    #[test]
    fn test_countermove_overwrite() {
        let mut cm = CountermoveTable::new();

        let prev = Move::new(Square::E2, Square::E4, MoveFlags::QUIET);
        let counter1 = Move::new(Square::E7, Square::E5, MoveFlags::QUIET);
        let counter2 = Move::new(Square::D7, Square::D5, MoveFlags::QUIET);

        cm.store(prev, counter1);
        assert_eq!(cm.get(prev), Some(counter1));

        // Overwrite with new countermove
        cm.store(prev, counter2);
        assert_eq!(cm.get(prev), Some(counter2));
    }

    #[test]
    fn test_countermove_clear() {
        let mut cm = CountermoveTable::new();

        let prev = Move::new(Square::E2, Square::E4, MoveFlags::QUIET);
        let counter = Move::new(Square::E7, Square::E5, MoveFlags::QUIET);

        cm.store(prev, counter);
        assert_eq!(cm.get(prev), Some(counter));

        cm.clear();
        assert_eq!(cm.get(prev), None);
    }

    #[test]
    fn test_continuation_history_basic() {
        let mut ch = ContinuationHistory::new();

        let prev = Move::new(Square::E2, Square::E4, MoveFlags::QUIET);
        let current = Move::new(Square::E7, Square::E5, MoveFlags::QUIET);

        // Initially 0
        assert_eq!(ch.get(prev, current), 0);

        // Update with depth 3: bonus = 9
        ch.update(prev, current, 3);
        assert_eq!(ch.get(prev, current), 9);

        // Update again with depth 5: bonus = 25
        ch.update(prev, current, 5);
        assert_eq!(ch.get(prev, current), 34);
    }

    #[test]
    fn test_continuation_history_different_pairs() {
        let mut ch = ContinuationHistory::new();

        let prev1 = Move::new(Square::E2, Square::E4, MoveFlags::QUIET);
        let prev2 = Move::new(Square::D2, Square::D4, MoveFlags::QUIET);
        let current = Move::new(Square::E7, Square::E5, MoveFlags::QUIET);

        ch.update(prev1, current, 5);
        ch.update(prev2, current, 3);

        // Different previous moves should have different scores
        assert_eq!(ch.get(prev1, current), 25);
        assert_eq!(ch.get(prev2, current), 9);
    }

    #[test]
    fn test_continuation_history_saturation() {
        let mut ch = ContinuationHistory::new();

        let prev = Move::new(Square::E2, Square::E4, MoveFlags::QUIET);
        let current = Move::new(Square::E7, Square::E5, MoveFlags::QUIET);

        // Update many times to test saturation
        for _ in 0..100 {
            ch.update(prev, current, 20); // bonus = 400 each time
        }

        let score = ch.get(prev, current);
        assert!(
            score <= 16000,
            "Score should be capped at 16000, got {}",
            score
        );
        assert!(score > 0, "Score should be positive");
    }

    #[test]
    fn test_capture_history_basic() {
        let mut ch = CaptureHistory::new();

        let capture = Move::new(Square::E4, Square::D5, MoveFlags::CAPTURE);
        let captured = PieceType::Pawn;

        // Initially 0
        assert_eq!(ch.get(capture, captured), 0);

        // Update with depth 4: bonus = 16
        ch.update(capture, captured, 4);
        assert_eq!(ch.get(capture, captured), 16);

        // Update again with depth 6: bonus = 36
        ch.update(capture, captured, 6);
        assert_eq!(ch.get(capture, captured), 52);
    }

    #[test]
    fn test_capture_history_different_pieces() {
        let mut ch = CaptureHistory::new();

        let capture = Move::new(Square::E4, Square::D5, MoveFlags::CAPTURE);

        ch.update(capture, PieceType::Pawn, 5);
        ch.update(capture, PieceType::Knight, 3);

        // Different captured pieces should have different scores
        assert_eq!(ch.get(capture, PieceType::Pawn), 25);
        assert_eq!(ch.get(capture, PieceType::Knight), 9);
        assert_eq!(ch.get(capture, PieceType::Bishop), 0);
    }

    #[test]
    fn test_capture_history_saturation() {
        let mut ch = CaptureHistory::new();

        let capture = Move::new(Square::E4, Square::D5, MoveFlags::CAPTURE);
        let captured = PieceType::Queen;

        // Update many times
        for _ in 0..100 {
            ch.update(capture, captured, 20); // bonus = 400 each time
        }

        let score = ch.get(capture, captured);
        assert!(
            score <= 16000,
            "Score should be capped at 16000, got {}",
            score
        );
        assert!(score > 0, "Score should be positive");
    }

    #[test]
    fn test_capture_history_different_squares() {
        let mut ch = CaptureHistory::new();

        let cap1 = Move::new(Square::E4, Square::D5, MoveFlags::CAPTURE);
        let cap2 = Move::new(Square::from_coords(2, 2), Square::D5, MoveFlags::CAPTURE); // c3

        ch.update(cap1, PieceType::Pawn, 5);
        ch.update(cap2, PieceType::Pawn, 3);

        // Same captured piece but different squares should be tracked separately
        assert_eq!(ch.get(cap1, PieceType::Pawn), 25);
        assert_eq!(ch.get(cap2, PieceType::Pawn), 9);
    }

    #[test]
    fn test_all_clear() {
        let mut cm = CountermoveTable::new();
        let mut ch = ContinuationHistory::new();
        let mut cap_hist = CaptureHistory::new();

        let mv1 = Move::new(Square::E2, Square::E4, MoveFlags::QUIET);
        let mv2 = Move::new(Square::E7, Square::E5, MoveFlags::QUIET);

        cm.store(mv1, mv2);
        ch.update(mv1, mv2, 5);
        cap_hist.update(mv1, PieceType::Pawn, 5);

        // Clear all
        cm.clear();
        ch.clear();
        cap_hist.clear();

        // Everything should be reset
        assert_eq!(cm.get(mv1), None);
        assert_eq!(ch.get(mv1, mv2), 0);
        assert_eq!(cap_hist.get(mv1, PieceType::Pawn), 0);
    }

    #[test]
    fn test_continuation_history_clear() {
        let mut ch = ContinuationHistory::new();

        let prev = Move::new(Square::E2, Square::E4, MoveFlags::QUIET);
        let current = Move::new(Square::E7, Square::E5, MoveFlags::QUIET);

        ch.update(prev, current, 10);
        assert!(ch.get(prev, current) > 0);

        ch.clear();
        assert_eq!(ch.get(prev, current), 0);
    }

    #[test]
    fn test_capture_history_clear() {
        let mut ch = CaptureHistory::new();

        let capture = Move::new(Square::E4, Square::D5, MoveFlags::CAPTURE);

        ch.update(capture, PieceType::Pawn, 10);
        assert!(ch.get(capture, PieceType::Pawn) > 0);

        ch.clear();
        assert_eq!(ch.get(capture, PieceType::Pawn), 0);
    }
}
