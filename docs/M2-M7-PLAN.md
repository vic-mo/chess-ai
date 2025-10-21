# Chess AI: M2-M7 Implementation Plan

## Executive Summary

**Current Status:** ✅ M0 (Tooling) and M1 (Protocol) complete

**Remaining Work:** 6 milestones (M2-M7) spanning ~12-16 weeks

**Goal:** Production-ready chess engine with React frontend and flexible deployment (WASM/Server)

---

## Milestone Overview

```
Timeline: 12-16 weeks total

Week 1-3:  M2 (Engine Core)          [CRITICAL PATH]
Week 4-6:  M3 (Search v1)             [CRITICAL PATH]
Week 7-8:  M4 (Evaluation)            [CRITICAL PATH]
Week 9-10: M5 (WASM) + M6 (Frontend) [PARALLEL]
Week 11-12: M7 (Server Mode)          [INTEGRATION]
Week 13-16: Testing, Polish, Deploy   [FINAL]
```

---

## Milestone Dependency Graph

```
M0 (Tooling) ✅
    ↓
M1 (Protocol) ✅
    ↓
    ├──────────────────┐
    ↓                  ↓
M2 (Engine Core)   M6 (Frontend MVP)*
    ↓                  ↑
M3 (Search v1)         │
    ↓                  │
M4 (Evaluation)        │
    ↓                  │
M5 (WASM Bridge) ──────┤
    ↓                  │
M7 (Server Mode) ──────┘
    ↓
Production Ready

* M6 can start in parallel with M2-M4 using mock engine
```

---

## M2: Engine Core (Weeks 1-3)

### What Gets Built

- **Board representation** (bitboards)
- **Move generation** (pseudo-legal → legal filtering)
- **FEN parsing/serialization**
- **Perft testing harness**
- **Zobrist hashing**

### Success Criteria

- ✅ Perft depth 1-6 matches canonical values
- ✅ ≥3M nodes/s single-threaded
- ✅ No panics under fuzz testing
- ✅ All tests passing in CI

### Key Files

```
crates/engine/src/
├── board.rs         (bitboards, state)
├── move.rs          (Move struct)
├── movegen.rs       (generation logic)
├── fen.rs           (FEN I/O)
├── movelist.rs      (stack-allocated list)
└── tests/perft.rs   (validation)
```

### Risks & Mitigations

| Risk                      | Mitigation                                 |
| ------------------------- | ------------------------------------------ |
| Incorrect move generation | Perft validation against known positions   |
| Poor performance          | Profile with criterion, optimize hot paths |
| Self-check bugs           | Dedicated legality tests per piece type    |

### Time Estimate

- **Optimistic:** 2 weeks (experienced Rust + chess dev)
- **Realistic:** 3 weeks
- **Pessimistic:** 4 weeks (if major bugs found)

---

## M3: Search v1 (Weeks 4-6)

### What Gets Built

- **Iterative deepening**
- **Alpha-beta pruning** (negamax)
- **Quiescence search**
- **Transposition table**
- **Move ordering** (MVV-LVA, killers)
- **Time management**

### Success Criteria

- ✅ Produces valid moves with stable scores
- ✅ ≥500k nodes/s single-threaded
- ✅ Stop request works within 50ms
- ✅ No illegal moves or PV corruption

### Key Files

```
crates/engine/src/
├── search.rs        (alpha-beta, ID)
├── eval.rs          (simple material eval)
├── tt.rs            (transposition table)
└── time.rs          (time control)
```

### Risks & Mitigations

| Risk                          | Mitigation                                         |
| ----------------------------- | -------------------------------------------------- |
| Infinite quiescence recursion | Depth limit + stand-pat cutoff                     |
| Non-deterministic results     | Single-threaded execution + deterministic ordering |
| Memory leaks in TT            | Fixed capacity with replacement scheme             |

### Time Estimate

- **Optimistic:** 2.5 weeks
- **Realistic:** 3 weeks
- **Pessimistic:** 4 weeks (if search bugs are hard to debug)

### Integration Point

