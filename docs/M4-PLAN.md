# M4: Advanced Search Techniques - Implementation Plan

**Milestone:** M4 - Advanced Search
**Prerequisites:** M1 ✅ M2 ✅ M3 ✅
**Duration:** 2-3 weeks (10-15 sessions)
**Goal:** Dramatically improve search efficiency and depth through advanced pruning and move ordering

---

## Executive Summary

M4 builds on the M3 foundation (negamax + alpha-beta + TT + iterative deepening + quiescence) by adding sophisticated search optimizations that will:

- **3-5× deeper search** at same time budget (depth 10-12 vs depth 6-8)
- **50-70% node reduction** through better pruning
- **Stronger tactical play** through better move ordering
- **Foundation for M5** (time management needs good search efficiency)

### Key Techniques

1. **Move Ordering** - Search better moves first for maximum pruning
2. **Null Move Pruning** - Skip moves to prove position is winning
3. **Late Move Reductions** - Search unlikely moves at reduced depth
4. **Aspiration Windows** - Narrow search bounds for better pruning
5. **Multi-PV** - Find multiple best moves for analysis

---

## Current State (M3)

### What We Have

```rust
// M3 Search Architecture
pub struct Searcher {
    evaluator: Evaluator,
    tt: TranspositionTable,
    nodes: u64,
}

// Current search features:
// ✅ Iterative deepening (1, 2, 3, ..., max_depth)
// ✅ Negamax with alpha-beta
// ✅ Transposition table probing/storing
// ✅ Quiescence search
// ✅ PV extraction from TT
// ✅ Mate detection
```

### What's Missing

```rust
// ❌ Move ordering (searches in arbitrary order)
// ❌ Null move pruning (wastes time on lost positions)
// ❌ Late move reductions (searches all moves at full depth)
// ❌ Aspiration windows (uses -INF to +INF bounds)
// ❌ Killer moves
// ❌ History heuristic
// ❌ Multi-PV search
```

### Performance Baseline

- **Depth:** 6-8 in ~1 second
- **Nodes:** ~100k-500k nodes per search
- **Branching factor:** ~5-6 effective (after alpha-beta)
- **Move ordering:** Random (no prioritization)

---

## M4 Implementation Plan

### Session Breakdown

| Session | Focus                        | Duration | Files                    |
| ------- | ---------------------------- | -------- | ------------------------ |
| 1-2     | Move ordering infrastructure | 2-3 hrs  | move_order.rs (new)      |
| 3       | MVV-LVA for captures         | 1-2 hrs  | move_order.rs            |
| 4-5     | Killer moves (2 per ply)     | 2-3 hrs  | search.rs, move_order.rs |
| 6-7     | History heuristic            | 2-3 hrs  | history.rs (new)         |
| 8-9     | Null move pruning            | 2-3 hrs  | search.rs                |
| 10-12   | Late move reductions (LMR)   | 3-4 hrs  | search.rs                |
| 13      | Aspiration windows           | 1-2 hrs  | search.rs                |
| 14      | Multi-PV search              | 2-3 hrs  | search.rs                |
| 15      | Testing & benchmarking       | 2-3 hrs  | tests/                   |
| **---** | **---**                      | **---**  | **---**                  |
| Total   | **M4 Complete**              | 20-30hrs | **3-5× deeper search!**  |

---

## Session 1-2: Move Ordering Infrastructure

### Goals

- Create `MoveOrder` struct to score and sort moves
- Integrate with existing search
- Establish ordering framework for future enhancements

### Technical Design

```rust
// crates/engine/src/move_order.rs

use crate::board::Board;
use crate::r#move::Move;

/// Move ordering scores for different move types
pub struct MoveOrder {
    /// Killer moves for each ply [ply][slot]
    killers: [[Option<Move>; 2]; MAX_PLY],
    /// History scores [from_sq][to_sq]
    history: [[i32; 64]; 64],
}

impl MoveOrder {
    pub fn new() -> Self {
        Self {
            killers: [[None; 2]; MAX_PLY],
            history: [[0; 64]; 64],
        }
    }

    /// Score a move for ordering purposes
    pub fn score_move(
        &self,
        board: &Board,
        m: Move,
        ply: usize,
        tt_move: Option<Move>,
    ) -> i32 {
        // High scores searched first

        // 1. TT move (from transposition table)
        if Some(m) == tt_move {
            return 10_000_000;
        }

        // 2. Winning captures (will add MVV-LVA in Session 3)
        if m.is_capture() {
            return 1_000_000; // Placeholder
        }

        // 3. Killer moves (will add in Sessions 4-5)
        // 4. History heuristic (will add in Sessions 6-7)

        // 5. Quiet moves (lowest priority)
        0
    }

    /// Sort moves in-place by score (highest first)
    pub fn order_moves(
        &mut self,
        board: &Board,
        moves: &mut Vec<Move>,
        ply: usize,
        tt_move: Option<Move>,
    ) {
        moves.sort_by_cached_key(|&m| {
            -self.score_move(board, m, ply, tt_move)
        });
    }

    /// Clear history and killers for new search
    pub fn clear(&mut self) {
        self.killers = [[None; 2]; MAX_PLY];
        self.history = [[0; 64]; 64];
    }
}
```

