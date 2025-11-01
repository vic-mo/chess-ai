//! Advanced pruning techniques for search optimization
//!
//! Pruning reduces the search tree by skipping branches unlikely to improve the score.
//! This module implements various safe pruning techniques:
//! - Futility pruning: Skip quiet moves when eval + margin < alpha
//! - Reverse futility pruning: Cut node when eval - margin >= beta
//! - Razoring: Drop into qsearch when position is hopeless
//! - Late move pruning: Skip late quiet moves at shallow depths
//! - SEE pruning: Skip bad captures
//! - Multi-cut pruning: Cut node when multiple moves fail high
//! - Probcut: Cut node when shallow search proves beta cutoff

use crate::board::Board;
use crate::r#move::Move;
use crate::search_params;

/// Futility pruning margins by depth
/// Index by depth (0, 1, 2, 3)
pub const FUTILITY_MARGINS: [i32; 4] = [0, 100, 200, 300];

/// Reverse futility pruning margins by depth
/// Index by depth (0, 1, 2, 3, 4, 5)
pub const RFP_MARGINS: [i32; 6] = [0, 100, 200, 300, 400, 500];

/// Razoring margins by depth
/// Index by depth (0, 1, 2, 3)
pub const RAZOR_MARGINS: [i32; 4] = [0, 200, 300, 400];

/// Late move pruning thresholds by depth
/// Number of moves to search before pruning
/// Index by depth (0, 1, 2, 3)
pub const LMP_THRESHOLDS: [usize; 4] = [0, 3, 6, 12];

/// SEE threshold for captures in main search
/// Conservative: only prune clearly losing captures
pub const SEE_QUIET_THRESHOLD: i32 = -10;

/// SEE threshold for captures in qsearch
/// More aggressive: can prune slightly losing captures
pub const SEE_CAPTURE_THRESHOLD: i32 = -10;

/// Probcut margin (how much higher than beta for probcut)
pub const PROBCUT_MARGIN: i32 = 200;

/// Probcut depth reduction
pub const PROBCUT_DEPTH_REDUCTION: i32 = 4;

/// Multi-cut threshold (number of cutoffs needed)
pub const MULTI_CUT_THRESHOLD: usize = 3;

/// Multi-cut depth reduction
pub const MULTI_CUT_DEPTH_REDUCTION: i32 = 3;

/// Check if futility pruning can be applied
///
/// Futility pruning skips quiet moves at shallow depths when the static
/// evaluation plus a margin is still below alpha.
///
/// # Safety conditions (must all be true):
/// - Depth <= 3
/// - Not in check
/// - Not a PV node
/// - eval + margin < alpha
///
/// # Arguments
/// * `depth` - Current search depth
/// * `in_check` - Whether current side is in check
/// * `is_pv` - Whether this is a PV node
/// * `eval` - Static evaluation of current position
/// * `alpha` - Alpha bound
///
/// # Returns
/// true if futility pruning can skip quiet moves
pub fn can_futility_prune(depth: i32, in_check: bool, is_pv: bool, eval: i32, alpha: i32) -> bool {
    if depth > 3 || in_check || is_pv {
        return false;
    }

    let params = search_params::get_search_params();
    let margin = match depth {
        1 => params.futility_margin_d1,
        2 => params.futility_margin_d2,
        3 => params.futility_margin_d3,
        _ => 0,
    };
    eval + margin < alpha
}

/// Check if reverse futility pruning can be applied
///
/// RFP cuts the node early when the static evaluation minus a margin
/// is still above beta, suggesting all moves will fail high.
///
/// # Safety conditions (must all be true):
/// - Depth <= 5
/// - Not in check
/// - Not a PV node
/// - eval - margin >= beta
/// - Not a mate score
///
/// # Arguments
/// * `depth` - Current search depth
/// * `in_check` - Whether current side is in check
/// * `is_pv` - Whether this is a PV node
/// * `eval` - Static evaluation of current position
/// * `beta` - Beta bound
///
/// # Returns
/// (can_prune, pruning_score)
pub fn can_reverse_futility_prune(
    depth: i32,
    in_check: bool,
    is_pv: bool,
    eval: i32,
    beta: i32,
) -> (bool, i32) {
    if depth > 5 || in_check || is_pv {
        return (false, 0);
    }

    // Don't apply RFP near mate scores
    if eval.abs() > 10000 {
        return (false, 0);
    }

    let params = search_params::get_search_params();
    let margin = match depth {
        1 => params.rfp_margin_d1,
        2 => params.rfp_margin_d2,
        3 => params.rfp_margin_d3,
        4 => params.rfp_margin_d4,
        5 => params.rfp_margin_d5,
        _ => 0,
    };
    if eval - margin >= beta {
        (true, eval)
    } else {
        (false, 0)
    }
}

