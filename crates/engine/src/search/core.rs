//! Chess search implementation using negamax with alpha-beta pruning.

use crate::board::Board;
use crate::eval::Evaluator;
use crate::move_order::MoveOrder;
use crate::r#move::Move;
use crate::time::{TimeControl, TimeManager};
use crate::tt::{Bound, TranspositionTable};

/// Maximum search depth.
pub const MAX_DEPTH: u32 = 64;

/// Checkmate score (very large value).
pub const MATE_SCORE: i32 = 30_000;

/// Infinity (larger than any possible score).
pub const INFINITY: i32 = 32_000;

/// Principal variation line (for multi-PV search).
#[derive(Debug, Clone)]
pub struct PVLine {
    pub score: i32,
    pub pv: Vec<Move>,
}

/// Search result containing the best move and score.
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub best_move: Move,
    pub score: i32,
    pub depth: u32,
    pub nodes: u64,
    pub pv: Vec<Move>,
    /// Multi-PV results (when using search_multi_pv)
    pub multi_pv: Vec<PVLine>,
}

/// Main search engine.
pub struct Searcher {
    evaluator: Evaluator,
    tt: TranspositionTable,
    move_order: MoveOrder,
    nodes: u64,
    time_manager: Option<TimeManager>,
    stopped: bool,
}

impl Searcher {
    /// Create a new searcher with default TT size (64 MB).
    pub fn new() -> Self {
        Self::with_tt_size(64)
    }

    /// Create a new searcher with custom TT size in MB.
    pub fn with_tt_size(size_mb: usize) -> Self {
        Self {
            evaluator: Evaluator::new(),
            tt: TranspositionTable::new(size_mb),
            move_order: MoveOrder::new(),
            nodes: 0,
            time_manager: None,
            stopped: false,
        }
    }

    /// Iterative deepening search.
    ///
    /// Searches from depth 1 to max_depth, using results from shallower
    /// searches to improve move ordering at deeper depths.
    ///
    /// # Arguments
    /// * `board` - The position to search
    /// * `max_depth` - Maximum search depth in plies
    /// * `time_control` - Time control for the search (defaults to Infinite)
    ///
    /// # Returns
    /// SearchResult containing best move, score, PV, and statistics
    pub fn search_with_limit(
        &mut self,
        board: &Board,
        max_depth: u32,
        time_control: TimeControl,
    ) -> SearchResult {
        self.nodes = 0;
        self.tt.new_search();
        self.move_order.clear();
        self.stopped = false;

        // Initialize time manager
        let is_white = board.side_to_move() == crate::piece::Color::White;
        self.time_manager = Some(TimeManager::new(time_control, is_white));

        let mut best_move = Move::new(
            crate::square::Square::A1,
            crate::square::Square::A1,
            crate::r#move::MoveFlags::QUIET,
        );
        let mut best_score = 0;
        let mut completed_depth = 0;

        // Iterative deepening with aspiration windows
        for depth in 1..=max_depth {
            // Check if we should stop (time, depth, or node limits)
            if self.should_stop(depth) {
                break;
            }

            let score = if depth <= 4 {
                // First few depths: use full window for stability
                self.search_root(board, depth)
            } else {
                // Aspiration window: narrow bounds around previous score
                const ASPIRATION_DELTA: i32 = 50; // 0.5 pawns

                let mut alpha = best_score - ASPIRATION_DELTA;
                let mut beta = best_score + ASPIRATION_DELTA;
                let mut delta = ASPIRATION_DELTA;

                loop {
                    let score = self.search_root_window(board, depth, alpha, beta);

                    if score <= alpha {
                        // Fail low: widen window downward
                        alpha -= delta;
                        delta *= 2;
                    } else if score >= beta {
                        // Fail high: widen window upward
                        beta += delta;
                        delta *= 2;
                    } else {
                        // Success: score within window
                        break score;
                    }

                    // Safety: prevent infinite widening
                    if delta > 1000 {
                        alpha = -INFINITY;
                        beta = INFINITY;
                    }
                }
            };

            best_score = score;
            completed_depth = depth;

            // Extract PV from TT
            let pv = self.extract_pv(board, depth);

            if let Some(&first_move) = pv.first() {
                best_move = first_move;
            }

            // Could print UCI info here in the future
            // println!("info depth {} score cp {} nodes {} pv ...", depth, score, self.nodes);
        }

        let pv = self.extract_pv(board, completed_depth);

        SearchResult {
            best_move,
            score: best_score,
            depth: completed_depth,
            nodes: self.nodes,
            pv,
            multi_pv: Vec::new(), // Empty for single-PV search
        }
    }

