# Remaining Work Analysis - Post M8

## Current Status (2025-10-26)

### ✅ Completed Milestones

| Milestone | Status      | Description                                                          |
| --------- | ----------- | -------------------------------------------------------------------- |
| M0        | ✅ Complete | Repository setup and tooling                                         |
| M1        | ✅ Complete | Shared protocol contract                                             |
| M2        | ✅ Complete | Engine core (bitboards, move generation, perft)                      |
| M3        | ✅ Complete | Search v1 (alpha-beta, iterative deepening)                          |
| M4        | ✅ Complete | Evaluation v1 (material, PST, positional)                            |
| M5        | ✅ Complete | Advanced features (time management, move ordering)                   |
| M6        | ✅ Complete | Advanced evaluation (king safety, pawn structure)                    |
| M7        | ✅ Complete | Advanced search (SEE, extensions, pruning, IID/IIR)                  |
| M8        | ✅ Complete | WASM integration (build, worker, client, performance, cross-browser) |

### ❌ The Critical Gap

**Problem**: The `EngineImpl::analyze()` method in `crates/engine/src/lib.rs` is still a scaffold!

```rust
// Current implementation (SCAFFOLD)
pub fn analyze<F>(&mut self, limit: SearchLimit, mut info_sink: F) -> BestMove
where
    F: FnMut(SearchInfo),
{
    self.stopped = false;
    // Dummy iterative deepening loop for scaffold
    let mut nodes = 0u64;
    for depth in 1..=match limit {
        SearchLimit::Depth { depth } => depth,
        _ => 6,
    } {
        // ... fake search info ...
    }
    BestMove {
        id: "scaffold".into(),
        best: "e2e4".into(),  // ❌ Always returns e2e4!
        ponder: Some("e7e5".into()),
    }
}
```

**Reality**: The **ACTUAL search engine IS fully implemented** in `crates/engine/src/search/core.rs`!

```rust
// What's ALREADY implemented:
pub struct Searcher {
    evaluator: Evaluator,
    tt: TranspositionTable,
    move_order: MoveOrder,
    nodes: u64,
    time_manager: Option<TimeManager>,
    stopped: bool,
}

impl Searcher {
    pub fn search_with_limit(
        &mut self,
        board: &Board,
        max_depth: u32,
        time_control: TimeControl,
    ) -> SearchResult { /* ... fully implemented ... */ }
}
```

**The search includes**:

- ✅ Alpha-beta pruning
- ✅ Iterative deepening
- ✅ Quiescence search
- ✅ Transposition table
- ✅ Move ordering (MVV-LVA, killers, history, countermoves)
- ✅ Late move reductions (LMR)
- ✅ Null move pruning
- ✅ Futility pruning, razoring, reverse futility
- ✅ SEE (Static Exchange Evaluation)
- ✅ Extensions (check, recapture, passed pawn)
- ✅ IID/IIR
- ✅ Time management
- ✅ Material + PST evaluation
- ✅ King safety, pawn structure, mobility

**Performance**: 10,470 nodes/sec (debug), 26M+ nodes/sec perft

---

## M9: Wire Up Real Engine (The ACTUAL Next Step)

### Objective

Connect the existing `Searcher` to the `EngineImpl` interface so the WASM/server/CLI actually use the real engine instead of returning "e2e4".

### Current Architecture

```
┌─────────────────────────────────────────┐
│     WASM / Web Worker / Server          │
│     (apps/web, crates/engine-server)    │
└──────────────────┬──────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────┐
│        EngineImpl::analyze()            │  ❌ SCAFFOLD
│        (crates/engine/src/lib.rs)       │  Returns "e2e4"
└─────────────────────────────────────────┘

         NOT CONNECTED TO ↓

┌─────────────────────────────────────────┐
│        Searcher::search_with_limit()    │  ✅ REAL ENGINE
│        (crates/engine/src/search/core.rs)│  Fully functional
└─────────────────────────────────────────┘
```

