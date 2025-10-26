# Chess Engine: M4+ Implementation Plan

**Date:** 2025-10-26
**Status:** M1 ✅ M2 ✅ M3 ✅ Complete

---

## Current State Assessment

### ✅ Completed (M1-M3)

**M1-M2: Engine Core**

- Bitboard representation
- Move generation (26M nps)
- Zobrist hashing
- FEN parsing/serialization
- Perft validation (234 tests passing)
- Make/unmake moves

**M3: Search & Evaluation**

- Position evaluation (material + PST + mobility)
- Negamax with alpha-beta pruning
- Transposition table (64 MB, Zobrist-based)
- Iterative deepening
- Quiescence search (horizon effect prevention)
- Principal variation extraction
- Mate detection (checkmate/stalemate)
- Performance: Depth 6-8 in <1s

### 🔶 Partially Complete

**Move Ordering** (basic only)

- ✅ Legal move generation
- ❌ MVV-LVA (Most Valuable Victim - Least Valuable Attacker)
- ❌ Killer moves (2 per ply)
- ❌ History heuristic
- ❌ Counter moves
- ❌ TT move prioritization

**Time Management** (none)

- ❌ Intelligent time allocation
- ❌ Time per move calculations
- ❌ Panic time handling
- ❌ Depth-based estimates

**Evaluation** (basic only)

- ✅ Material counting
- ✅ Piece-square tables (MG/EG)
- ✅ Mobility
- ❌ Pawn structure (doubled, isolated, passed, chains)
- ❌ King safety (pawn shield, attacks)
- ❌ Rook on open files
- ❌ Bishop pairs
- ❌ Knight outposts

### 📋 Not Started

- UCI protocol
- Opening book
- Endgame tablebases
- Parameter tuning
- WASM compilation
- Frontend UI
- Server mode

---

## Two Proposed Tracks

We have two viable paths forward. You should choose based on your priorities:

### 🎯 Track A: Engine Strength First (Recommended for Chess Engine Focus)

**Goal:** Build a strong, complete chess engine before web integration

**Milestones:**

- **M4:** Advanced Search (null move, LMR, move ordering)
- **M5:** Time Management & UCI Protocol
- **M6:** Advanced Evaluation (pawn structure, king safety)
- **M7:** Opening Book & Endgame Tablebases
- **M8:** Tuning & Strength Testing
- **M9:** WASM & Web Integration
- **M10:** Server Mode & Deployment

**Pros:**

- Complete, strong engine first
- Can play against other engines via UCI
- More focused development
- Natural progression of complexity

**Cons:**

- No web UI until M9
- Longer path to deployment

---

### 🌐 Track B: Web Integration First (Recommended for Product Focus)

**Goal:** Get a working web app quickly, then improve strength

**Milestones:**

- **M4:** Time Management (basic)
- **M5:** WASM Bridge & Web Worker
- **M6:** Frontend React MVP
- **M7:** Server Mode (optional)
- **M8:** Advanced Search (null move, LMR, move ordering)
- **M9:** Advanced Evaluation (pawn structure, king safety)
- **M10:** UCI & Testing Against Other Engines

**Pros:**

- Working web app in ~4-6 weeks
- Early user testing possible
- Visual progress
- Parallelizable (frontend + backend)

**Cons:**

- Engine will be weaker initially
- May need to refactor WASM integration as engine evolves

---

## Track A Details: Engine Strength First ⭐ RECOMMENDED

This is the recommended track for building a strong chess engine.

---

### M4: Advanced Search Techniques (2-3 weeks)

**Goal:** Dramatically improve search efficiency and depth

#### Features to Implement

**1. Null Move Pruning**

- Try "passing" the turn to see if position is still winning
- Skip full search if null move fails high
- Reduction factor R=2 or R=3
- Massive node reduction in won positions

**2. Late Move Reductions (LMR)**

- Reduce depth for moves searched late (unlikely to be best)
- Re-search at full depth if score improves
- Typically saves 30-50% of nodes

**3. Move Ordering Improvements**