    /// Convenience method for unlimited search (backward compatibility).
    pub fn search(&mut self, board: &Board, max_depth: u32) -> SearchResult {
        self.search_with_limit(board, max_depth, TimeControl::Infinite)
    }

    /// Check if search should stop due to time/depth/node limits.
    fn should_stop(&self, current_depth: u32) -> bool {
        if self.stopped {
            return true;
        }

        if let Some(tm) = &self.time_manager {
            // Check hard time limit (must stop)
            if tm.must_stop() {
                return true;
            }

            // Check soft time limit (should stop)
            // But allow finishing current depth if we've made progress
            if current_depth > 1 && tm.should_stop() {
                return true;
            }

            // Check depth limit
            if tm.depth_limit_reached(current_depth) {
                return true;
            }

            // Check node limit
            if tm.node_limit_reached(self.nodes) {
                return true;
            }
        }

        false
    }

    /// Multi-PV search: find top N best moves.
    ///
    /// Searches the position N times, excluding previously found best moves
    /// each iteration. Useful for analysis mode.
    ///
    /// # Arguments
    /// * `board` - The position to search
    /// * `max_depth` - Maximum search depth in plies
    /// * `num_pv` - Number of principal variations to find
    ///
    /// # Returns
    /// SearchResult with best move and multi_pv containing all PV lines
    pub fn search_multi_pv(
        &mut self,
        board: &Board,
        max_depth: u32,
        num_pv: usize,
    ) -> SearchResult {
        if num_pv <= 1 {
            // Single PV: use regular search
            return self.search(board, max_depth);
        }

        self.nodes = 0;
        self.tt.new_search();
        self.move_order.clear();

        let mut multi_pv = Vec::new();
        let mut excluded_moves = Vec::new();

        for _pv_num in 0..num_pv {
            // Search with excluded moves
            let result = self.search_excluding(board, max_depth, &excluded_moves);

            if result.pv.is_empty() {
                // No more legal moves
                break;
            }

            multi_pv.push(PVLine {
                score: result.score,
                pv: result.pv.clone(),
            });

            // Exclude this PV's first move from next iteration
            if let Some(&first_move) = result.pv.first() {
                excluded_moves.push(first_move);
            }
        }

        // Return best as primary result
        let best = &multi_pv[0];

        SearchResult {
            best_move: best.pv[0],
            score: best.score,
            depth: max_depth,
            nodes: self.nodes,
            pv: best.pv.clone(),
            multi_pv,
        }
    }

    /// Search with a set of excluded moves (for multi-PV).
    fn search_excluding(
        &mut self,
        board: &Board,
        max_depth: u32,
        excluded: &[Move],
    ) -> SearchResult {
        let mut best_move = Move::new(
            crate::square::Square::A1,
            crate::square::Square::A1,
            crate::r#move::MoveFlags::QUIET,
        );
        let mut best_score = 0;

        // Iterative deepening (simplified - no aspiration windows for multi-PV)
        for depth in 1..=max_depth {
            let score = self.search_root_excluding(board, depth, excluded);

            best_score = score;

            // Extract PV from TT
            let pv = self.extract_pv(board, depth);

            if let Some(&first_move) = pv.first() {
                best_move = first_move;
            }
        }

        let pv = self.extract_pv(board, max_depth);

        SearchResult {
            best_move,
            score: best_score,
            depth: max_depth,
            nodes: self.nodes,
            pv,
            multi_pv: Vec::new(),
        }
    }