### Integration with Search

```rust
// crates/engine/src/search.rs

pub struct Searcher {
    evaluator: Evaluator,
    tt: TranspositionTable,
    move_order: MoveOrder,  // NEW
    nodes: u64,
}

impl Searcher {
    pub fn new() -> Self {
        Self {
            evaluator: Evaluator::new(),
            tt: TranspositionTable::new(64),
            move_order: MoveOrder::new(),  // NEW
            nodes: 0,
        }
    }

    pub fn search(&mut self, board: &Board, max_depth: u32) -> SearchResult {
        self.nodes = 0;
        self.tt.new_search();
        self.move_order.clear();  // NEW

        // ... rest of iterative deepening
    }

    fn negamax(&mut self, board: &Board, depth: i32, mut alpha: i32, beta: i32, ply: u32) -> i32 {
        // ... TT probe to get tt_move ...

        let mut legal_moves = board.generate_legal_moves();

        if legal_moves.is_empty() {
            // checkmate/stalemate
        }

        // NEW: Order moves before searching
        let tt_move = self.tt.probe(board.hash()).map(|e| e.best_move);
        self.move_order.order_moves(board, &mut legal_moves, ply as usize, tt_move);

        for m in legal_moves.iter() {
            // ... search move ...
        }
    }
}
```

### Success Criteria

- ✅ Move ordering compiles and runs
- ✅ TT move is searched first (when available)
- ✅ All existing tests still pass
- ✅ No performance regression

### Testing

```rust
#[test]
fn test_move_ordering_tt_move_first() {
    let board = Board::startpos();
    let mut move_order = MoveOrder::new();

    let mut moves = board.generate_legal_moves();
    let tt_move = moves[5]; // arbitrary move

    move_order.order_moves(&board, &mut moves, 0, Some(tt_move));

    // TT move should be first
    assert_eq!(moves[0], tt_move);
}
```

---

## Session 3: MVV-LVA (Most Valuable Victim - Least Valuable Attacker)

### Goals

- Prioritize capturing high-value pieces with low-value pieces
- Example: Queen captures Pawn scored higher than Pawn captures Queen

### Technical Design

```rust
// crates/engine/src/move_order.rs

impl MoveOrder {
    /// MVV-LVA score for a capture
    /// Returns higher score for QxP than PxQ
    fn mvv_lva_score(board: &Board, m: Move) -> i32 {
        if !m.is_capture() {
            return 0;
        }

        let victim = board.piece_at(m.to());
        let attacker = board.piece_at(m.from());

        if victim.is_none() {
            // En passant
            return PIECE_VALUES[PieceType::Pawn.index()];
        }

        let victim_value = PIECE_VALUES[victim.unwrap().piece_type().index()];
        let attacker_value = PIECE_VALUES[attacker.unwrap().piece_type().index()];

        // MVV-LVA: victim value * 10 - attacker value
        // This makes QxP (900*10 - 100 = 8900) > PxQ (100*10 - 900 = 100)
        victim_value * 10 - attacker_value
    }

    pub fn score_move(
        &self,
        board: &Board,
        m: Move,
        ply: usize,
        tt_move: Option<Move>,
    ) -> i32 {
        // 1. TT move
        if Some(m) == tt_move {
            return 10_000_000;
        }

        // 2. Winning captures (MVV-LVA)
        if m.is_capture() {
            return 1_000_000 + Self::mvv_lva_score(board, m);
        }

        // 3. Killers, history (later sessions)

        0
    }
}
```

### Success Criteria