After M3: **First playable engine!** Can integrate with server for basic testing.

---

## M4: Evaluation v1 (Weeks 7-8)

### What Gets Built

- **Material balance** (piece values)
- **Piece-square tables** (PST)
- **Mobility evaluation**
- **Pawn structure** (doubled, isolated, passed)
- **King safety** (pawn shield, open files)
- **Game phase interpolation** (midgame/endgame)

### Success Criteria

- ✅ Mirror-board symmetry validated
- ✅ Eval time <20µs per call
- ✅ Improves search stability and strength
- ✅ Deterministic across runs

### Key Files

```
crates/engine/src/eval/
├── mod.rs           (main evaluate fn)
├── material.rs      (piece values)
├── pst.rs           (piece-square tables)
├── pawns.rs         (pawn structure + hash)
└── king.rs          (king safety)
```

### Risks & Mitigations

| Risk                 | Mitigation                              |
| -------------------- | --------------------------------------- |
| Feature overcounting | Coefficient registry + regression tests |
| Asymmetric eval      | Mirror-board tests in CI                |
| Slow pawn structure  | Pawn hash table with Zobrist keys       |

### Time Estimate

- **Optimistic:** 1.5 weeks
- **Realistic:** 2 weeks
- **Pessimistic:** 3 weeks (if tuning reveals issues)

### Integration Point

After M4: **Strong playable engine** ready for user testing.

---

## M5: WASM Bridge (Weeks 9-10)

### What Gets Built

- **WASM compilation** (wasm32-unknown-unknown)
- **wasm-bindgen exports**
- **Web Worker integration**
- **Mode selector** (fake/wasm/remote)

### Success Criteria

- ✅ WASM binary <2.5MB
- ✅ Performance within 1.5× of native
- ✅ Works in Chrome, Firefox, Safari
- ✅ Protocol parity with server

### Key Files

```
crates/engine-bridge-wasm/src/lib.rs
apps/web/src/workers/engine.worker.ts
apps/web/src/engine/engineClient.ts
```

### Risks & Mitigations

| Risk                   | Mitigation                 |
| ---------------------- | -------------------------- |
| Large binary size      | LTO, wasm-opt, strip debug |
| UI thread blocking     | All compute in Worker      |
| Serialization overhead | serde_wasm_bindgen         |

### Time Estimate

- **Optimistic:** 1 week
- **Realistic:** 1.5 weeks
- **Pessimistic:** 2 weeks

### Parallel Opportunity

✅ **M5 can run in parallel with M6** (frontend team uses mock while WASM builds)

---

## M6: Frontend React MVP (Weeks 9-10)

### What Gets Built

- **Interactive chessboard** component
- **FEN input** and validation
- **Search log viewer** (depth, score, PV)
- **Analysis controls** (start/stop/reset)
- **State management** (Zustand)
- **Mode switching** (fake/wasm/remote)

### Success Criteria

- ✅ Functional UI for analysis
- ✅ Works with all 3 engine modes
- ✅ No UI lag during search
- ✅ E2E tests passing

### Key Files

```
apps/web/src/
├── App.tsx                    (root)
├── components/
│   ├── Board.tsx              (chessboard)
│   ├── Controls.tsx           (inputs)
│   └── Log.tsx                (search info)
└── engine/engineClient.ts     (mode abstraction)
```

### Risks & Mitigations

| Risk                | Mitigation                   |
| ------------------- | ---------------------------- |
| State sync issues   | Central store + cancel logic |
| Excessive rerenders | Batched updates + throttling |
| Network disconnects | Auto-reconnect + retry UI    |

### Time Estimate

- **Optimistic:** 1 week
- **Realistic:** 1.5 weeks
- **Pessimistic:** 2 weeks

### Parallel Opportunity

✅ **M6 can start as early as M2** using the existing fake engine!

---

## M7: Server Mode (Weeks 11-12)

### What Gets Built

- **HTTP API** (/analyze, /stop, /health)
- **WebSocket streaming** (/streams/{id})
- **Session management** (UUID, broadcast)
- **Docker container**
- **Metrics/logging**

