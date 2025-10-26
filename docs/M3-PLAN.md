# M3: Search & Evaluation - Technical Plan

**Milestone:** M3 - Search & Evaluation
**Prerequisites:** ✅ M2 Engine Core (complete)
**Estimated Sessions:** 15-20

## Overview

M3 will implement a competitive chess search engine with evaluation, alpha-beta pruning, transposition tables, iterative deepening, and quiescence search. The goal is to build an engine that can search to depth 6-8 in reasonable time and play tactically sound chess.

## Architecture Overview

```
Search Framework
├── Evaluation (static position scoring)
├── Negamax with Alpha-Beta (minimax search)
├── Transposition Table (position caching)
├── Iterative Deepening (progressive depth)
├── Quiescence Search (tactical extension)
└── Time Management (search control)
```

## Component Breakdown

### 1. Evaluation Function (Sessions 1-3)

**Purpose:** Assign a numeric score to any position from the current side's perspective.

**Components:**

- **Material counting** (base scoring)
  - Pawn: 100, Knight: 320, Bishop: 330, Rook: 500, Queen: 900
  - King: 20000 (effectively infinite)

- **Piece-Square Tables (PST)** (positional bonuses)
  - 6 piece types × 2 colors × 64 squares = pre-computed tables
  - Encourage piece development, central control
  - Different tables for opening/middlegame vs endgame

- **Basic positional factors**
  - Pawn structure (doubled, isolated, passed pawns)
  - King safety (pawn shield, open files near king)
  - Mobility (number of legal moves)
  - Piece coordination bonuses

**Data Structures:**

```rust
pub struct Evaluator {
    pst: PieceSquareTables,
}

pub struct PieceSquareTables {
    // [piece_type][square] for each color
    mg_tables: [[i32; 64]; 6],  // middlegame
    eg_tables: [[i32; 64]; 6],  // endgame
}

impl Board {
    pub fn evaluate(&self, eval: &Evaluator) -> i32;
}
```

**Session Plan:**

- **Session 1:** Material counting + basic PST framework
- **Session 2:** Full PST values (tuned for all pieces)
- **Session 3:** Basic positional evaluation (pawn structure, mobility)

---

### 2. Negamax Search with Alpha-Beta (Sessions 4-6)

**Purpose:** Search the game tree to find the best move using minimax with alpha-beta pruning.

**Core Algorithm:**

```rust
fn negamax(board: &mut Board, depth: i32, mut alpha: i32, beta: i32) -> i32 {
    // Base case: leaf node
    if depth == 0 {
        return quiesce(board, alpha, beta);  // Will implement later
    }

    let moves = board.generate_legal_moves();

    // Checkmate/stalemate detection
    if moves.is_empty() {
        return if board.is_in_check() {
            -MATE_SCORE + ply  // Checkmate
        } else {
            0  // Stalemate
        };
    }

    let mut best_score = -INFINITY;

    for m in moves.iter() {
        let undo = board.make_move(*m);
        let score = -negamax(board, depth - 1, -beta, -alpha);
        board.unmake_move(*m, undo);

        best_score = best_score.max(score);
        alpha = alpha.max(score);

        if alpha >= beta {
            break;  // Beta cutoff
        }
    }

    best_score
}
```

**Key Concepts:**

- **Negamax:** Simplified minimax using negation (one function instead of min/max)
- **Alpha-Beta Pruning:** Skip branches that can't affect the final result
  - Alpha: best score we can guarantee (lower bound)
  - Beta: opponent's best alternative (upper bound)
  - Cutoff when alpha >= beta
- **Ply tracking:** Distance from root (for mate scoring)

**Data Structures:**

```rust
pub struct SearchInfo {
    pub nodes: u64,
    pub depth: u32,
    pub seldepth: u32,  // Selective depth (quiescence)
    pub time_ms: u64,
    pub pv: Vec<Move>,   // Principal variation
}

pub struct Searcher {
    evaluator: Evaluator,
    tt: TranspositionTable,  // Will add later
}

impl Searcher {
    pub fn search(&mut self, board: &Board, depth: u32) -> SearchResult;
}
```

**Session Plan:**

- **Session 4:** Basic negamax framework (no pruning)
- **Session 5:** Alpha-beta pruning implementation
- **Session 6:** Move ordering (MVV-LVA for captures, killers later)

---

### 3. Transposition Table (Sessions 7-8)

**Purpose:** Cache previously searched positions to avoid redundant work.

**Hash Table Design:**

```rust
pub struct TTEntry {
    hash: u64,           // Zobrist hash (for verification)
    best_move: Move,     // Best move found
    score: i32,          // Evaluation score
    depth: u8,           // Search depth
    bound: Bound,        // Exact, LowerBound, or UpperBound
    age: u8,             // Generation counter
}

pub enum Bound {
    Exact,       // PV node (exact score)
    LowerBound,  // Beta cutoff (score >= beta)
    UpperBound,  // Alpha node (score <= alpha)
}

pub struct TranspositionTable {
    entries: Vec<TTEntry>,
    size: usize,  // Number of entries (power of 2)
    generation: u8,
}
```