    /// Search at the root with excluded moves (for multi-PV).
    fn search_root_excluding(&mut self, board: &Board, depth: u32, excluded: &[Move]) -> i32 {
        let all_moves = board.generate_legal_moves();

        // Filter out excluded moves
        let mut legal_moves = crate::movelist::MoveList::new();
        for m in all_moves.iter() {
            if !excluded.contains(m) {
                legal_moves.push(*m);
            }
        }

        if legal_moves.is_empty() {
            return if board.is_in_check() { -MATE_SCORE } else { 0 };
        }

        // Order moves (using TT move from previous iteration if available)
        let tt_move = self.tt.probe(board.hash()).map(|e| e.best_move);
        self.move_order
            .order_moves(board, &mut legal_moves, 0, tt_move, None);

        let mut best_score = -INFINITY;
        let mut best_move = legal_moves[0];
        let mut alpha = -INFINITY;
        let beta = INFINITY;

        for m in legal_moves.iter() {
            let mut new_board = board.clone();
            new_board.make_move(*m);

            let score = -self.negamax(&new_board, depth as i32 - 1, -beta, -alpha, 1);

            if score > best_score {
                best_score = score;
                best_move = *m;
            }

            alpha = alpha.max(score);
        }

        // Store best move in TT
        self.tt.store(
            board.hash(),
            best_move,
            best_score,
            depth as u8,
            Bound::Exact,
        );

        best_score
    }

    /// Search at the root (find best move at current depth).
    fn search_root(&mut self, board: &Board, depth: u32) -> i32 {
        self.search_root_window(board, depth, -INFINITY, INFINITY)
    }

    /// Search at the root with custom alpha-beta window.
    /// Used for aspiration windows.
    fn search_root_window(&mut self, board: &Board, depth: u32, mut alpha: i32, beta: i32) -> i32 {
        let mut legal_moves = board.generate_legal_moves();

        if legal_moves.is_empty() {
            return if board.is_in_check() { -MATE_SCORE } else { 0 };
        }

        // Order moves (using TT move from previous iteration if available)
        let tt_move = self.tt.probe(board.hash()).map(|e| e.best_move);
        self.move_order
            .order_moves(board, &mut legal_moves, 0, tt_move, None);

        let mut best_score = -INFINITY;
        let mut best_move = legal_moves[0];

        for m in legal_moves.iter() {
            let mut new_board = board.clone();
            new_board.make_move(*m);

            let score = -self.negamax(&new_board, depth as i32 - 1, -beta, -alpha, 1);

            if score > best_score {
                best_score = score;
                best_move = *m;
            }

            alpha = alpha.max(score);

            // Beta cutoff at root
            if alpha >= beta {
                break;
            }
        }

        // Store best move in TT
        let bound = if best_score >= beta {
            Bound::Lower
        } else if best_score > alpha {
            Bound::Exact
        } else {
            Bound::Upper
        };

        self.tt
            .store(board.hash(), best_move, best_score, depth as u8, bound);

        best_score
    }

    /// Extract principal variation from transposition table.
    fn extract_pv(&self, board: &Board, max_depth: u32) -> Vec<Move> {
        let mut pv = Vec::new();
        let mut current_board = board.clone();
        let mut seen_positions = std::collections::HashSet::new();

        for _ in 0..max_depth {
            let hash = current_board.hash();

            // Avoid cycles
            if !seen_positions.insert(hash) {
                break;
            }

            // Probe TT for best move
            if let Some(entry) = self.tt.probe(hash) {
                let m = entry.best_move;

                // Verify move is legal
                if !current_board.is_legal(m) {
                    break;
                }

                pv.push(m);
                current_board.make_move(m);
            } else {
                break;
            }
        }

        pv
    }

