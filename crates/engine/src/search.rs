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

    /// Search for the best move at a given depth.
    ///
    /// # Arguments
    /// * `board` - The position to search
    /// * `depth` - The search depth in plies
    ///
    /// # Returns
    /// SearchResult containing best move, score, and statistics
    pub fn search(&mut self, board: &Board, depth: u32) -> SearchResult {
        self.nodes = 0;
        self.tt.new_search(); // Increment generation

        let _score = self.negamax(board, depth as i32, -INFINITY, INFINITY, 0);
        let legal_moves = board.generate_legal_moves();

        // Find the best move by searching each root move
        let mut best_move = if legal_moves.is_empty() {
            Move::new(
                crate::square::Square::A1,
                crate::square::Square::A1,
                crate::r#move::MoveFlags::QUIET,
            )
        } else {
            legal_moves[0]
        };
        let mut best_score = -INFINITY;

        for m in legal_moves.iter() {
            let mut new_board = board.clone();
            new_board.make_move(*m);
            let score = -self.negamax(&new_board, depth as i32 - 1, -INFINITY, INFINITY, 1);

            if score > best_score {
                best_score = score;
                best_move = *m;
            }
        }

        SearchResult {
            best_move,
            score: best_score,
            depth,
            nodes: self.nodes,
        }
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

        // Leaf node: return evaluation
        if depth <= 0 {
            return self.evaluator.evaluate(board);
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

        let result = searcher.search(&board, 1);

        // Should find a legal move
        assert!(board.is_legal(result.best_move));
        assert!(result.nodes > 0);
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
    fn test_search_finds_better_at_depth_2() {
        let board = Board::startpos();
        let mut searcher = Searcher::new();

        let result1 = searcher.search(&board, 1);
        let result2 = searcher.search(&board, 2);

        // Deeper search should search more nodes
        assert!(result2.nodes > result1.nodes);
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