**TT Integration:**

```rust
fn negamax(board: &mut Board, depth: i32, mut alpha: i32, beta: i32, tt: &mut TT) -> i32 {
    // Probe TT
    if let Some(entry) = tt.probe(board.hash()) {
        if entry.depth >= depth {
            match entry.bound {
                Bound::Exact => return entry.score,
                Bound::LowerBound => alpha = alpha.max(entry.score),
                Bound::UpperBound => beta = beta.min(entry.score),
            }
            if alpha >= beta {
                return entry.score;
            }
        }
    }

    // ... search ...

    // Store in TT
    let bound = if score >= beta {
        Bound::LowerBound
    } else if score > original_alpha {
        Bound::Exact
    } else {
        Bound::UpperBound
    };

    tt.store(board.hash(), best_move, score, depth, bound);

    score
}
```

**Session Plan:**

- **Session 7:** TT data structure and basic probe/store
- **Session 8:** TT integration with search + replacement scheme

---

### 4. Iterative Deepening (Sessions 9-10)

**Purpose:** Search progressively deeper depths, using results to improve move ordering.

**Core Loop:**

```rust
pub fn iterative_deepening(&mut self, board: &Board, max_depth: u32) -> SearchResult {
    let mut best_move = Move::NULL;
    let mut best_score = 0;

    for depth in 1..=max_depth {
        let score = self.negamax(board, depth as i32, -INFINITY, INFINITY);

        // Extract PV from TT
        let pv = self.extract_pv(board, depth);

        if let Some(mv) = pv.first() {
            best_move = *mv;
            best_score = score;
        }

        // Print UCI info
        println!("info depth {} score cp {} nodes {} pv {}",
                 depth, score, self.nodes, format_pv(&pv));
    }

    SearchResult { best_move, score: best_score, .. }
}
```

**Benefits:**

- Better move ordering (use best move from previous depth first)
- Time management (can stop between depths)
- Early results (can play best move from depth N-1 if time runs out)
- Aspiration windows (optimization for later)

**Session Plan:**

- **Session 9:** Iterative deepening framework
- **Session 10:** PV extraction from TT + UCI info output

---

### 5. Quiescence Search (Sessions 11-13)

**Purpose:** Extend search at leaf nodes to avoid horizon effect (missing tactics).

**Algorithm:**

```rust
fn quiesce(board: &mut Board, mut alpha: i32, beta: i32) -> i32 {
    let stand_pat = board.evaluate(&self.evaluator);

    // Stand pat: assume we can at least maintain current position
    if stand_pat >= beta {
        return beta;
    }

    if stand_pat > alpha {
        alpha = stand_pat;
    }

    // Generate only tactical moves (captures, maybe checks)
    let moves = board.generate_captures();

    // Delta pruning: skip moves that can't improve position
    let big_delta = 900 + 200;  // Queen value + margin
    if stand_pat + big_delta < alpha {
        return alpha;  // Futile
    }

    for m in moves.iter() {
        // SEE pruning: skip losing captures
        if see(board, m) < 0 {
            continue;
        }

        let undo = board.make_move(*m);
        let score = -quiesce(board, -beta, -alpha);
        board.unmake_move(*m, undo);

        if score >= beta {
            return beta;
        }

        alpha = alpha.max(score);
    }

    alpha
}
```

**Key Concepts:**

- **Stand pat:** Can choose to not search further
- **Tactical moves only:** Captures (and promotions, maybe checks)
- **Delta pruning:** Skip moves that can't improve position enough
- **SEE (Static Exchange Evaluation):** Skip obviously losing captures

**Session Plan:**

- **Session 11:** Basic quiescence (stand pat + capture search)
- **Session 12:** Delta pruning + SEE (simple version)
- **Session 13:** Integration with main search

---

### 6. Move Ordering (Sessions 14-15)

**Purpose:** Search best moves first for maximum alpha-beta cutoffs.

**Ordering Priority:**

1. **TT move** (best move from previous search/depth)
2. **Winning captures** (MVV-LVA: Most Valuable Victim - Least Valuable Attacker)
3. **Killer moves** (quiet moves that caused cutoffs at same ply)
4. **History heuristic** (quiet moves that caused cutoffs historically)
5. **Losing captures** (bad captures from SEE)
6. **Remaining quiet moves**

**Data Structures:**