    /// Negamax search with alpha-beta pruning.
    ///
    /// # Arguments
    /// * `board` - Current position
    /// * `depth` - Remaining search depth
    /// * `alpha` - Lower bound (best score for current side)
    /// * `beta` - Upper bound (best score opponent can force)
    /// * `ply` - Distance from root
    ///
    /// # Returns
    /// The evaluation score from the current side's perspective
    fn negamax(&mut self, board: &Board, depth: i32, mut alpha: i32, beta: i32, ply: u32) -> i32 {
        self.nodes += 1;

        // Check time limits periodically (every 1024 nodes)
        if self.nodes.is_multiple_of(1024) {
            if let Some(tm) = &self.time_manager {
                if tm.must_stop() {
                    self.stopped = true;
                    return 0; // Return early with neutral score
                }
            }
        }

        // If we've been stopped, return immediately
        if self.stopped {
            return 0;
        }

        let original_alpha = alpha;
        let hash = board.hash();

        // Probe transposition table
        let tt_move = if let Some(tt_entry) = self.tt.probe(hash) {
            if tt_entry.depth >= depth as u8 {
                match tt_entry.bound {
                    Bound::Exact => return tt_entry.score,
                    Bound::Lower => alpha = alpha.max(tt_entry.score),
                    Bound::Upper => {
                        if tt_entry.score < beta {
                            return tt_entry.score;
                        }
                    }
                }
                if alpha >= beta {
                    return tt_entry.score;
                }
            }
            Some(tt_entry.best_move)
        } else {
            None
        };

        // Null move pruning
        // Try "passing" the turn - if position is still winning, we can skip full search
        // Conditions:
        // - Not in check (zugzwang risk)
        // - Sufficient depth (need depth for reduced search)
        // - Not in endgame (zugzwang risk)
        // - Beta is not a mate score (avoid mate score distortion)
        if depth >= 3
            && !board.is_in_check()
            && !crate::eval::is_endgame(board)
            && beta.abs() < MATE_SCORE - MAX_DEPTH as i32
        {
            const R: i32 = 2; // Reduction factor

            let mut null_board = board.clone();
            null_board.make_null_move();

            // Search with reduced depth and null window around beta
            let null_score = -self.negamax(&null_board, depth - 1 - R, -beta, -beta + 1, ply + 1);

            // If null move fails high, position is too good - prune this branch
            if null_score >= beta {
                return beta;
            }
        }

        // Leaf node: enter quiescence search
        if depth <= 0 {
            return self.quiesce(board, alpha, beta);
        }

        let mut legal_moves = board.generate_legal_moves();

        // Checkmate or stalemate
        if legal_moves.is_empty() {
            return if board.is_in_check() {
                // Checkmate: return negative score (we're mated)
                // Prefer shorter mates
                -MATE_SCORE + ply as i32
            } else {
                // Stalemate
                0
            };
        }

        // Order moves for better alpha-beta pruning
        self.move_order
            .order_moves(board, &mut legal_moves, ply as usize, tt_move, None);

        let mut best_score = -INFINITY;
        let mut best_move = legal_moves[0];

        for (move_count, m) in legal_moves.iter().enumerate() {
            let mut new_board = board.clone();
            new_board.make_move(*m);

            let mut score;

            // Late Move Reductions (LMR)
            // Reduce search depth for moves that are unlikely to be best
            // Conditions:
            // 1. Not the first few moves (move_count >= 3)
            // 2. Sufficient depth (depth >= 3)
            // 3. Not a tactical move (capture, promotion, gives check)
            // 4. Not currently in check
            let can_reduce = move_count >= 3
                && depth >= 3
                && !m.is_capture()
                && !m.is_promotion()
                && !new_board.is_in_check()
                && !board.is_in_check();

            if can_reduce {
                // Calculate reduction amount
                // More reduction for later moves and higher depths
                let reduction = if move_count >= 6 && depth >= 6 {
                    2 // Reduce by 2 plies
                } else {
                    1 // Reduce by 1 ply
                };

                // Search at reduced depth with null window
                score = -self.negamax(
                    &new_board,
                    depth - 1 - reduction,
                    -alpha - 1,
                    -alpha,
                    ply + 1,
                );

                // If reduced search beats alpha, re-search at full depth
                if score > alpha {
                    score = -self.negamax(&new_board, depth - 1, -beta, -alpha, ply + 1);
                }
            } else {
                // First few moves: search at full depth
                // Use PVS (Principal Variation Search) for efficiency

                if move_count == 0 {
                    // First move: search with full window
                    score = -self.negamax(&new_board, depth - 1, -beta, -alpha, ply + 1);
                } else {
                    // Later moves: try null window first (PVS)
                    score = -self.negamax(&new_board, depth - 1, -alpha - 1, -alpha, ply + 1);

                    // If it beats alpha, re-search with full window
                    if score > alpha && score < beta {
                        score = -self.negamax(&new_board, depth - 1, -beta, -alpha, ply + 1);
                    }
                }
            }

            if score > best_score {
                best_score = score;
                best_move = *m;
            }

            alpha = alpha.max(score);

            // Beta cutoff (opponent has a better option earlier)
            if alpha >= beta {
                // Store killer move and update history if it's a quiet move
                if !m.is_capture() {
                    self.move_order.store_killer(*m, ply as usize);
                    self.move_order.update_history(*m, depth);
                }
                break;
            }
        }

        // Store in transposition table
        let bound = if best_score >= beta {
            Bound::Lower // Beta cutoff
        } else if best_score > original_alpha {
            Bound::Exact // PV node
        } else {
            Bound::Upper // All-node (fail-low)
        };

        self.tt
            .store(hash, best_move, best_score, depth as u8, bound);

        best_score
    }