/// Check if razoring can be applied
///
/// Razoring drops into qsearch when the position appears hopeless
/// even with a margin added to the evaluation.
///
/// # Safety conditions (must all be true):
/// - Depth <= 3
/// - Not in check
/// - Not a PV node
/// - eval + margin < alpha
///
/// # Arguments
/// * `depth` - Current search depth
/// * `in_check` - Whether current side is in check
/// * `is_pv` - Whether this is a PV node
/// * `eval` - Static evaluation of current position
/// * `alpha` - Alpha bound
///
/// # Returns
/// true if razoring should be attempted
pub fn can_razor(depth: i32, in_check: bool, is_pv: bool, eval: i32, alpha: i32) -> bool {
    if !(1..=3).contains(&depth) || in_check || is_pv {
        return false;
    }

    let params = search_params::get_search_params();
    let margin = match depth {
        1 => params.razor_margin_d1,
        2 => params.razor_margin_d2,
        3 => params.razor_margin_d3,
        _ => 0,
    };
    eval + margin < alpha
}

/// Check if late move pruning can be applied
///
/// LMP skips remaining quiet moves after searching a certain number
/// of moves at shallow depths.
///
/// # Safety conditions (must all be true):
/// - Depth <= 3
/// - Not in check
/// - Move count exceeds threshold
/// - Move is quiet (not tactical)
///
/// # Arguments
/// * `depth` - Current search depth
/// * `in_check` - Whether current side is in check
/// * `move_count` - Number of moves searched so far
/// * `mv` - The move to check
///
/// # Returns
/// true if this move should be pruned
pub fn can_late_move_prune(depth: i32, in_check: bool, move_count: usize, mv: Move) -> bool {
    if depth > 3 || in_check {
        return false;
    }

    // Only prune quiet moves
    if mv.is_capture() || mv.is_promotion() {
        return false;
    }

    let params = search_params::get_search_params();
    let threshold = match depth {
        1 => params.lmp_threshold_d1,
        2 => params.lmp_threshold_d2,
        3 => params.lmp_threshold_d3,
        _ => 99,
    };
    move_count > threshold
}

/// Check if a move should be pruned based on SEE
///
/// SEE pruning skips moves that lose material according to
/// static exchange evaluation.
///
/// # Arguments
/// * `board` - Current board position
/// * `mv` - The move to check
/// * `in_qsearch` - Whether we're in quiescence search
///
/// # Returns
/// true if this move should be pruned
pub fn can_see_prune(board: &Board, mv: Move, in_qsearch: bool) -> bool {
    if !mv.is_capture() {
        return false;
    }

    let threshold = if in_qsearch {
        SEE_CAPTURE_THRESHOLD
    } else {
        SEE_QUIET_THRESHOLD
    };

    // Use SEE from our see module
    !crate::search::see::see(board, mv, threshold)
}

/// Multi-cut pruning detection
///
/// If we find M >= 3 moves that fail high at reduced depth,
/// we can cut the node assuming a real fail-high would occur.
///
/// This is implemented during search by tracking cutoff count.
///
/// # Arguments
/// * `cutoff_count` - Number of moves that caused beta cutoff
///
/// # Returns
/// true if multi-cut can be applied
pub fn can_multi_cut(cutoff_count: usize) -> bool {
    cutoff_count >= MULTI_CUT_THRESHOLD
}

/// Probcut detection
///
/// If a shallow search proves the score is much higher than beta,
/// we can safely cut the node.
///
/// # Arguments
/// * `depth` - Current search depth
/// * `is_pv` - Whether this is a PV node
///
/// # Returns
/// true if probcut should be attempted
pub fn should_try_probcut(depth: i32, is_pv: bool) -> bool {
    depth >= 5 && !is_pv
}