```rust
pub struct MoveOrdering {
    killers: [[Move; 2]; MAX_PLY],  // 2 killers per ply
    history: [[i32; 64]; 64],        // [from][to] scores
}

impl MoveOrdering {
    pub fn score_move(&self, m: Move, tt_move: Option<Move>, board: &Board) -> i32 {
        // TT move gets highest score
        if Some(m) == tt_move {
            return 10_000_000;
        }

        // Captures: MVV-LVA
        if m.is_capture() {
            let victim = board.piece_at(m.to()).unwrap();
            let attacker = board.piece_at(m.from()).unwrap();
            return 1_000_000 + mvv_lva(victim, attacker);
        }

        // Killers
        if self.is_killer(m, ply) {
            return 900_000;
        }

        // History
        self.history[m.from().index()][m.to().index()]
    }
}
```

**Session Plan:**

- **Session 14:** MVV-LVA capture ordering + move scoring
- **Session 15:** Killer moves + history heuristic

---

### 7. Time Management & Testing (Sessions 16-18)

**Purpose:** Control how long to search based on time constraints.

**Time Control:**

```rust
pub struct TimeManager {
    start_time: Instant,
    allocated_ms: u64,
    max_ms: u64,
}

impl TimeManager {
    pub fn should_stop(&self) -> bool {
        self.start_time.elapsed().as_millis() as u64 >= self.allocated_ms
    }

    pub fn allocate_time(&mut self, tc: &TimeControl, moves_played: u32) {
        match tc {
            TimeControl::Infinite => self.allocated_ms = u64::MAX,
            TimeControl::Depth(d) => self.allocated_ms = u64::MAX,
            TimeControl::MoveTime(ms) => self.allocated_ms = *ms,
            TimeControl::Clock { time, inc, movestogo } => {
                // Allocate time based on remaining time + increment
                let time_left = *time;
                let moves_to_go = movestogo.unwrap_or(40 - moves_played.min(40));
                self.allocated_ms = (time_left / moves_to_go) + inc / 2;
                self.max_ms = time_left / 2;  // Never use more than half
            }
        }
    }
}
```

**Session Plan:**

- **Session 16:** Time manager implementation
- **Session 17:** Integration with iterative deepening
- **Session 18:** Testing suite (tactical puzzles, perft, self-play)

---

## Testing Strategy

### Unit Tests

- Evaluation function correctness
- Search finds forced mates (mate in 1, 2, 3)
- TT probe/store correctness
- Move ordering correctness

### Integration Tests

- Tactical test suites (WAC, BK tests)
- Known positions (should find best move)
- Perft from various depths (verify search doesn't break move gen)

### Performance Tests

- Nodes per second (target: >1M nps with TT)
- Search depth in fixed time
- Branching factor (alpha-beta effectiveness)

---

## Success Criteria

**M3 is complete when:**

- ✅ Engine can evaluate any position
- ✅ Search finds tactical wins (mate in 3-5)
- ✅ Transposition table reduces node count >50%
- ✅ Alpha-beta achieves <10% of full minimax nodes
- ✅ Can search to depth 6-8 in <1 second (starting position)
- ✅ Passes tactical test suite (>50% correct)
- ✅ Clean API for search control (depth, time, infinite)
- ✅ All tests passing

---

## File Structure

```
crates/engine/src/
├── eval.rs              # Evaluation function
├── eval/
│   ├── material.rs      # Material counting
│   ├── pst.rs           # Piece-square tables
│   └── positional.rs    # Pawn structure, king safety, etc.
├── search.rs            # Main search logic
├── search/
│   ├── negamax.rs       # Core negamax algorithm
│   ├── quiesce.rs       # Quiescence search
│   ├── ordering.rs      # Move ordering
│   └── time.rs          # Time management
├── tt.rs                # Transposition table
└── see.rs               # Static Exchange Evaluation

crates/engine/tests/
├── eval_tests.rs        # Evaluation tests
├── search_tests.rs      # Search tests (mate in N)
├── tactical_tests.rs    # WAC/BK tactical suite
└── bench_tests.rs       # Performance benchmarks
```

---

## Performance Targets

| Metric                 | Target  | Stretch Goal |
| ---------------------- | ------- | ------------ |
| Nodes/sec (with TT)    | 1M nps  | 5M nps       |
| Depth in 1s (startpos) | Depth 6 | Depth 8      |
| Branching factor       | <3.5    | <3.0         |
| Tactical accuracy      | >50%    | >80%         |
| Mate in 3              | <1s     | <0.1s        |

---

## Dependencies on M2

**Already implemented in M2:**

- ✅ Move generation (all we need for search)
- ✅ Make/unmake moves (reversibility critical)
- ✅ Legal move validation
- ✅ Zobrist hashing (critical for TT)
- ✅ FEN parsing (for test positions)
- ✅ Check detection (for mate scoring)

**M2 provides perfect foundation for M3!**

---

## Next: M4 Preview

After M3, M4 will focus on advanced search techniques:

- Null move pruning
- Late move reductions
- Aspiration windows
- Multi-PV search
- Opening book
- Endgame tablebase probing

But first, let's build a solid search & evaluation foundation in M3!