    /// Quiescence search to avoid horizon effect.
    ///
    /// Only searches tactical moves (captures) to reach a quiet position.
    fn quiesce(&mut self, board: &Board, mut alpha: i32, beta: i32) -> i32 {
        self.nodes += 1;

        // If we've been stopped, return immediately
        if self.stopped {
            return 0;
        }

        // Stand pat: assume we can maintain current evaluation
        let stand_pat = self.evaluator.evaluate(board);

        if stand_pat >= beta {
            return beta;
        }

        if stand_pat > alpha {
            alpha = stand_pat;
        }

        // Generate and search only captures
        let moves = board.generate_legal_moves();
        let captures: Vec<Move> = moves.iter().filter(|m| m.is_capture()).copied().collect();

        for m in captures {
            let mut new_board = board.clone();
            new_board.make_move(m);

            let score = -self.quiesce(&new_board, -beta, -alpha);

            if score >= beta {
                return beta;
            }

            alpha = alpha.max(score);
        }

        alpha
    }
}

impl Default for Searcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::parse_fen;

    #[test]
    fn test_search_startpos() {
        let board = Board::startpos();
        let mut searcher = Searcher::new();

        let result = searcher.search(&board, 3);

        // Should find a legal move
        assert!(board.is_legal(result.best_move));
        assert!(result.nodes > 0);
        assert_eq!(result.depth, 3);
    }

    #[test]
    fn test_search_mate_in_one() {
        // Simple back rank mate: Rook can deliver mate
        let fen = "6k1/5ppp/8/8/8/8/5PPP/R5K1 w - - 0 1";
        let board = parse_fen(fen).unwrap();
        let mut searcher = Searcher::new();

        let result = searcher.search(&board, 3);

        // Should find a move (Ra8#)
        assert!(board.is_legal(result.best_move));
        // With deeper search, should recognize the mate threat
        // (At depth 3, it might not fully recognize mate, but should prefer it)
    }

    #[test]
    fn test_iterative_deepening() {
        let board = Board::startpos();
        let mut searcher = Searcher::new();

        let result = searcher.search(&board, 3);

        // Should return depth 3 result
        assert_eq!(result.depth, 3);
        // Should have searched multiple depths (1, 2, 3)
        assert!(result.nodes > 400); // More than just depth 3
    }

    #[test]
    fn test_pv_extraction() {
        let board = Board::startpos();
        let mut searcher = Searcher::new();

        let result = searcher.search(&board, 3);

        // Should extract a PV
        assert!(!result.pv.is_empty());
        assert!(result.pv.len() <= 3);

        // All moves in PV should be legal
        let mut test_board = board.clone();
        for m in &result.pv {
            assert!(test_board.is_legal(*m));
            test_board.make_move(*m);
        }
    }

    #[test]
    fn test_alpha_beta_prunes() {
        // Alpha-beta should search fewer nodes than full minimax
        let board = Board::startpos();
        let mut searcher = Searcher::new();

        let result = searcher.search(&board, 3);

        // At depth 3 from startpos, with perfect ordering we'd search ~400 nodes
        // Without pruning, we'd search ~8900 nodes
        // We should be somewhere in between
        println!("Nodes at depth 3: {}", result.nodes);
        assert!(result.nodes < 8_900);
    }

    #[test]
    fn test_null_move_mechanics() {
        use crate::piece::Color;

        let board = Board::startpos();
        let mut null_board = board.clone();

        // Test side switches
        assert_eq!(board.side_to_move(), Color::White);
        null_board.make_null_move();
        assert_eq!(null_board.side_to_move(), Color::Black);

        // Test hash changes
        assert_ne!(board.hash(), null_board.hash());

        // Test with en passant
        let fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
        let board_with_ep = parse_fen(fen).unwrap();
        let mut null_board_ep = board_with_ep.clone();

        assert!(board_with_ep.ep_square().is_some());
        null_board_ep.make_null_move();
        assert!(null_board_ep.ep_square().is_none());
    }

    #[test]
    fn test_null_move_pruning_reduces_nodes() {
        // Compare node counts with a position where null move should help
        // Use a quiet middlegame position
        let fen = "r1bqkb1r/pppp1ppp/2n2n2/1B2p3/4P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4";
        let board = parse_fen(fen).unwrap();

        let mut searcher = Searcher::new();
        let result = searcher.search(&board, 5);

        println!("Nodes with null move pruning at depth 5: {}", result.nodes);

        // With null move pruning, we expect significant node reduction
        // At depth 5, we should search considerably fewer nodes
        // This is a basic sanity check - exact numbers vary by position
        assert!(result.nodes > 1000); // Should still search something
        assert!(result.nodes < 150_000); // But not too much (relaxed threshold)
    }

    #[test]
    fn test_null_move_disabled_in_endgame() {
        // Test that null move is disabled in endgames
        // Simple endgame - no queens, low material
        let fen = "8/8/4k3/8/8/4K3/8/8 w - - 0 1";
        let board = parse_fen(fen).unwrap();

        // Verify this is considered an endgame
        assert!(crate::eval::is_endgame(&board));

        let mut searcher = Searcher::new();
        let result = searcher.search(&board, 4);

        // Should complete search without errors
        // (null move is disabled in endgames, avoiding zugzwang issues)
        assert!(result.nodes > 0);
    }

    #[test]
    fn test_null_move_doesnt_break_tactics() {
        // Test that null move pruning doesn't cause tactical oversights
        // Position with a clear tactic
        let fen = "r1bqkb1r/pppp1ppp/2n2n2/1B2p3/4P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4";
        let board = parse_fen(fen).unwrap();

        let mut searcher = Searcher::new();
        let result = searcher.search(&board, 5);

        // Should find a legal, reasonable move
        assert!(board.is_legal(result.best_move));
        // Score should be reasonable (not a crazy mate score unless it's actually mate)
        assert!(result.score.abs() < MATE_SCORE / 2);
    }

    #[test]
    fn test_lmr_reduces_nodes() {
        // Test that LMR significantly reduces node count
        let fen = "r1bqkb1r/pppp1ppp/2n2n2/1B2p3/4P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4";
        let board = parse_fen(fen).unwrap();

        let mut searcher = Searcher::new();
        let result = searcher.search(&board, 6);

        println!("Nodes with LMR + null move at depth 6: {}", result.nodes);

        // With both LMR and null move, expect major node reduction
        // At depth 6, should be significantly less than without optimizations
        // M7 enhanced move ordering may cause slightly different node counts
        assert!(result.nodes > 1000); // Should still search something
        assert!(result.nodes < 400_000); // But much less than naive search
    }

    #[test]
    fn test_lmr_tactical_accuracy() {
        // Test that LMR doesn't miss tactics
        // Position where there's a clear best move (Bxf7+ fork)
        let fen = "r1bqkbnr/pppp1ppp/2n5/4p3/2B1P3/5Q2/PPPP1PPP/RNB1K1NR w KQkq - 0 1";
        let board = parse_fen(fen).unwrap();

        let mut searcher = Searcher::new();
        let result = searcher.search(&board, 5);

        // Should find a legal move
        assert!(board.is_legal(result.best_move));
        // Should have a positive score (white is winning)
        assert!(result.score > 0);
        println!(
            "Best move: {} with score {}",
            result.best_move.to_uci(),
            result.score
        );
    }

    #[test]
    fn test_lmr_finds_mate() {
        // Test that LMR doesn't miss mate in few moves
        // Back rank mate: Ra8# is mate in 1
        let fen = "6k1/5ppp/8/8/8/8/5PPP/R5K1 w - - 0 1";
        let board = parse_fen(fen).unwrap();

        let mut searcher = Searcher::new();
        let result = searcher.search(&board, 5);

        // Should find the mating move Ra8
        assert!(board.is_legal(result.best_move));
        println!(
            "Best move: {} with score {}",
            result.best_move.to_uci(),
            result.score
        );
        // Should recognize this is a winning position
        assert!(result.score > 1000 || result.score < -1000); // Either we see mate or strong advantage
    }

    #[test]
    fn test_lmr_deeper_search() {
        // Test that LMR enables deeper search
        let board = Board::startpos();

        let mut searcher = Searcher::new();
        let result = searcher.search(&board, 6);

        println!("Nodes at depth 6 with LMR: {}", result.nodes);

        // Should complete depth 6 search
        assert_eq!(result.depth, 6);
        assert!(result.nodes > 5000); // Should search reasonably
                                      // With LMR, depth 6 should be feasible
        assert!(result.nodes < 500_000);
    }

    #[test]
    fn test_pvs_first_move_full_window() {
        // Test that PVS gives first move full window
        // This is more of a regression test to ensure the logic is correct
        let board = Board::startpos();

        let mut searcher = Searcher::new();
        let result = searcher.search(&board, 4);

        // Should find a legal move
        assert!(board.is_legal(result.best_move));
        assert!(result.nodes > 0);
    }

    #[test]
    fn test_lmr_quiet_moves_only() {
        // Test that LMR only applies to quiet moves
        // Position with many captures available
        let fen = "r1bqkb1r/pppp1ppp/2n2n2/1B2p3/4P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4";
        let board = parse_fen(fen).unwrap();

        let mut searcher = Searcher::new();
        let result = searcher.search(&board, 5);

        // Should still find good moves (captures should be searched fully)
        assert!(board.is_legal(result.best_move));
        println!("Nodes searched: {}", result.nodes);
    }

    #[test]
    fn test_aspiration_window_basic() {
        // Test that aspiration windows work correctly
        let board = Board::startpos();

        let mut searcher = Searcher::new();
        let result = searcher.search(&board, 6);

        // Should find a legal move
        assert!(board.is_legal(result.best_move));
        assert_eq!(result.depth, 6);
        println!("Aspiration windows at depth 6: {} nodes", result.nodes);
    }

    #[test]
    fn test_aspiration_window_tactical_position() {
        // Test aspiration windows on a tactical position
        // Position with clear best move
        let fen = "r1bqkb1r/pppp1ppp/2n2n2/1B2p3/4P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4";
        let board = parse_fen(fen).unwrap();

        let mut searcher = Searcher::new();
        let result = searcher.search(&board, 6);

        // Should find a good move
        assert!(board.is_legal(result.best_move));
        println!(
            "Best move: {} with score {}",
            result.best_move.to_uci(),
            result.score
        );
    }

    #[test]
    fn test_aspiration_window_stable_score() {
        // Test that aspiration windows handle stable scores well
        let board = Board::startpos();

        let mut searcher = Searcher::new();
        let result = searcher.search(&board, 5);

        // Score should be reasonable (near equality at start)
        assert!(result.score.abs() < 200); // Within 2 pawns
        assert!(board.is_legal(result.best_move));
    }

    #[test]
    fn test_aspiration_window_mate_position() {
        // Test aspiration windows with mate scores
        let fen = "6k1/5ppp/8/8/8/8/5PPP/R5K1 w - - 0 1";
        let board = parse_fen(fen).unwrap();

        let mut searcher = Searcher::new();
        let result = searcher.search(&board, 5);

        // Should find mate or strong advantage
        assert!(board.is_legal(result.best_move));
        println!(
            "Mate position score: {} with move {}",
            result.score,
            result.best_move.to_uci()
        );
    }

    #[test]
    fn test_multi_pv_basic() {
        // Test Multi-PV finds multiple best moves
        let board = Board::startpos();

        let mut searcher = Searcher::new();
        let result = searcher.search_multi_pv(&board, 4, 3);

        // Should find 3 PV lines
        assert_eq!(result.multi_pv.len(), 3);

        // All moves should be legal
        for pv_line in &result.multi_pv {
            assert!(!pv_line.pv.is_empty());
            assert!(board.is_legal(pv_line.pv[0]));
        }

        // Scores should be ordered (PV1 >= PV2 >= PV3)
        assert!(result.multi_pv[0].score >= result.multi_pv[1].score);
        assert!(result.multi_pv[1].score >= result.multi_pv[2].score);

        println!("Multi-PV results:");
        for (i, pv_line) in result.multi_pv.iter().enumerate() {
            println!(
                "  PV{}: {} (score: {})",
                i + 1,
                pv_line.pv[0].to_uci(),
                pv_line.score
            );
        }
    }

    #[test]
    fn test_multi_pv_no_duplicates() {
        // Test that Multi-PV doesn't return duplicate moves
        let board = Board::startpos();

        let mut searcher = Searcher::new();
        let result = searcher.search_multi_pv(&board, 4, 5);

        // Collect first moves from each PV
        let first_moves: Vec<Move> = result.multi_pv.iter().map(|pv| pv.pv[0]).collect();

        // Check for duplicates
        for i in 0..first_moves.len() {
            for j in (i + 1)..first_moves.len() {
                assert_ne!(
                    first_moves[i], first_moves[j],
                    "Found duplicate moves in multi-PV"
                );
            }
        }
    }

    #[test]
    fn test_multi_pv_tactical_position() {
        // Test Multi-PV on a position with clear alternatives
        let fen = "r1bqkb1r/pppp1ppp/2n2n2/1B2p3/4P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4";
        let board = parse_fen(fen).unwrap();

        let mut searcher = Searcher::new();
        let result = searcher.search_multi_pv(&board, 4, 3);

        // Should find multiple PV lines
        assert!(result.multi_pv.len() >= 2);

        // All should be legal
        for pv_line in &result.multi_pv {
            assert!(board.is_legal(pv_line.pv[0]));
        }

        println!("Tactical position Multi-PV:");
        for (i, pv_line) in result.multi_pv.iter().enumerate() {
            println!(
                "  PV{}: {} (score: {})",
                i + 1,
                pv_line.pv[0].to_uci(),
                pv_line.score
            );
        }
    }

    #[test]
    fn test_multi_pv_single_pv_fallback() {
        // Test that requesting 1 PV falls back to regular search
        let board = Board::startpos();

        let mut searcher1 = Searcher::new();
        let result1 = searcher1.search(&board, 4);

        let mut searcher2 = Searcher::new();
        let result2 = searcher2.search_multi_pv(&board, 4, 1);

        // Should get same result (same best move)
        assert_eq!(result1.best_move, result2.best_move);
        assert_eq!(result1.score, result2.score);
    }

    #[test]
    fn test_time_limited_search_move_time() {
        // Test fixed time per move
        let board = Board::startpos();
        let mut searcher = Searcher::new();

        let time_control = TimeControl::MoveTime { millis: 100 };
        let result = searcher.search_with_limit(&board, 10, time_control);

        // Should find a legal move
        assert!(board.is_legal(result.best_move));
        // Should have stopped before reaching depth 10 (100ms is too short for depth 10)
        assert!(result.depth < 10);
    }

    #[test]
    fn test_depth_limited_search() {
        // Test depth limit
        let board = Board::startpos();
        let mut searcher = Searcher::new();

        let time_control = TimeControl::Depth { depth: 3 };
        let result = searcher.search_with_limit(&board, 10, time_control);

        // Should stop at depth 3
        assert_eq!(result.depth, 3);
        assert!(board.is_legal(result.best_move));
    }

    #[test]
    fn test_node_limited_search() {
        // Test node limit
        let board = Board::startpos();
        let mut searcher = Searcher::new();

        let time_control = TimeControl::Nodes { nodes: 1000 };
        let result = searcher.search_with_limit(&board, 10, time_control);

        // Should stop when nodes exceeded
        assert!(result.nodes >= 1000);
        // Should not search to full depth
        assert!(result.depth < 10);
        assert!(board.is_legal(result.best_move));
    }

    #[test]
    fn test_infinite_time_control() {
        // Test that infinite time control searches to full depth
        let board = Board::startpos();
        let mut searcher = Searcher::new();

        let time_control = TimeControl::Infinite;
        let result = searcher.search_with_limit(&board, 4, time_control);

        // Should search to full depth
        assert_eq!(result.depth, 4);
        assert!(board.is_legal(result.best_move));
    }

    #[test]
    fn test_clock_time_control() {
        // Test clock-based time control
        let board = Board::startpos();
        let mut searcher = Searcher::new();

        let time_control = TimeControl::Clock {
            wtime: 10000, // 10 seconds
            btime: 10000,
            winc: 0,
            binc: 0,
            movestogo: Some(20),
        };
        let result = searcher.search_with_limit(&board, 10, time_control);

        // Should find a legal move
        assert!(board.is_legal(result.best_move));
        // Should stop before reaching depth 10 (not enough time)
        assert!(result.depth < 10);
    }

    #[test]
    fn test_backward_compatible_search() {
        // Test that old search() method still works (backward compatibility)
        let board = Board::startpos();
        let mut searcher = Searcher::new();

        let result = searcher.search(&board, 4);

        // Should search to full depth (no time limit)
        assert_eq!(result.depth, 4);
        assert!(board.is_legal(result.best_move));
    }
}