- ✅ MVV-LVA prioritizes QxP over PxQ
- ✅ All captures scored correctly
- ✅ Test suite validates ordering

### Testing

```rust
#[test]
fn test_mvv_lva_ordering() {
    // Position with multiple captures possible
    let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
    let board = parse_fen(fen).unwrap();

    let moves = board.generate_legal_moves();
    let mut move_order = MoveOrder::new();
    let mut ordered = moves.clone();
    move_order.order_moves(&board, &mut ordered, 0, None);

    // Higher value captures should come first
    // QxP should be before PxP, etc.
}
```

---

## Session 4-5: Killer Moves

### Goals

- Track "killer moves" - quiet moves that caused beta cutoffs
- Store 2 killers per ply
- Try killers early (after TT move and captures)

### Technical Design

```rust
impl MoveOrder {
    /// Store a killer move at this ply
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

    /// Check if move is a killer at this ply
    fn is_killer(&self, m: Move, ply: usize) -> bool {
        if ply >= MAX_PLY {
            return false;
        }

        self.killers[ply][0] == Some(m) || self.killers[ply][1] == Some(m)
    }

    pub fn score_move(
        &self,
        board: &Board,
        m: Move,
        ply: usize,
        tt_move: Option<Move>,
    ) -> i32 {
        // 1. TT move
        if Some(m) == tt_move {
            return 10_000_000;
        }

        // 2. Winning captures (MVV-LVA)
        if m.is_capture() {
            return 1_000_000 + Self::mvv_lva_score(board, m);
        }

        // 3. Killer moves
        if !m.is_capture() && self.is_killer(m, ply) {
            return 900_000;
        }

        // 4. History (next session)

        0
    }
}
```

### Integration with Search

```rust
fn negamax(&mut self, board: &Board, depth: i32, mut alpha: i32, beta: i32, ply: u32) -> i32 {
    // ... existing code ...

    for m in legal_moves.iter() {
        let mut new_board = board.clone();
        new_board.make_move(*m);

        let score = -self.negamax(&new_board, depth - 1, -beta, -alpha, ply + 1);

        if score > best_score {
            best_score = score;
            best_move = *m;
        }

        alpha = alpha.max(score);

        if alpha >= beta {
            // Beta cutoff!
            // NEW: Store killer if it's a quiet move
            if !m.is_capture() {
                self.move_order.store_killer(*m, ply as usize);
            }
            break;
        }
    }

    // ... store in TT ...
}
```

### Success Criteria

- ✅ Killers stored correctly on beta cutoffs
- ✅ Killers tried early (after TT move and captures)
- ✅ Measurable improvement in node count
- ✅ Tests validate killer storage/retrieval

---

## Session 6-7: History Heuristic

### Goals

- Track how often moves cause cutoffs across all positions
- Score quiet moves by their historical success
- Butterfly history table: [from_sq][to_sq]

### Technical Design

```rust
impl MoveOrder {
    /// Update history score for a move that caused a cutoff
    pub fn update_history(&mut self, m: Move, depth: i32) {
        if m.is_capture() {
            return; // Only track quiet moves
        }

        let from = m.from().index() as usize;
        let to = m.to().index() as usize;

        // Bonus proportional to depth (deeper = more important)
        let bonus = depth * depth;

        self.history[from][to] += bonus;

        // Prevent overflow - scale down if too large
        if self.history[from][to] > 100_000 {
            for i in 0..64 {
                for j in 0..64 {
                    self.history[i][j] /= 2;
                }
            }
        }
    }

    /// Get history score for a move
    fn history_score(&self, m: Move) -> i32 {
        if m.is_capture() {
            return 0;
        }

        let from = m.from().index() as usize;
        let to = m.to().index() as usize;
        self.history[from][to]
    }

    pub fn score_move(
        &self,
        board: &Board,
        m: Move,
        ply: usize,
        tt_move: Option<Move>,
    ) -> i32 {
        // 1. TT move
        if Some(m) == tt_move {
            return 10_000_000;
        }

        // 2. Winning captures (MVV-LVA)
        if m.is_capture() {
            return 1_000_000 + Self::mvv_lva_score(board, m);
        }

        // 3. Killer moves
        if !m.is_capture() && self.is_killer(m, ply) {
            return 900_000;
        }

        // 4. History heuristic
        if !m.is_capture() {
            return self.history_score(m);
        }

        0
    }
}
```

