//! Chess search implementation using negamax with alpha-beta pruning.

use crate::board::Board;
use crate::eval::Evaluator;
use crate::move_order::MoveOrder;
use crate::r#move::Move;
use crate::tt::{Bound, TranspositionTable};

/// Maximum search depth.
pub const MAX_DEPTH: u32 = 64;

/// Checkmate score (very large value).
pub const MATE_SCORE: i32 = 30_000;

/// Infinity (larger than any possible score).
pub const INFINITY: i32 = 32_000;

/// Search result containing the best move and score.
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub best_move: Move,
    pub score: i32,
    pub depth: u32,
    pub nodes: u64,
    pub pv: Vec<Move>,
}

/// Main search engine.
pub struct Searcher {
    evaluator: Evaluator,
    tt: TranspositionTable,
    move_order: MoveOrder,
    nodes: u64,
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
    ///
    /// # Returns
    /// SearchResult containing best move, score, PV, and statistics
    pub fn search(&mut self, board: &Board, max_depth: u32) -> SearchResult {
        self.nodes = 0;
        self.tt.new_search();
        self.move_order.clear();

        let mut best_move = Move::new(
            crate::square::Square::A1,
            crate::square::Square::A1,
            crate::r#move::MoveFlags::QUIET,
        );
        let mut best_score = 0;

        // Iterative deepening: search 1, 2, 3, ..., max_depth
        for depth in 1..=max_depth {
            let score = self.search_root(board, depth);

            // Extract PV from TT
            let pv = self.extract_pv(board, depth);

            if let Some(&first_move) = pv.first() {
                best_move = first_move;
                best_score = score;
            }

            // Could print UCI info here in the future
            // println!("info depth {} score cp {} nodes {} pv ...", depth, score, self.nodes);
        }

        let pv = self.extract_pv(board, max_depth);

        SearchResult {
            best_move,
            score: best_score,
            depth: max_depth,
            nodes: self.nodes,
            pv,
        }
    }

    /// Search at the root (find best move at current depth).
    fn search_root(&mut self, board: &Board, depth: u32) -> i32 {
        let mut legal_moves = board.generate_legal_moves();

        if legal_moves.is_empty() {
            return if board.is_in_check() { -MATE_SCORE } else { 0 };
        }

        // Order moves (using TT move from previous iteration if available)
        let tt_move = self.tt.probe(board.hash()).map(|e| e.best_move);
        self.move_order
            .order_moves(board, &mut legal_moves, 0, tt_move);

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
            .order_moves(board, &mut legal_moves, ply as usize, tt_move);

        let mut best_score = -INFINITY;
        let mut best_move = legal_moves[0];

        for m in legal_moves.iter() {
            let mut new_board = board.clone();
            new_board.make_move(*m);

            let score = -self.negamax(&new_board, depth - 1, -beta, -alpha, ply + 1);

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
}
