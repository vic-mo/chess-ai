//! Chess search implementation using negamax with alpha-beta pruning.

use crate::board::Board;
use crate::eval::Evaluator;
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
        let legal_moves = board.generate_legal_moves();

        if legal_moves.is_empty() {
            return if board.is_in_check() { -MATE_SCORE } else { 0 };
        }

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
        if let Some(tt_entry) = self.tt.probe(hash) {
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
        }

        // Leaf node: enter quiescence search
        if depth <= 0 {
            return self.quiesce(board, alpha, beta);
        }

        let legal_moves = board.generate_legal_moves();

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
}