### Integration with Search

```rust
fn negamax(&mut self, board: &Board, depth: i32, mut alpha: i32, beta: i32, ply: u32) -> i32 {
    // ... existing code ...

    for m in legal_moves.iter() {
        // ... search move ...

        if alpha >= beta {
            // Beta cutoff!
            if !m.is_capture() {
                self.move_order.store_killer(*m, ply as usize);
                self.move_order.update_history(*m, depth);  // NEW
            }
            break;
        }
    }
}
```

### Success Criteria

- ✅ History table updates on cutoffs
- ✅ History scores influence move ordering
- ✅ Scaling prevents overflow
- ✅ Node count improvement over killers alone

---

## Session 8-9: Null Move Pruning

### Goals

- Try "passing" the turn to see if position is still winning
- If null move fails high, position is so good we can skip full search
- Massive speedup in won positions

### Technical Design

```rust
fn negamax(&mut self, board: &Board, depth: i32, mut alpha: i32, beta: i32, ply: u32) -> i32 {
    // ... TT probe ...

    // Null move pruning
    // Conditions:
    // 1. Not in check (illegal to pass when in check)
    // 2. Depth >= 3 (not near leaves)
    // 3. Not in endgame (zugzwang possible)
    // 4. Beta is not a mate score
    const NULL_MOVE_REDUCTION: i32 = 2;  // R=2 (conservative)

    let can_null_move = depth >= 3
        && !board.is_in_check()
        && !is_endgame(board)
        && beta.abs() < MATE_SCORE - 100;

    if can_null_move {
        // Make a "null move" (pass the turn)
        let mut null_board = board.clone();
        null_board.make_null_move();  // Toggle side to move

        // Search at reduced depth
        let null_score = -self.negamax(
            &null_board,
            depth - 1 - NULL_MOVE_REDUCTION,
            -beta,
            -beta + 1,  // Null window
            ply + 1,
        );

        // If null move fails high, we can prune
        if null_score >= beta {
            return beta;  // Fail-high cutoff
        }
    }

    // Leaf node: quiescence
    if depth <= 0 {
        return self.quiesce(board, alpha, beta);
    }

    // ... rest of normal search ...
}
```

### Board Support

```rust
// crates/engine/src/board.rs

impl Board {
    /// Make a null move (pass the turn)
    pub fn make_null_move(&mut self) {
        // Toggle side to move
        self.side_to_move = self.side_to_move.opponent();

        // Update hash
        self.hash ^= zobrist::side_to_move();

        // Reset en passant
        if self.en_passant.is_some() {
            self.hash ^= zobrist::en_passant(self.en_passant.unwrap());
            self.en_passant = None;
        }

        // Increment halfmove clock
        self.halfmove_clock += 1;
    }
}
```

### Success Criteria

- ✅ Null move pruning reduces nodes by 20-40%
- ✅ No tactical oversights (test suite still passes)
- ✅ Disabled in check positions
- ✅ Disabled in endgame (zugzwang)

### Testing

```rust
#[test]
fn test_null_move_pruning() {
    let board = Board::startpos();
    let mut searcher = Searcher::new();

    let result = searcher.search(&board, 6);
    let nodes_with_null = result.nodes;

    // Disable null move and search again
    // (temporarily comment out null move code)
    // let nodes_without_null = ...;

    // Should reduce nodes by at least 20%
    // assert!(nodes_with_null < nodes_without_null * 80 / 100);
}
```

---

## Session 10-12: Late Move Reductions (LMR)

### Goals

- Search first few moves at full depth
- Reduce depth for moves searched late (unlikely to be best)
- Re-search at full depth if reduced search improves alpha
- Most complex optimization but huge gains

### Technical Design