### Required Changes

**1. Update `EngineImpl` in `crates/engine/src/lib.rs`:**

```rust
pub struct EngineImpl {
    pub opts: EngineOptions,
    pub current_fen: String,
    stopped: bool,
    searcher: Searcher,  // ← ADD THIS
}

impl EngineImpl {
    pub fn new_with(opts: EngineOptions) -> Self {
        let tt_size = opts.hash_size_mb.unwrap_or(128);
        Self {
            opts,
            current_fen: "startpos".to_string(),
            stopped: false,
            searcher: Searcher::with_tt_size(tt_size),  // ← ADD THIS
        }
    }

    pub fn analyze<F>(&mut self, limit: SearchLimit, mut info_sink: F) -> BestMove
    where
        F: FnMut(SearchInfo),
    {
        // Parse FEN to Board
        let board = parse_fen(&self.current_fen).expect("Invalid FEN");

        // Convert SearchLimit to (max_depth, TimeControl)
        let (max_depth, time_control) = match limit {
            SearchLimit::Depth { depth } => (depth, TimeControl::Infinite),
            SearchLimit::Nodes { nodes } => (MAX_DEPTH, TimeControl::Nodes(nodes)),
            SearchLimit::Time { time_ms } => (MAX_DEPTH, TimeControl::MoveTime(time_ms)),
            // ... handle other limits
        };

        // ← CALL THE REAL SEARCH
        let result = self.searcher.search_with_limit(&board, max_depth, time_control);

        // Convert SearchResult to BestMove
        BestMove {
            id: "real-engine".into(),  // ← Use request ID
            best: result.best_move.to_string(),
            ponder: None,  // TODO: extract from PV
        }
    }
}
```

**2. Stream Search Info:**

Currently `search_with_limit` doesn't call the callback. Need to add callback support:

```rust
// Option A: Modify Searcher to accept callback
pub fn search_with_limit_callback<F>(
    &mut self,
    board: &Board,
    max_depth: u32,
    time_control: TimeControl,
    mut callback: F,  // ← ADD THIS
) -> SearchResult
where
    F: FnMut(SearchInfo),
{
    // ... in iterative deepening loop:
    for depth in 1..=max_depth {
        let result = self.search(board, depth);

        // ← CALL CALLBACK
        callback(SearchInfo {
            id: "real".into(),
            depth,
            nodes: self.nodes,
            nps: calculate_nps(),
            time_ms: elapsed_ms,
            score: convert_score(result.score),
            pv: result.pv.iter().map(|m| m.to_string()).collect(),
            // ...
        });
    }
}
```

**3. Handle Stop Signal:**

```rust
impl EngineImpl {
    pub fn stop(&mut self) {
        self.stopped = true;
        self.searcher.stop();  // ← ADD THIS
    }
}
```

**4. Update WASM Bridge:**

No changes needed! The WASM bridge already calls `EngineImpl::analyze()`.

---

## Session Plan: M9 Wire-Up (1-2 sessions)

### Session 1: Basic Integration

**Tasks:**

1. Add `Searcher` field to `EngineImpl`
2. Wire up `analyze()` to call `search_with_limit()`
3. Convert between protocol types and engine types
4. Test basic search works (depth 1-6)
5. Verify WASM integration still works

**Deliverables:**

- Modified `crates/engine/src/lib.rs`
- Tests for EngineImpl::analyze()
- WASM build works
- Returns real moves (not "e2e4")

**Success Criteria:**

- WASM engine finds different moves for different positions
- Depth 6 search completes in <5 seconds
- No regressions in M8 integration tests

---

### Session 2: Search Info Streaming & Polish

**Tasks:**

1. Add callback support to Searcher
2. Stream SearchInfo during iterative deepening
3. Handle all SearchLimit variants (depth, time, nodes, infinite)
4. Add proper stop handling
5. Extract ponder move from PV
6. Update tests