### Success Criteria

- ✅ Handles 100+ concurrent sessions
- ✅ Latency <100ms per event
- ✅ No memory leaks under load
- ✅ Docker health checks pass
- ✅ Frontend integration complete

### Key Files

```
services/engine-server/src/
├── main.rs          (entry point)
├── routes.rs        (HTTP handlers)
├── session.rs       (state tracking)
└── stream.rs        (WebSocket)
```

### Risks & Mitigations

| Risk            | Mitigation                    |
| --------------- | ----------------------------- |
| Memory leaks    | Timeout cleanup + weak refs   |
| Engine blocking | Async channels + yield points |
| WS instability  | Heartbeat ping/pong           |

### Time Estimate

- **Optimistic:** 1.5 weeks
- **Realistic:** 2 weeks
- **Pessimistic:** 3 weeks (if concurrency bugs arise)

### Integration Point

After M7: **Full stack working!** Production-ready system.

---

## Recommended Implementation Strategy

### Option A: Sequential (Safest)

```
M2 → M3 → M4 → M5 → M6 → M7
Pros: Simple, reduces integration risk
Cons: Long timeline (~16 weeks), no early UI
```

### Option B: Parallel Tracks (Recommended ⭐)

```
Track 1 (Backend):  M2 → M3 → M4 → M5 → M7
Track 2 (Frontend): M6 (starts at M2, uses mock)
                      └─ integrates at M5/M7

Week 1-3:  M2 (Backend)
Week 1-2:  M6 (Frontend starts with mock)
Week 4-6:  M3 (Backend)
Week 3-4:  M6 continues (UI polish)
Week 7-8:  M4 (Backend)
Week 9:    M5 (Backend) + M6 finishes (Frontend)
Week 9:    First WASM integration
Week 10-11: M7 (Backend)
Week 12:   Final integration testing

Total: 12 weeks (25% faster!)
```

### Option C: Aggressive Parallel (Riskiest)

```
All of M2-M6 in parallel with 3 developers
Pros: Fastest (10 weeks)
Cons: High coordination overhead, merge conflicts
```

**Recommendation:** **Option B** - Parallel tracks with staggered start

---

## Resource Allocation

### Single Developer

- **Timeline:** 14-16 weeks
- **Strategy:** Sequential with M6 mock work during breaks
- **Critical:** Stay on M2→M3→M4 critical path first

### Two Developers

- **Timeline:** 10-12 weeks
- **Strategy:** Option B (parallel tracks)
- **Split:**
  - Dev 1: M2, M3, M4, M5, M7 (backend)
  - Dev 2: M6, integration testing (frontend)

### Three Developers

- **Timeline:** 8-10 weeks
- **Strategy:** Enhanced Option B
- **Split:**
  - Dev 1: M2, M3 (engine core)
  - Dev 2: M4, M5 (eval + WASM)
  - Dev 3: M6, M7 (frontend + server)

---

## Week-by-Week Plan (Option B, 2 Developers)

| Week | Backend Dev              | Frontend Dev             | Milestone |
| ---- | ------------------------ | ------------------------ | --------- |
| 1    | M2: Board representation | M6: Chessboard component | -         |
| 2    | M2: Move generation      | M6: Controls + mock      | -         |
| 3    | M2: FEN + perft          | M6: State management     | ✅ M2     |
| 4    | M3: Alpha-beta           | M6: UI polish            | -         |
| 5    | M3: Quiescence + TT      | M6: Testing              | -         |
| 6    | M3: Time management      | M6: Documentation        | ✅ M3     |
| 7    | M4: Material + PST       | Integration testing      | -         |
| 8    | M4: Pawns + king safety  | Integration testing      | ✅ M4     |
| 9    | M5: WASM bridge          | M6: WASM integration     | ✅ M5, M6 |
| 10   | M7: HTTP routes          | M7: Frontend fixes       | -         |
| 11   | M7: WebSocket + session  | M7: E2E testing          | -         |
| 12   | M7: Docker + metrics     | M7: Load testing         | ✅ M7     |

---