```rust
fn negamax(&mut self, board: &Board, depth: i32, mut alpha: i32, beta: i32, ply: u32) -> i32 {
    // ... TT probe, null move pruning ...

    if depth <= 0 {
        return self.quiesce(board, alpha, beta);
    }

    let mut legal_moves = board.generate_legal_moves();

    if legal_moves.is_empty() {
        // checkmate/stalemate
    }

    // Order moves
    let tt_move = self.tt.probe(board.hash()).map(|e| e.best_move);
    self.move_order.order_moves(board, &mut legal_moves, ply as usize, tt_move);

    let mut best_score = -INFINITY;
    let mut best_move = legal_moves[0];
    let pv_node = beta - alpha > 1;  // PV node has wide window

    for (move_count, m) in legal_moves.iter().enumerate() {
        let mut new_board = board.clone();
        new_board.make_move(*m);

        let mut score;

        // Late Move Reductions (LMR)
        // Conditions for LMR:
        // 1. Not the first few moves (move_count >= 3)
        // 2. Depth is sufficient (depth >= 3)
        // 3. Not a tactical move (not capture, not promotion, not giving check)
        // 4. Not in check
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
                2  // Reduce by 2 plies
            } else {
                1  // Reduce by 1 ply
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
                score = -self.negamax(
                    &new_board,
                    depth - 1,
                    -beta,
                    -alpha,
                    ply + 1,
                );
            }
        } else {
            // First few moves: full depth search

            // PVS (Principal Variation Search) optimization:
            // After first move, do null-window search
            if move_count == 0 {
                // First move: full window
                score = -self.negamax(&new_board, depth - 1, -beta, -alpha, ply + 1);
            } else {
                // Try null window first
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

        if alpha >= beta {
            // Beta cutoff
            if !m.is_capture() {
                self.move_order.store_killer(*m, ply as usize);
                self.move_order.update_history(*m, depth);
            }
            break;
        }
    }

    // Store in TT
    let bound = if best_score >= beta {
        Bound::Lower
    } else if best_score > original_alpha {
        Bound::Exact
    } else {
        Bound::Upper
    };

    self.tt.store(board.hash(), best_move, best_score, depth as u8, bound);

    best_score
}
```

### Success Criteria

- ✅ LMR reduces nodes by 30-50%
- ✅ No tactical oversights (test suite passes)
- ✅ Re-search logic works correctly
- ✅ Combined with null move: 50-70% total reduction

### Testing

```rust
#[test]
fn test_lmr_tactical_accuracy() {
    // Known tactical positions
    let positions = [
        "r1bqkbnr/pppp1ppp/2n5/4p3/2B1P3/5Q2/PPPP1PPP/RNB1K1NR w KQkq - 0 1", // Scholar's mate
        "r1b1kb1r/pppp1ppp/2n2q2/4n3/2B1P3/2N2N2/PPPP1PPP/R1BQK2R w KQkq - 0 1", // Fried Liver
        // ... more tactical positions
    ];

    let mut searcher = Searcher::new();

    for fen in positions {
        let board = parse_fen(fen).unwrap();
        let result = searcher.search(&board, 6);

        // Should still find the correct tactical move
        // (compare with known best moves)
    }
}
```

---

## Session 13: Aspiration Windows

### Goals

- Search with narrow alpha-beta bounds around expected score
- Re-search with wider window if score falls outside
- Improves pruning efficiency

### Technical Design

```rust
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

    // Iterative deepening with aspiration windows
    for depth in 1..=max_depth {
        let score = if depth <= 4 {
            // First few depths: full window
            self.search_root(board, depth)
        } else {
            // Aspiration window: narrow bounds around previous score
            const ASPIRATION_DELTA: i32 = 50;  // 0.5 pawns

            let mut alpha = best_score - ASPIRATION_DELTA;
            let mut beta = best_score + ASPIRATION_DELTA;
            let mut delta = ASPIRATION_DELTA;

            loop {
                let score = self.search_root_window(board, depth, alpha, beta);

                if score <= alpha {
                    // Fail low: widen down
                    alpha -= delta;
                    delta *= 2;
                } else if score >= beta {
                    // Fail high: widen up
                    beta += delta;
                    delta *= 2;
                } else {
                    // Success: score within window
                    break score;
                }

                // Safety: don't widen beyond infinity
                if delta > 1000 {
                    alpha = -INFINITY;
                    beta = INFINITY;
                }
            }
        };

        best_score = score;

        // Extract PV
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
    }
}

fn search_root_window(&mut self, board: &Board, depth: u32, alpha: i32, beta: i32) -> i32 {
    let legal_moves = board.generate_legal_moves();

    if legal_moves.is_empty() {
        return if board.is_in_check() { -MATE_SCORE } else { 0 };
    }

    let mut best_score = -INFINITY;
    let mut best_move = legal_moves[0];
    let mut current_alpha = alpha;

    for m in legal_moves.iter() {
        let mut new_board = board.clone();
        new_board.make_move(*m);

        let score = -self.negamax(&new_board, depth as i32 - 1, -beta, -current_alpha, 1);

        if score > best_score {
            best_score = score;
            best_move = *m;
        }

        current_alpha = current_alpha.max(score);

        if current_alpha >= beta {
            break;
        }
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
```