**Deliverables:**

- SearchInfo streaming working
- All limit types supported
- Stop functionality working
- Ponder moves included
- Updated integration tests

**Success Criteria:**

- UI shows depth-by-depth progress
- All 48 integration tests still pass
- Finds mate-in-2 positions correctly
- Stops cleanly when requested

---

## Post-M9 Remaining Work

### High Priority

**M10: UCI Server Integration (1 week)**

- Implement full UCI protocol in server
- Add command parsing (position, go, stop, etc.)
- WebSocket streaming of search info
- Server deployment configuration

**M11: Opening Book (1 week)**

- Download/create opening book (Polyglot format)
- Book probe in search
- Book moves in UI
- Configurable book usage

**M12: Endgame Tablebases (1 week)**

- Integrate Syzygy tablebases
- Probe during search
- Display tablebase hits in UI
- Download/configure TB files

### Medium Priority

**M13: Advanced Time Management**

- Better time allocation
- Increment handling
- Panic time mode
- Time-based move ordering

**M14: Multi-PV Analysis**

- Show top N lines
- UI for multi-PV display
- Configurable depth per line

**M15: Analysis Features**

- Infinite analysis mode
- Analyze game (all positions)
- Export analysis to PGN
- Evaluation graph

### Low Priority

**M16: Tuning**

- Automated parameter tuning (Texel's method)
- Evaluation weight optimization
- Search parameter tuning

**M17: Testing & Quality**

- Extended tactical test suite (>100 positions)
- Chess960 support
- Additional perft positions
- Fuzzing

**M18: Performance**

- SIMD optimization
- Parallel search (Lazy SMP)
- Profile-guided optimization

**M19: UI Polish**

- Board editor
- Game database
- PGN import/export
- Analysis board

**M20: Deployment**

- Production server setup
- CDN for WASM files
- Analytics
- User accounts (optional)

---

## Immediate Next Steps

### Step 1: Complete M9 Wire-Up (2 sessions, 1-2 days)

This is **CRITICAL** - without this, all the work on M2-M7 is unused!

**Commands:**

```bash
# Start M9 Session 1
cd crates/engine
cargo test  # Verify current state

# Make changes to lib.rs
# Test locally
cargo test

# Test WASM build
cd ../../
pnpm build:wasm

# Test in browser
cd apps/web
pnpm dev
# Select WASM mode, analyze position
```

### Step 2: Validate Real Engine Works

**Test Checklist:**

- [ ] Starting position doesn't return e2e4 for all depths
- [ ] Finds mate in 1 (Scholar's mate position)
- [ ] Tactical position (fork/pin/skewer) finds correct move
- [ ] Search depth increases show different scores
- [ ] SearchInfo updates appear in UI
- [ ] Stop button works
- [ ] Performance metrics track real search
- [ ] All 48 integration tests pass

### Step 3: Deploy & Celebrate! 🎉

Once M9 is complete, you have a **fully functional chess engine** running in the browser!

---

## Summary

**What's Done:** M0-M8 (8 milestones complete!)

- ✅ Complete search engine with all modern techniques
- ✅ WASM integration infrastructure
- ✅ Web worker, performance monitoring, cross-browser support
- ✅ 365 passing tests

**What's NOT Done:** The 10-line connection between them!

- ❌ `EngineImpl::analyze()` doesn't call `Searcher::search_with_limit()`

**The Fix:** M9 (1-2 sessions, ~200 LOC)

- Connect EngineImpl to Searcher
- Add callback streaming
- Handle all limit types
- Test & validate

**After M9:** Feature additions (opening book, tablebases, UCI, etc.)

---

**Status:** 📋 Ready for M9 Session 1
**Estimated Time:** 2-4 hours
**Complexity:** Low (mostly plumbing)
**Impact:** HIGH (makes engine actually work!)

**Last Updated:** 2025-10-26