## Testing Strategy

### Per-Milestone Testing

- **M2:** Perft validation (depth 1-6)
- **M3:** Search correctness (known positions)
- **M4:** Mirror-board symmetry
- **M5:** WASM/native parity
- **M6:** E2E UI flows
- **M7:** Load testing (100 sessions)

### Integration Testing Windows

1. **Week 6:** M3 complete → test with mock server
2. **Week 9:** M5 complete → test WASM in browser
3. **Week 12:** M7 complete → full stack E2E

### Continuous Testing

- **Every commit:** Unit tests + clippy + prettier
- **Every PR:** Full CI pipeline
- **Weekly:** Performance regression checks

---

## Risk Management

### High-Risk Areas

1. **M2 Correctness** (Critical Path)
   - **Risk:** Move generation bugs
   - **Mitigation:** Perft early, test incrementally
   - **Fallback:** Use existing chess library temporarily

2. **M3 Performance** (Critical Path)
   - **Risk:** <500k nps target missed
   - **Mitigation:** Profile early, optimize hot paths
   - **Fallback:** Reduce target to 300k nps

3. **M7 Concurrency** (Integration Risk)
   - **Risk:** Deadlocks or memory leaks
   - **Mitigation:** Load testing, memory profiling
   - **Fallback:** Limit to 50 concurrent sessions

### Medium-Risk Areas

4. **M5 WASM Size**
   - **Risk:** Binary >2.5MB
   - **Mitigation:** wasm-opt aggressive mode
   - **Fallback:** Accept 3MB, async loading

5. **M6 UX Complexity**
   - **Risk:** Scope creep on features
   - **Mitigation:** Stick to MVP, defer extras
   - **Fallback:** Ship with basic UI first

---

## Success Metrics

### Technical Metrics

- **M2:** Perft(6) in <2s
- **M3:** Depth 10 search in <10s on startpos
- **M4:** Eval within 50cp of reference engine
- **M5:** WASM loads in <500ms
- **M6:** Time to interactive <2s
- **M7:** P95 latency <200ms under load

### Business Metrics

- **Functionality:** Can analyze any legal position
- **Reliability:** 99.9% uptime (server mode)
- **Performance:** Competitive with online tools
- **Usability:** <5 min to first analysis (new users)

---

## Next Steps

### Immediate (This Week)

1. ✅ Review and approve this plan
2. ✅ Decide on implementation strategy (A/B/C)
3. ✅ Assign developers to tracks
4. ✅ Create M2 branch and first PR

### Week 1 Kickoff

1. Set up project board (GitHub Projects / Jira)
2. Create M2 subtasks in todo list
3. Schedule weekly sync meetings
4. Begin M2: Board representation

### Ongoing

- Weekly milestone review
- Bi-weekly integration testing
- Monthly stakeholder demo

---

## Appendix: Command Cheatsheet

### Starting M2

```bash
git checkout -b feature/m2-engine-core
cd crates/engine
# Edit src/board.rs, src/movegen.rs
cargo test
cargo bench
```

### Starting M6 (Parallel)

```bash
git checkout -b feature/m6-frontend-mvp
cd apps/web
# Edit src/components/Board.tsx
pnpm dev
pnpm test
```

### Integration Testing

```bash
# Terminal 1: Start server
cargo run -p engine-server

# Terminal 2: Start frontend
cd apps/web
VITE_ENGINE_MODE=remote pnpm dev

# Browser: http://localhost:5173
```

### Full CI Check (Before Merge)

```bash
make ci
```

---

## Questions & Decisions

### Open Questions

- [ ] Deploy server to cloud (AWS/GCP/Render)?
- [ ] Add opening book or endgame tablebases?
- [ ] Support PGN import/export?
- [ ] Multiplayer/analysis sharing?

### Deferred Features (Post-M7)

- Neural network evaluation (M8?)
- Multi-threaded search (M9?)
- UCI protocol support (M10?)
- Mobile app (React Native) (M11?)

---

**Document Version:** 1.0
**Last Updated:** 2025-10-21
**Next Review:** After M2 completion