### Success Criteria

- ✅ Aspiration windows reduce nodes by 5-10%
- ✅ Re-search logic handles fail-high/fail-low correctly
- ✅ No search instability
- ✅ Delta widening prevents infinite loops

---

## Session 14: Multi-PV Search

### Goals

- Find top N best moves instead of just the best
- Useful for analysis mode
- Exclude already-searched moves on subsequent iterations

### Technical Design

```rust
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub best_move: Move,
    pub score: i32,
    pub depth: u32,
    pub nodes: u64,
    pub pv: Vec<Move>,
    pub multi_pv: Vec<PVLine>,  // NEW
}

#[derive(Debug, Clone)]
pub struct PVLine {
    pub score: i32,
    pub pv: Vec<Move>,
}

pub fn search_multi_pv(&mut self, board: &Board, max_depth: u32, num_pv: usize) -> SearchResult {
    self.nodes = 0;
    self.tt.new_search();
    self.move_order.clear();

    let mut multi_pv = Vec::new();
    let mut excluded_moves = Vec::new();

    for pv_num in 0..num_pv {
        // Search with excluded moves
        let result = self.search_excluding(board, max_depth, &excluded_moves);

        multi_pv.push(PVLine {
            score: result.score,
            pv: result.pv.clone(),
        });

        // Exclude this PV's first move from next iteration
        if let Some(&first_move) = result.pv.first() {
            excluded_moves.push(first_move);
        } else {
            break;  // No more legal moves
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

fn search_excluding(&mut self, board: &Board, max_depth: u32, excluded: &[Move]) -> SearchResult {
    // Similar to regular search, but filter out excluded moves
    // ...
}
```

### Success Criteria

- ✅ Finds top N moves correctly
- ✅ No duplicate moves in multi-PV results
- ✅ Scores are reasonable (PV 1 >= PV 2 >= PV 3, etc.)
- ✅ Performance reasonable (N×slower for N PVs)

---

## Session 15: Testing & Benchmarking

### Goals

- Comprehensive test suite for all M4 features
- Performance benchmarks vs M3
- Tactical test positions
- Regression testing

### Test Suite

```rust
// crates/engine/tests/m4_search.rs

#[test]
fn test_move_ordering_effectiveness() {
    // Measure beta cutoff on first move %
}

#[test]
fn test_null_move_reduction() {
    // Verify node count reduction
}

#[test]
fn test_lmr_tactical_accuracy() {
    // Ensure tactical moves still found
}

#[test]
fn test_aspiration_window_stability() {
    // Check for search instability
}

#[test]
fn test_multi_pv_correctness() {
    // Verify top N moves
}

#[test]
fn benchmark_m4_vs_m3() {
    // Compare depth reached in 1 second
    // M3: depth 6-8
    // M4: depth 10-12+ expected
}

#[test]
fn tactical_test_suite() {
    // Bratko-Kopec positions
    // WAC (Win At Chess) positions
    // etc.
}
```

### Performance Benchmarks

```rust
// Create benchmarks/m4_search.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_search_depth(c: &mut Criterion) {
    let board = Board::startpos();
    let mut searcher = Searcher::new();

    c.bench_function("search depth 10", |b| {
        b.iter(|| {
            searcher.search(black_box(&board), 10)
        })
    });
}

fn bench_move_ordering(c: &mut Criterion) {
    // Benchmark move ordering overhead
}

criterion_group!(benches, bench_search_depth, bench_move_ordering);
criterion_main!(benches);
```

---

## File Structure

### New Files

```
crates/engine/src/
├── move_order.rs          (NEW: move ordering logic)
└── tests/
    ├── test_move_order.rs (NEW: move ordering tests)
    ├── test_null_move.rs  (NEW: null move tests)
    ├── test_lmr.rs        (NEW: LMR tests)
    └── tactical_suite.rs  (NEW: tactical positions)

benchmarks/
└── m4_search.rs           (NEW: performance benchmarks)
```

### Modified Files

```
crates/engine/src/
├── lib.rs                 (add move_order module)
├── search.rs              (add null move, LMR, aspiration, multi-PV)
└── board.rs               (add make_null_move method)
```

---

## Success Criteria

### Performance Targets