/// Calculate probcut beta
///
/// # Arguments
/// * `beta` - Original beta bound
///
/// # Returns
/// Adjusted beta for probcut search
pub fn probcut_beta(beta: i32) -> i32 {
    beta + PROBCUT_MARGIN
}

/// Check if position is in endgame (affects pruning decisions)
///
/// # Arguments
/// * `board` - Current board position
///
/// # Returns
/// true if in endgame
pub fn is_endgame(_board: &Board) -> bool {
    // Simple heuristic: endgame if no queens or very few pieces
    // This will be refined during integration
    // For now, return false to keep pruning active
    false
}

/// Pruning safety check
///
/// Some conditions universally disable all forward pruning:
/// - In check (must search all moves)
/// - PV nodes (need accurate scores)
/// - Mate scores in TT (critical lines)
///
/// # Arguments
/// * `in_check` - Whether in check
/// * `is_pv` - Whether PV node
/// * `tt_score` - Score from transposition table (if any)
///
/// # Returns
/// true if pruning is safe
pub fn is_pruning_safe(in_check: bool, is_pv: bool, tt_score: Option<i32>) -> bool {
    if in_check || is_pv {
        return false;
    }

    // Check for mate scores in TT
    if let Some(score) = tt_score {
        if score.abs() > 10000 {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::parse_fen;
    use crate::movegen::generate_moves;
    use crate::r#move::MoveFlags;
    use crate::square::Square;

    #[test]
    fn test_futility_pruning_depth() {
        // Should work at shallow depths
        assert!(can_futility_prune(1, false, false, -200, 0));
        assert!(can_futility_prune(2, false, false, -300, 0));
        assert!(can_futility_prune(3, false, false, -400, 0));

        // Should not work at higher depths
        assert!(!can_futility_prune(4, false, false, -500, 0));
        assert!(!can_futility_prune(5, false, false, -600, 0));
    }

    #[test]
    fn test_futility_pruning_conditions() {
        // Should not work in check
        assert!(!can_futility_prune(1, true, false, -200, 0));

        // Should not work in PV
        assert!(!can_futility_prune(1, false, true, -200, 0));

        // Should not work if eval + margin >= alpha
        assert!(!can_futility_prune(1, false, false, 50, 0));
    }

    #[test]
    fn test_reverse_futility_pruning() {
        // Should work when eval - margin >= beta
        // depth=3, margin=300, so need eval >= beta + 300 = 300 + 300 = 600
        let (can_prune, score) = can_reverse_futility_prune(3, false, false, 700, 300);
        assert!(can_prune);
        assert_eq!(score, 700);

        // Should not work when eval - margin < beta
        let (can_prune, _) = can_reverse_futility_prune(3, false, false, 500, 300);
        assert!(!can_prune);

        // Should not work in check
        let (can_prune, _) = can_reverse_futility_prune(3, true, false, 700, 300);
        assert!(!can_prune);

        // Should not work in PV
        let (can_prune, _) = can_reverse_futility_prune(3, false, true, 700, 300);
        assert!(!can_prune);

        // Should not work at high depth
        let (can_prune, _) = can_reverse_futility_prune(6, false, false, 700, 300);
        assert!(!can_prune);
    }

    #[test]
    fn test_reverse_futility_mate_scores() {
        // Should not apply near mate scores
        let (can_prune, _) = can_reverse_futility_prune(3, false, false, 15000, 300);
        assert!(!can_prune);

        let (can_prune, _) = can_reverse_futility_prune(3, false, false, -15000, 300);
        assert!(!can_prune);
    }

    #[test]
    fn test_razoring() {
        // Should work when eval + margin < alpha
        // depth=2, margin=300, so need eval < -300 for eval+margin < 0
        assert!(can_razor(2, false, false, -400, 0));
        assert!(can_razor(1, false, false, -250, 0));

        // Should not work when eval + margin >= alpha
        assert!(!can_razor(2, false, false, -300, 0)); // -300 + 300 = 0, not < 0

        // Should not work in check
        assert!(!can_razor(2, true, false, -400, 0));

        // Should not work in PV
        assert!(!can_razor(2, false, true, -400, 0));

        // Should not work at depth 0 or > 3
        assert!(!can_razor(0, false, false, -400, 0));
        assert!(!can_razor(4, false, false, -400, 0));
    }

    #[test]
    fn test_late_move_pruning() {
        let quiet_move = Move::new(
            Square::from_algebraic("e2").unwrap(),
            Square::from_algebraic("e4").unwrap(),
            MoveFlags::QUIET,
        );

        // Should prune late quiet moves
        assert!(can_late_move_prune(1, false, 5, quiet_move));
        assert!(can_late_move_prune(2, false, 10, quiet_move));

        // Should not prune early moves
        assert!(!can_late_move_prune(1, false, 2, quiet_move));
        assert!(!can_late_move_prune(2, false, 4, quiet_move));

        // Should not prune in check
        assert!(!can_late_move_prune(1, true, 10, quiet_move));
    }

    #[test]
    fn test_late_move_pruning_tactical() {
        let capture = Move::new(
            Square::from_algebraic("e4").unwrap(),
            Square::from_algebraic("d5").unwrap(),
            MoveFlags::CAPTURE,
        );

        let promotion = Move::new(
            Square::from_algebraic("e7").unwrap(),
            Square::from_algebraic("e8").unwrap(),
            MoveFlags::QUEEN_PROMOTION,
        );

        // Should not prune captures or promotions
        assert!(!can_late_move_prune(1, false, 10, capture));
        assert!(!can_late_move_prune(1, false, 10, promotion));
    }

    #[test]
    fn test_see_pruning() {
        // Test with a position where we can evaluate captures
        let board =
            parse_fen("rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 2").unwrap();

        let moves = generate_moves(&board);
        let capture = moves
            .iter()
            .find(|m| m.from().to_string() == "e4" && m.to().to_string() == "d5")
            .unwrap();

        // Good capture should not be pruned
        assert!(!can_see_prune(&board, *capture, false));
    }

    #[test]
    fn test_multi_cut() {
        // Should trigger with enough cutoffs
        assert!(can_multi_cut(3));
        assert!(can_multi_cut(4));
        assert!(can_multi_cut(5));

        // Should not trigger with too few
        assert!(!can_multi_cut(0));
        assert!(!can_multi_cut(1));
        assert!(!can_multi_cut(2));
    }

    #[test]
    fn test_probcut_conditions() {
        // Should work at high depth, non-PV
        assert!(should_try_probcut(5, false));
        assert!(should_try_probcut(10, false));

        // Should not work at low depth
        assert!(!should_try_probcut(4, false));
        assert!(!should_try_probcut(3, false));

        // Should not work in PV
        assert!(!should_try_probcut(5, true));
    }

    #[test]
    fn test_probcut_beta() {
        assert_eq!(probcut_beta(100), 300);
        assert_eq!(probcut_beta(0), 200);
        assert_eq!(probcut_beta(-100), 100);
    }

    #[test]
    fn test_pruning_safety() {
        // Safe in normal positions
        assert!(is_pruning_safe(false, false, None));
        assert!(is_pruning_safe(false, false, Some(100)));

        // Not safe in check
        assert!(!is_pruning_safe(true, false, None));

        // Not safe in PV
        assert!(!is_pruning_safe(false, true, None));

        // Not safe with mate scores
        assert!(!is_pruning_safe(false, false, Some(15000)));
        assert!(!is_pruning_safe(false, false, Some(-15000)));
    }

    #[test]
    fn test_pruning_margins() {
        // Check futility margins increase with depth
        assert!(FUTILITY_MARGINS[1] < FUTILITY_MARGINS[2]);
        assert!(FUTILITY_MARGINS[2] < FUTILITY_MARGINS[3]);

        // Check RFP margins increase with depth
        assert!(RFP_MARGINS[1] < RFP_MARGINS[2]);
        assert!(RFP_MARGINS[2] < RFP_MARGINS[3]);

        // Check razor margins increase with depth
        assert!(RAZOR_MARGINS[1] < RAZOR_MARGINS[2]);
        assert!(RAZOR_MARGINS[2] < RAZOR_MARGINS[3]);

        // Check LMP thresholds increase with depth
        assert!(LMP_THRESHOLDS[1] < LMP_THRESHOLDS[2]);
        assert!(LMP_THRESHOLDS[2] < LMP_THRESHOLDS[3]);
    }
}