- **TT move first** (from transposition table)
- **MVV-LVA** for captures (Queen takes Pawn before Pawn takes Queen)
- **Killer moves** (2 per ply - quiet moves that caused cutoffs)
- **History heuristic** (move success history across positions)
- **Counter moves** (moves that refute the opponent's last move)

**4. Aspiration Windows**

- Search with tight alpha-beta bounds around expected score
- Re-search if score falls outside window
- Improves pruning efficiency

**5. Multi-PV Search** (optional)

- Find top N best moves instead of just best
- Useful for analysis mode

#### Success Criteria

- ✅ Reaches depth 10+ in <5s on startpos (vs depth 6-8 currently)
- ✅ Null move reduces nodes by 20-40%
- ✅ Move ordering: best move is first in >80% of positions
- ✅ LMR doesn't cause tactical oversights (verified with test suite)
- ✅ All 234+ tests still passing

#### Files to Create/Modify

```
crates/engine/src/
├── search.rs              (add null move, LMR, aspiration)
├── move_order.rs          (NEW: MVV-LVA, killers, history)
├── history.rs             (NEW: history table)
└── tests/
    ├── test_null_move.rs  (NEW)
    ├── test_lmr.rs        (NEW)
    └── test_ordering.rs   (NEW)
```

#### Estimated Time

- **Optimistic:** 2 weeks
- **Realistic:** 3 weeks
- **Pessimistic:** 4 weeks (if bugs in LMR/null move)

---

### M5: Time Management & UCI Protocol (2 weeks)

**Goal:** Make engine playable and interoperable with chess GUIs

#### Features to Implement

**1. Time Management**

- Parse `go` command time controls (wtime, btime, movestogo, movetime)
- Allocate time per move intelligently
- Emergency time handling (panic mode)
- Soft/hard time limits
- Time per depth estimates

**2. UCI Protocol**

- `uci` - identify engine
- `isready` / `readyok` - synchronization
- `ucinewgame` - reset state
- `position` - set position (FEN or startpos + moves)
- `go` - start search with limits
- `stop` - stop search
- `quit` - exit
- `setoption` - configure engine
- `info` - search information output (depth, score, pv, nodes, nps, time, hashfull)
- `bestmove` - result output

**3. UCI Options**

- Hash size
- Threads (for future)
- MultiPV
- Contempt (for future)

#### Success Criteria

- ✅ Works with Arena, CuteChess, Lichess, etc.
- ✅ Respects time controls (doesn't lose on time)
- ✅ Responds to `stop` within 50ms
- ✅ UCI protocol compliance (validated with cutechess-cli)
- ✅ Search info updates every ~500ms

#### Files to Create/Modify

```
crates/engine/src/
├── uci.rs                 (NEW: UCI protocol handler)
├── time.rs                (NEW: time management)
└── search.rs              (modify: add time checks, info output)

crates/engine/bin/
└── uci_main.rs            (NEW: UCI binary entry point)
```

#### Estimated Time

- **Optimistic:** 1.5 weeks
- **Realistic:** 2 weeks
- **Pessimistic:** 3 weeks

---

### M6: Advanced Evaluation (2-3 weeks)

**Goal:** Positional understanding beyond material

#### Features to Implement

**1. Pawn Structure**

- **Doubled pawns** (penalty)
- **Isolated pawns** (penalty, especially in endgame)
- **Backward pawns** (penalty)
- **Passed pawns** (bonus, increases by rank)
- **Pawn chains** (bonus for protected pawns)
- **Pawn islands** (fewer is better)
- **Pawn hash table** (cache pawn eval with Zobrist key)

**2. King Safety**

- **Pawn shield** (bonus for pawns in front of king)
- **King attackers** (count enemy pieces attacking king zone)
- **Attack weight** (different pieces have different attack values)
- **Open files near king** (penalty)
- **King tropism** (enemy pieces near king)

**3. Piece Activity**

- **Rook on open file** (bonus)
- **Rook on semi-open file** (smaller bonus)
- **Rook on 7th rank** (bonus)
- **Bishop pair** (bonus ~50cp)
- **Bad bishop** (penalty if pawns on same color)
- **Knight outposts** (protected square in enemy territory)
- **Trapped pieces** (penalty for trapped bishops/rooks)

**4. Game Phase Interpolation**

- Smooth middlegame → endgame transition
- Based on remaining material
- Different evaluation weights per phase

#### Success Criteria

- ✅ Mirror-board symmetry tests pass
- ✅ Evaluation time <30µs per call
- ✅ Positional play improves (verified with test positions)
- ✅ Pawn hash table gives 20%+ speedup on pawn eval
- ✅ Strength increase measurable (win rate vs M5 version)

#### Files to Create/Modify

```
crates/engine/src/eval/
├── mod.rs                 (orchestration)
├── material.rs            (existing)
├── pst.rs                 (existing)
├── positional.rs          (existing, expand mobility)
├── pawns.rs               (NEW: pawn structure + hash)
├── king.rs                (NEW: king safety)
├── pieces.rs              (NEW: rook/bishop/knight bonuses)
└── phase.rs               (NEW: game phase calculation)
```

#### Estimated Time

- **Optimistic:** 2 weeks
- **Realistic:** 3 weeks
- **Pessimistic:** 4 weeks (if tuning reveals issues)

---

### M7: Opening Book & Endgame Tablebases (1-2 weeks)

**Goal:** Add chess knowledge to engine

#### Features to Implement

**1. Opening Book**

- Polyglot book format (.bin files)
- Probe book at start of game
- Weighted random selection
- Fallback to search when out of book

**2. Endgame Tablebases (Optional)**

- Syzygy tablebase format
- Probe at root and during search
- WDL (Win/Draw/Loss) probing
- DTZ (Distance To Zero) probing for 50-move rule

#### Success Criteria

- ✅ Plays human openings in first 5-10 moves
- ✅ Solves all tablebase positions perfectly
- ✅ Performance: tablebase probe <1ms
- ✅ Falls back gracefully if book/TB missing

#### Files to Create/Modify

```
crates/engine/src/
├── book.rs                (NEW: polyglot book reader)
├── syzygy.rs              (NEW: tablebase probing, or use crate)
└── search.rs              (modify: probe book/TB)

assets/
└── book.bin               (polyglot book file, ~5-50MB)
```

#### Estimated Time

- **Optimistic:** 1 week (book only)
- **Realistic:** 2 weeks (book + tablebases)
- **Pessimistic:** 3 weeks

---

### M8: Tuning & Strength Testing (2-3 weeks)

**Goal:** Optimize evaluation parameters and measure strength

#### Features to Implement

**1. Parameter Tuning**

- Texel's tuning method (gradient descent on evaluation parameters)
- Dataset: 10,000-100,000 positions with known outcomes
- Optimize piece values, PST values, bonuses/penalties
- Automated tuning pipeline

**2. Test Suite**

- **Tactical test suite** (Bratko-Kopec, WAC, etc.)
- **Endgame test suite** (Lucena, Philidor, etc.)
- **Strategic test suite** (positional understanding)
- Automated regression testing

**3. Strength Estimation**

- Play against known-strength engines
- ELO rating estimation
- Performance tracking over time
- Weaknesses identification

**4. Bug Fixes & Optimization**

- Fix issues discovered in testing
- Profile and optimize hot paths
- Memory optimization
- Cache optimization

#### Success Criteria

- ✅ Solves 80%+ of tactical test suite
- ✅ Estimated ELO >2000 (beginner intermediate)
- ✅ No known game-losing bugs
- ✅ Deterministic results (same position → same move)

#### Files to Create/Modify

```
crates/tuning/              (NEW crate)
├── src/
│   ├── texel.rs           (Texel tuning implementation)
│   ├── dataset.rs         (position database)
│   └── optimize.rs        (gradient descent)

tests/
├── tactical_suite.rs      (NEW: test positions)
├── endgame_suite.rs       (NEW)
└── regression.rs          (NEW)

tools/
└── play_match.sh          (NEW: automated matches)
```

#### Estimated Time

- **Optimistic:** 2 weeks
- **Realistic:** 3 weeks
- **Pessimistic:** 4 weeks (if major bugs found)

---

### M9: WASM & Web Integration (2 weeks)

**Goal:** Compile engine to WebAssembly for browser use

#### Features to Implement

**1. WASM Compilation**

- Target: `wasm32-unknown-unknown`
- wasm-bindgen exports
- Binary size optimization (<2MB)
- Memory management

**2. Web Worker Integration**

- Engine runs in Web Worker (non-blocking)
- Message passing for commands
- Async result handling

**3. TypeScript Client**

- EngineClient interface
- Mode switching (fake/wasm/remote)
- Promise-based API
- Event streaming for search info

#### Success Criteria

- ✅ WASM binary <2MB
- ✅ Performance within 2× of native
- ✅ Works in Chrome, Firefox, Safari
- ✅ Non-blocking UI during search

#### Files to Create/Modify

```
crates/engine-bridge-wasm/  (NEW crate)
└── src/lib.rs             (wasm-bindgen exports)

apps/web/src/
├── workers/
│   └── engine.worker.ts   (NEW: Web Worker)
└── engine/
    └── engineClient.ts    (NEW: Client interface)
```

#### Estimated Time

- **Optimistic:** 1 week
- **Realistic:** 2 weeks
- **Pessimistic:** 3 weeks

---

### M10: Server Mode & Deployment (2 weeks)

**Goal:** Optional server-side engine for analysis

#### Features to Implement

**1. HTTP API**

- POST /analyze (start analysis)
- POST /stop (stop analysis)
- GET /health (health check)

**2. WebSocket Streaming**

- Real-time search info
- Session management
- Multiple concurrent analyses

**3. Deployment**

- Docker container
- Health checks
- Metrics/logging
- Cloud deployment (optional)

#### Success Criteria

- ✅ Handles 50+ concurrent sessions
- ✅ Latency <100ms per event
- ✅ Docker health checks pass
- ✅ Runs on Render/Fly.io/etc.

#### Estimated Time

- **Optimistic:** 1.5 weeks
- **Realistic:** 2 weeks
- **Pessimistic:** 3 weeks

---

## Track A Timeline (Engine Strength First)

### Sequential (Single Developer)

| Milestone | Duration      | Cumulative | What Gets Delivered                 |
| --------- | ------------- | ---------- | ----------------------------------- |
| M4        | 2-3 weeks     | 2-3 weeks  | Null move, LMR, move ordering       |
| M5        | 2 weeks       | 4-5 weeks  | Time mgmt, UCI protocol             |
| M6        | 2-3 weeks     | 6-8 weeks  | Advanced eval (pawns, king safety)  |
| M7        | 1-2 weeks     | 7-10 weeks | Opening book, tablebases            |
| M8        | 2-3 weeks     | 9-13 weeks | Tuning, testing, bug fixes          |
| M9        | 2 weeks       | 11-15 wks  | WASM compilation                    |
| M10       | 2 weeks       | 13-17 wks  | Server mode (optional)              |
| **TOTAL** | **13-17 wks** |            | **Complete chess engine + web app** |

### Key Milestones

- ✅ **After M5:** Can play against other engines via UCI
- ✅ **After M6:** Strong positional play
- ✅ **After M8:** Tuned, tested, production-grade engine
- ✅ **After M9:** Playable in browser
- ✅ **After M10:** Full-stack deployment

---

## Track B Details: Web Integration First

_(Abbreviated - full details available if you choose this track)_

### Quick Summary

- **M4:** Basic time management (1 week)
- **M5:** WASM + Web Worker (2 weeks)
- **M6:** React frontend MVP (2 weeks)
- **M7:** Server mode (optional, 2 weeks)
- **M8:** Advanced search (3 weeks)
- **M9:** Advanced eval (3 weeks)
- **M10:** UCI + testing (2 weeks)

**Total:** 13-15 weeks for complete system

**Key Milestone:** Working web app after 5 weeks (M4-M6)

---

## Recommendation

**I recommend Track A: Engine Strength First** for these reasons:

1. **Natural progression:** Build a complete, strong engine before web integration
2. **UCI compatibility:** Can test against other engines early (after M5)
3. **Less refactoring:** WASM integration is easier when engine is stable
4. **Focus:** One clear goal at a time
5. **Flexibility:** Can add web UI anytime after M5

**Choose Track B if:**

- You need a web demo quickly (investor/user testing)
- You have frontend developers waiting
- Visual progress is important for motivation
- You want to iterate on UX while engine improves

---

## Next Steps

### Decision Point

**Which track do you prefer?**

- A: Engine Strength First (M4-M10 as detailed above)
- B: Web Integration First (I'll detail this if chosen)
- C: Custom hybrid approach

### After Decision

1. Create M4 branch: `git checkout -b feature/m4-advanced-search`
2. Set up todo list for M4 sessions
3. Begin implementation

---

**Document Version:** 1.0
**Created:** 2025-10-26
**Based on:** M2-M7-PLAN.md (original), M3_COMPLETION.md (current state)