- ✅ **Depth:** 10-12 in 1 second (vs 6-8 in M3)
- ✅ **Node reduction:** 50-70% fewer nodes vs M3
- ✅ **Move ordering:** Best move searched first >80% of time
- ✅ **Branching factor:** 2-3 effective (vs 5-6 in M3)

### Correctness Targets

- ✅ All 234+ tests passing
- ✅ Tactical test suite: >70% solved at depth 10
- ✅ No search instabilities or infinite loops
- ✅ Deterministic results (same position → same move)

### Code Quality Targets

- ✅ Zero compiler warnings
- ✅ Zero clippy warnings
- ✅ Documented public APIs
- ✅ Comprehensive test coverage

---

## Risk Management

### High Risk

**LMR Tactical Oversights**

- **Risk:** Reducing depth too aggressively misses tactics
- **Mitigation:** Conservative reduction (R=1 for most moves), extensive testing
- **Fallback:** Make LMR optional via feature flag

**Null Move Zugzwang**

- **Risk:** Null move fails in zugzwang positions (endgame)
- **Mitigation:** Disable in endgame, additional verification search
- **Fallback:** Make null move optional via config

### Medium Risk

**Move Ordering Overhead**

- **Risk:** Sorting moves costs more than it saves
- **Mitigation:** Profile and optimize, consider partial sorting
- **Fallback:** Simpler ordering (TT + MVV-LVA only)

**Aspiration Window Instability**

- **Risk:** Re-searches cause time waste
- **Mitigation:** Gradual window widening, safety bounds
- **Fallback:** Use full window

---

## Testing Strategy

### Unit Tests (per session)

- Test each feature in isolation
- Known positions with expected results
- Edge cases (endgame, check, zugzwang)

### Integration Tests (after M4)

- Full search with all features enabled
- Tactical test suites
- Performance regression tests

### Benchmarks

- Compare M3 vs M4 depth at 1 second
- Node count reduction measurement
- Move ordering effectiveness

---

## Expected Timeline

### Optimistic (2 weeks)

- **Week 1:** Sessions 1-9 (move ordering + null move)
- **Week 2:** Sessions 10-15 (LMR + aspiration + testing)

### Realistic (3 weeks)

- **Week 1:** Sessions 1-7 (move ordering infrastructure)
- **Week 2:** Sessions 8-12 (null move + LMR)
- **Week 3:** Sessions 13-15 (aspiration + multi-PV + testing)

### Pessimistic (4 weeks)

- If LMR bugs are hard to debug
- If null move causes zugzwang issues
- If extensive refactoring needed

---

## Commit Strategy

### Per Feature

- **Commit 1:** Move ordering infrastructure (Sessions 1-2)
- **Commit 2:** MVV-LVA (Session 3)
- **Commit 3:** Killer moves (Sessions 4-5)
- **Commit 4:** History heuristic (Sessions 6-7)
- **Commit 5:** Null move pruning (Sessions 8-9)
- **Commit 6:** Late move reductions (Sessions 10-12)
- **Commit 7:** Aspiration windows (Session 13)
- **Commit 8:** Multi-PV search (Session 14)
- **Commit 9:** Testing & benchmarks (Session 15)
- **Commit 10:** M4 completion documentation

### Branch Strategy

```bash
git checkout -b feature/m4-advanced-search
# ... implement sessions 1-15 ...
git push origin feature/m4-advanced-search
# ... create PR, review, merge to main ...
```

---

## Integration with Future Milestones

### M5 Dependencies

M5 (Time Management & UCI) will benefit from M4:

- **UCI info output:** Nodes, depth, PV from M4
- **Time management:** Needs efficient search from M4
- **Multi-PV:** Supports UCI MultiPV option

### M6 Dependencies

M6 (Advanced Eval) will benefit from M4:

- **Move ordering:** Can incorporate eval-based scores
- **Null move:** Endgame detection from eval
- **Search depth:** Deeper search validates better eval

---

## Next Steps

1. ✅ Review and approve this M4 plan
2. ✅ Create feature branch: `git checkout -b feature/m4-advanced-search`
3. ✅ Begin Session 1: Move ordering infrastructure
4. Commit after each major feature
5. Test continuously (run full test suite after each session)
6. Benchmark at end to verify performance gains

---

**Ready to start M4 Session 1?**

Let me know if you want to proceed, or if you'd like any clarifications on the plan!
