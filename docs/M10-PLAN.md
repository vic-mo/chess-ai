# M10 Plan: WASM Debugging & Production Readiness

**Status**: 📋 Planning
**Priority**: High
**Target**: Fix critical WASM bug, alternative deployment options

---

## Objectives

1. **🔴 Critical**: Fix WASM mate detection bug
2. **🟡 High**: Implement SearchInfo streaming for WASM (or polling alternative)
3. **🟢 Medium**: UCI Server integration for production deployment
4. **🟢 Medium**: Opening book integration
5. **🟢 Low**: Endgame tablebase support

---

## Scope

### In Scope

- Root cause analysis of WASM mate detection failure
- WASM build configuration and optimization flags investigation
- Alternative WASM timing implementation using JS `performance.now()`
- SearchInfo streaming via polling or message passing
- Basic UCI server with remote engine support
- Simple opening book (polyglot format)
- Basic Syzygy tablebase probing

### Out of Scope

- Advanced book learning/tuning
- Cloud-based tablebase storage
- Multi-PV search
- UCI protocol extensions (like Chess960)

---

## Session Breakdown

### Session 1: WASM Mate Detection Root Cause Analysis 🔴

**Goal**: Identify why WASM build fails to find mate in 1

**Approach**:

1. Add extensive logging to search in WASM mode
2. Compare native vs WASM builds on same position
3. Verify evaluation scores at leaf nodes
4. Check if mate scores are being propagated correctly
5. Test different WASM optimization levels (`opt-level`, `lto`)
6. Profile WASM build to identify differences

**Test Positions**:

- Mate in 1: `r5k1/5ppp/8/8/8/8/5PPP/4Q1K1 w - - 0 1` (Qe8#)
- Mate in 2: `r1bqkb1r/pppp1Qpp/2n2n2/4p3/2B1P3/8/PPPP1PPP/RNB1K1NR b KQkq - 0 1`
- Mate in 3: Complex tactical mate

**Success Criteria**:

- Identify root cause (optimization flag, timing, evaluation bug)
- WASM finds mate in 1 correctly
- WASM mate detection matches native behavior

**Files to Modify**:

- `crates/engine/src/search/core.rs` - Add debug logging
- `crates/engine-bridge-wasm/Cargo.toml` - Test different opt levels
- `Cargo.toml` - Profile WASM settings

---

### Session 2: WASM Mate Detection Fix 🔴

**Goal**: Apply fix identified in Session 1

**Potential Fixes** (depending on root cause):

**If optimization issue**:

- Adjust `opt-level` in release profile for WASM
- Use `#[inline(never)]` on critical evaluation functions
- Disable specific optimizations causing miscompilation

**If timing issue**:

- Implement JS-based timing using `performance.now()`
- Pass time budget from JS to WASM
- Remove dummy `Instant` implementation

**If evaluation issue**:

- Fix mate score constants for WASM builds
- Ensure `MATE_SCORE - ply` calculation is correct
- Verify score propagation in negamax

**Success Criteria**:

- All mate detection tests pass in WASM
- Performance comparable to native (within 2x)
- No regressions in tactical tests

---

### Session 3: SearchInfo Streaming for WASM 🟡

**Goal**: Get real-time search progress updates in WASM mode

**Approach**: Polling API (avoids callback aliasing issues)

**Implementation**:

1. Add `get_search_state()` method to `WasmEngine`
2. Store current `SearchInfo` in `Arc<Mutex<Option<SearchInfo>>>`
3. Update state after each depth completes
4. JS polls every 100ms for updates

**API Design**:

```rust
#[wasm_bindgen]
impl WasmEngine {
    pub fn get_search_state(&self) -> Result<JsValue, JsValue> {
        // Returns Option<SearchInfo>
    }
}
```

```typescript
// Web worker polling
const pollInterval = setInterval(() => {
  const info = wasmEngine.get_search_state();
  if (info) postMessage({ type: 'searchInfo', payload: info });
}, 100);
```

**Alternative**: Message passing via shared memory or WebWorker postMessage

**Success Criteria**:

- Real-time depth updates in UI
- NPS and score display during search
- No performance degradation from polling

**Files to Modify**:

- `crates/engine-bridge-wasm/src/lib.rs` - Add polling API
- `apps/web/src/workers/engine.worker.ts` - Implement polling
- `apps/web/src/engine/engineClient.ts` - Handle search updates

---

### Session 4: UCI Server Integration 🟢

**Goal**: Provide production-ready remote engine alternative to WASM

**Approach**: WebSocket-based UCI protocol server

**Architecture**:

```
┌─────────────┐          ┌─────────────┐          ┌─────────────┐
│   Browser   │  ◄─WS──► │  UCI Server │  ◄─UCI─► │   Engine    │
│    (React)  │          │  (Node/Rust)│          │  (Native)   │
└─────────────┘          └─────────────┘          └─────────────┘
```

**Implementation**:

1. Create `apps/uci-server` package
2. Use `tokio` + `tokio-tungstenite` for WebSocket server
3. Bridge WebSocket messages to UCI protocol
4. Spawn engine process per connection
5. Handle position/analyze/stop commands

**Protocol**:

```json
// Client → Server
{
  "type": "analyze",
  "id": "uuid",
  "fen": "startpos",
  "limit": { "kind": "depth", "depth": 20 }
}

// Server → Client
{
  "type": "searchInfo",
  "id": "uuid",
  "payload": { "depth": 5, "score": {...}, "pv": [...] }
}
```

**Success Criteria**:

- Server handles multiple concurrent connections
- Full SearchInfo streaming works
- Graceful handling of disconnects
- Performance matches native CLI

**Files to Create**:

- `apps/uci-server/src/main.rs` - WebSocket server
- `apps/uci-server/src/bridge.rs` - UCI protocol bridge
- `apps/web/src/engine/remoteEngine.ts` - WebSocket client

---

### Session 5: Opening Book Integration 🟢

**Goal**: Use opening book for first 10-15 moves

**Approach**: Polyglot opening book format

**Implementation**:

1. Add `opening-book` crate for Polyglot reader
2. Probe book before search in `EngineImpl::analyze()`
3. Return book move immediately if found
4. Download free book (e.g., `performance.bin`, `gm2600.bin`)

**Book Probing**:

```rust
impl EngineImpl {
    fn analyze(&mut self, limit: SearchLimit, info_sink: F) -> BestMove {
        // Check opening book first
        if let Some(book_move) = self.book.probe(&self.current_board) {
            return BestMove {
                best: book_move.to_uci(),
                ponder: None,
            };
        }

        // Fall back to search
        self.searcher.search_with_limit_callback(...)
    }
}
```

**Success Criteria**:

- Book moves played in opening phase
- Smooth transition to engine search
- Configurable book usage (on/off, variety level)

**Files to Create**:

- `crates/opening-book/src/lib.rs` - Polyglot reader
- `crates/opening-book/src/zobrist.rs` - Hash computation

**Files to Modify**:

- `crates/engine/src/lib.rs` - Book probing before search

---

### Session 6: Endgame Tablebase Support 🟢

**Goal**: Perfect play in simple endgames

**Approach**: Syzygy tablebase probing

**Implementation**:

1. Add `syzygy` crate dependency (or implement basic probe)
2. Initialize tablebase in `EngineImpl::new()`
3. Probe before search if material count ≤ 6 pieces
4. Return DTZ (distance to zeroing move) or DTM (distance to mate)

**Tablebase Probing**:

```rust
impl EngineImpl {
    fn analyze(&mut self, limit: SearchLimit, info_sink: F) -> BestMove {
        // Probe tablebase if endgame
        if self.current_board.piece_count() <= 6 {
            if let Some(tb_result) = self.tb.probe(&self.current_board) {
                return BestMove {
                    best: tb_result.best_move.to_uci(),
                    ponder: None,
                };
            }
        }

        // Fall back to search
        self.searcher.search_with_limit_callback(...)
    }
}
```

**Success Criteria**:

- Perfect play in KQK, KRK, KPK endgames
- Tablebase hits reported in SearchInfo
- Configurable tablebase path

**Files to Modify**:

- `crates/engine/Cargo.toml` - Add syzygy dependency
- `crates/engine/src/lib.rs` - Tablebase probing

---

## Technical Risks

### High Risk

1. **WASM mate detection root cause unknown** - May require significant refactoring
2. **WASM optimization limitations** - May need to accept lower performance
3. **JS timing in WASM unreliable** - May need different time management approach

### Medium Risk

4. **SearchInfo polling overhead** - May impact WASM search speed
5. **UCI server scalability** - Need to handle load balancing for multiple users
6. **Opening book memory usage** - Large books may be too big for WASM

### Low Risk

7. **Tablebase file size** - 6-piece tablebases are ~1.2GB, may need CDN
8. **Book move randomization** - Need weighted random selection

---

## Success Metrics

### M10 Complete When:

- ✅ WASM finds mate in 1, 2, 3 correctly
- ✅ SearchInfo updates visible in UI (polling or streaming)
- ✅ UCI server deployed and accessible
- ✅ Opening book returns moves for e4, d4, Nf3, etc.
- ✅ Tablebase probes work for KQK position
- ✅ All 433 tests still passing (native)
- ✅ 19+ web integration tests passing (WASM/fake/remote)

### Performance Targets:

- **Native**: 10k+ NPS (debug), 100k+ NPS (release)
- **WASM**: 5k+ NPS (acceptable, given WASM overhead)
- **Remote**: <100ms latency for SearchInfo updates

---

## Testing Strategy

### Unit Tests

- Mate detection: 10+ positions (mate in 1-5)
- Opening book: Verify book moves from known openings
- Tablebase: KQK, KRK, KPK positions

### Integration Tests

- WASM worker: End-to-end analyze with polling
- UCI server: WebSocket connection, analyze, stop
- Book + engine: Transition from book to search

### Manual Tests

- Play full game in UI (opening → middlegame → endgame)
- Test WASM performance on mobile devices
- Verify UCI server under concurrent load

---

## Timeline Estimate

- **Session 1-2** (WASM debugging): 4-6 hours
- **Session 3** (SearchInfo polling): 2-3 hours
- **Session 4** (UCI server): 3-4 hours
- **Session 5** (Opening book): 2-3 hours
- **Session 6** (Tablebases): 2-3 hours

**Total**: 13-19 hours across 6 sessions

---

## Alternatives Considered

### For WASM Issues:

1. **Abandon WASM entirely** - Pro: Simpler, Con: Requires server for all users
2. **Use emscripten instead of wasm-bindgen** - Pro: Better stdlib support, Con: Larger binaries
3. **Rewrite timing in pure JS** - Pro: Accurate timing, Con: Complex FFI

### For SearchInfo Streaming:

1. **Callback with unsafe code** - Pro: Real-time, Con: May cause UB
2. **Shared memory** - Pro: Fast, Con: Complex synchronization
3. **WebWorker postMessage** - Pro: Safe, Con: Serialization overhead

**Decision**: Polling API for simplicity and safety.

### For Remote Engine:

1. **HTTP polling** - Pro: Simple, Con: High latency
2. **Server-Sent Events** - Pro: Unidirectional, Con: No bidirectional stop
3. **gRPC** - Pro: Efficient, Con: Browser support limited

**Decision**: WebSockets for bidirectional, low-latency communication.

---

## Dependencies

### New Crates:

- `tokio` + `tokio-tungstenite` - WebSocket server
- `opening-book` (internal) - Polyglot reader
- `syzygy` (external) or implement basic probe

### External Resources:

- Polyglot opening book (e.g., `performance.bin` ~5MB)
- Syzygy tablebases (3-4 piece: ~10MB, 5-6 piece: ~1.2GB)

---

## Future Work (Post-M10)

- **Multi-PV search** - Show multiple best lines
- **Chess960 support** - Fischer Random Chess
- **Pondering** - Think on opponent's time
- **NNUE evaluation** - Neural network eval (like Stockfish)
- **Distributed search** - Cloud-based parallel search
- **Book learning** - Update book from played games
- **Advanced time management** - Better time allocation

---

## Related Documents

- `M9-COMPLETE.md` - Current state and WASM issues
- `M9-REMAINING-WORK.md` - Original work items
- `ARCHITECTURE.md` - System design overview

---

**Created**: 2025-10-26
**Status**: Ready for Session 1
