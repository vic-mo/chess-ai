//! Late Move Reductions (LMR) table
//!
//! Precomputed table for LMR reductions based on depth and move count.
//! Uses the logarithmic formula: ln(depth) * ln(moves) / divisor

use std::sync::OnceLock;

/// Maximum depth for LMR table
const MAX_DEPTH: usize = 64;

/// Maximum move count for LMR table
const MAX_MOVES: usize = 64;

/// LMR table: [depth][move_count] -> reduction
static LMR_TABLE: OnceLock<[[i32; MAX_MOVES]; MAX_DEPTH]> = OnceLock::new();

/// Initialize the LMR table
///
/// Uses the formula: reduction = ln(depth) * ln(moves) / divisor
/// Divisor of 2.75 is more conservative (less aggressive reduction)
fn compute_lmr_table() -> [[i32; MAX_MOVES]; MAX_DEPTH] {
    let mut table = [[0; MAX_MOVES]; MAX_DEPTH];

    for depth in 1..MAX_DEPTH {
        for moves in 1..MAX_MOVES {
            // Modern formula: ln(depth) * ln(moves) / divisor
            // Divisor of 2.75 is more conservative than 2.0
            // Engines tune this between 2.0-3.5 depending on evaluation strength
            let reduction = ((depth as f64).ln() * (moves as f64).ln() / 2.75).round() as i32;

            // Clamp to reasonable range
            table[depth][moves] = reduction.max(0).min((depth - 1) as i32);
        }
    }

    table
}

/// Get the LMR reduction for a given depth and move count
///
/// # Arguments
/// * `depth` - Current search depth
/// * `move_count` - Number of moves searched (0-indexed)
///
/// # Returns
/// The reduction amount in plies (0 to depth-1)
#[inline]
pub fn get_reduction(depth: i32, move_count: usize) -> i32 {
    let table = LMR_TABLE.get_or_init(compute_lmr_table);

    // Clamp indices to valid range
    let d = (depth as usize).min(MAX_DEPTH - 1).max(1);
    let m = move_count.min(MAX_MOVES - 1).max(1);

    table[d][m]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lmr_basic() {
        // First move should have no reduction
        assert_eq!(get_reduction(5, 0), 0);
        assert_eq!(get_reduction(10, 0), 0);

        // Reductions should increase with move count
        let r1 = get_reduction(8, 4);
        let r2 = get_reduction(8, 8);
        let r3 = get_reduction(8, 16);

        assert!(r2 > r1, "More moves should mean more reduction");
        assert!(r3 > r2, "Even more moves should mean even more reduction");
    }

    #[test]
    fn test_lmr_depth_scaling() {
        // Reductions should increase with depth
        let r1 = get_reduction(4, 8);
        let r2 = get_reduction(8, 8);
        let r3 = get_reduction(12, 8);

        assert!(r2 > r1, "Higher depth should mean more reduction");
        assert!(r3 > r2, "Even higher depth should mean even more reduction");
    }

    #[test]
    fn test_lmr_reasonable_values() {
        // Check that reductions are reasonable at common depths

        // Depth 6, move 4: should be small reduction (1-2)
        let r = get_reduction(6, 4);
        assert!(r >= 0 && r <= 3, "Depth 6, move 4 reduction should be 0-3, got {}", r);

        // Depth 10, move 10: should be moderate reduction (2-3)
        let r = get_reduction(10, 10);
        assert!(r >= 1 && r <= 4, "Depth 10, move 10 reduction should be 1-4, got {}", r);

        // Depth 10, move 30: should be large reduction (3-5)
        let r = get_reduction(10, 30);
        assert!(r >= 2 && r <= 6, "Depth 10, move 30 reduction should be 2-6, got {}", r);
    }

    #[test]
    fn test_lmr_clamping() {
        // Reduction should never exceed depth - 1
        let depth = 5;
        for move_count in 1..50 {
            let r = get_reduction(depth, move_count);
            assert!(r < depth, "Reduction {} should be less than depth {}", r, depth);
        }
    }

    #[test]
    fn test_lmr_zero_for_low_counts() {
        // First few moves should have little to no reduction
        assert_eq!(get_reduction(5, 0), 0);
        assert_eq!(get_reduction(5, 1), 0);

        // Small move counts at low depth should have small/no reduction
        let r = get_reduction(3, 2);
        assert!(r <= 1, "Low depth, low move count should have minimal reduction");
    }

    #[test]
    fn test_lmr_table_initialized_once() {
        // Call multiple times to ensure table is only computed once
        let r1 = get_reduction(8, 10);
        let r2 = get_reduction(8, 10);
        let r3 = get_reduction(8, 10);

        assert_eq!(r1, r2);
        assert_eq!(r2, r3);
    }

    #[test]
    fn test_lmr_comparison_with_old_formula() {
        // Our old formula: if move >= 6 && depth >= 6 then 2 else 1

        // At depth 6, move 6: old = 2
        let new = get_reduction(6, 6);
        println!("Depth 6, move 6: old=2, new={}", new);
        assert!(new >= 1 && new <= 3, "Should be in reasonable range");

        // At depth 10, move 10: old = 2
        let new = get_reduction(10, 10);
        println!("Depth 10, move 10: old=2, new={}", new);
        assert!(new >= 2 && new <= 4, "Should be more aggressive than old");

        // At depth 10, move 20: old = 2
        let new = get_reduction(10, 20);
        println!("Depth 10, move 20: old=2, new={}", new);
        assert!(new >= 3 && new <= 5, "Should be much more aggressive than old");
    }
}
