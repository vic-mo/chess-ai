# M2 Implementation Plan - Execution Strategy

**Goal:** Implement M2 Engine Core systematically over 3 weeks

**Approach:** Incremental development with continuous testing

---

## Implementation Strategy

### Philosophy

- **Build bottom-up:** Simple types → Complex structures
- **Test continuously:** Don't wait for Week 3 to test
- **Commit frequently:** Small, atomic commits per component
- **Validate early:** Run tests after each module completion

### Risk Mitigation

- Write tests BEFORE complex logic (TDD-lite)
- Use existing chess positions to validate
- Run perft as soon as move generation works
- Profile performance from Day 1

---

## Phase 1: Foundation (Days 1-3, ~12 hours)

### Objectives

✅ All basic types working
✅ Board can represent positions
✅ Moves can be created and inspected

### Implementation Order

**Session 1: Basic Enums (2-3 hours)**

```
1.1 Create module structure (files only)
1.2 Implement Square enum with tests
1.3 Implement Piece enum
1.4 Implement Color enum
1.5 Update lib.rs with module declarations
1.6 Run: cargo test
```

**Session 2: Bitboards (2-3 hours)**

```
2.1 Implement Bitboard struct
2.2 Add bit operations (set, clear, toggle)
2.3 Add bit scanning (lsb, pop_lsb, msb)
2.4 Implement BitOr, BitAnd, BitXor, Not traits
2.5 Write comprehensive bitboard tests
2.6 Run: cargo test
```

**Session 3: Board Structure (3 hours)**

```
3.1 Define Board struct with all fields
3.2 Implement Board::new() and Board::startpos()
3.3 Implement piece_at(), set_piece(), remove_piece()
3.4 Implement all_occupancy()
3.5 Write board accessor tests
3.6 Run: cargo test
3.7 MILESTONE: Can represent chess positions
```

**Session 4: Move & Castling (3 hours)**

```
4.1 Define Move struct with bit packing
4.2 Implement MoveFlag enum
4.3 Define CastlingRights bitflags
4.4 Implement move accessor methods
4.5 Write move packing/unpacking tests
4.6 Run: cargo test
```

**Validation Checkpoint 1:**

```bash
cargo test
cargo clippy
# Should have: ~15 tests passing
# Should have: Square, Piece, Color, Bitboard, Board basics, Move
```

---

## Phase 2: Move Generation Foundation (Days 4-6, ~14 hours)

### Objectives

✅ Attack maps for all pieces
✅ Pseudo-legal move generation working
✅ Make/unmake moves functional

### Implementation Order

**Session 5: Attack Tables (4 hours)**

```
5.1 Create attacks module
5.2 Implement precomputed pawn attacks
5.3 Implement precomputed knight attacks
5.4 Implement precomputed king attacks
5.5 Implement sliding piece attack generation (rays)
5.6 Add lazy_static for table initialization
5.7 Write attack generation tests
5.8 Run: cargo test
```

**Session 6: MoveList & Basic Movegen (3 hours)**

```
6.1 Implement MoveList (stack-allocated)
6.2 Create movegen module
6.3 Implement generate_pawn_moves (quiet only)
6.4 Implement generate_knight_moves
6.5 Implement generate_king_moves
6.6 Write movegen tests (count moves from startpos)
6.7 Run: cargo test
```

**Session 7: Sliding Piece Movegen (3 hours)**

```
7.1 Implement generate_bishop_moves
7.2 Implement generate_rook_moves
7.3 Implement generate_queen_moves
7.4 Write tests for each piece type
7.5 Run: cargo test
```

**Session 8: Make/Unmake Moves (4 hours)**

```
8.1 Define UndoInfo struct
8.2 Implement Board::make_move() - basic version
8.3 Handle piece movement (from → to)
8.4 Handle captures
8.5 Implement Board::unmake_move()
8.6 Write make/unmake tests
8.7 Run: cargo test
8.8 MILESTONE: Can make and unmake simple moves
```

**Validation Checkpoint 2:**

```bash
cargo test
# Should have: ~30 tests passing
# Can generate moves for all pieces
# Can make/unmake basic moves
```

---

## Phase 3: Special Moves & FEN (Days 7-8, ~8 hours)

### Objectives

✅ All special moves handled (castling, en passant, promotion)
✅ FEN parsing and serialization working
✅ Board state fully reversible

### Implementation Order

**Session 9: Special Moves in Movegen (3 hours)**

```
9.1 Add double pawn push detection
9.2 Add en passant move generation
9.3 Add pawn promotion moves
9.4 Add castling move generation
9.5 Implement generate_all_moves() wrapper
9.6 Write tests for special moves
9.7 Run: cargo test
```

**Session 10: Special Moves in Make/Unmake (2 hours)**

```
10.1 Handle en passant in make_move
10.2 Handle castling (move rook) in make_move
10.3 Handle promotions in make_move
10.4 Update unmake_move for all special cases
10.5 Write comprehensive make/unmake tests
10.6 Run: cargo test
```

**Session 11: FEN Parser (3 hours)**

```
11.1 Create fen module
11.2 Implement FromStr for Board (FEN parsing)
11.3 Handle piece placement parsing
11.4 Handle castling rights parsing
11.5 Handle en passant square parsing
11.6 Write FEN parsing tests
11.7 Test with standard positions
11.8 Run: cargo test
11.9 MILESTONE: Can parse any FEN string
```

**Session 12: FEN Serializer (1 hour)**

```
12.1 Implement Board::to_fen()
12.2 Handle piece placement serialization
12.3 Handle castling/en passant serialization
12.4 Write roundtrip tests (parse → serialize → parse)
12.5 Run: cargo test
```

**Validation Checkpoint 3:**

```bash
cargo test
# Should have: ~45 tests passing
# Can parse FEN
# Can serialize FEN
# All special moves work
```

---

## Phase 4: Legality & Zobrist (Days 9-10, ~7 hours)

### Objectives

✅ Only legal moves generated
✅ Position hashing working
✅ All state properly tracked

### Implementation Order

**Session 13: Legality Checking (4 hours)**

```
13.1 Implement is_square_attacked()
13.2 Implement is_in_check()
13.3 Implement is_legal() (make → check → unmake)
13.4 Implement generate_legal_moves()
13.5 Update castling to check for attacked squares
13.6 Write legality tests
13.7 Run: cargo test
13.8 MILESTONE: All illegal moves filtered
```

**Session 14: Zobrist Hashing (3 hours)**

```
14.1 Create zobrist module
14.2 Generate random Zobrist keys (lazy_static)
14.3 Implement Board::calculate_hash()
14.4 Add hash field to Board
14.5 Initialize hash in from_fen and startpos
14.6 Update hash incrementally in make_move
14.7 Restore hash in unmake_move
14.8 Write hash tests (incremental == full recalc)
14.9 Run: cargo test
14.10 Add rand and once_cell dependencies
```

**Validation Checkpoint 4:**

```bash
cargo test
# Should have: ~55 tests passing
# Only legal moves generated
# Hash correctly tracks position
```

---

## Phase 5: Perft Validation (Days 11-13, ~12 hours)

### Objectives

✅ Perft implementation complete
✅ All canonical positions pass
✅ Performance ≥3M nodes/s

### Implementation Order

**Session 15: Perft Implementation (2 hours)**

```
15.1 Create perft module
15.2 Implement perft(board, depth) function
15.3 Implement perft_divide(board, depth) function
15.4 Write basic perft test (depth 1-3)
15.5 Run: cargo test
```

**Session 16: Canonical Perft Tests (2 hours)**

```
16.1 Create tests/perft.rs
16.2 Add startpos perft tests (depth 1-5)
16.3 Add Kiwipete perft tests
16.4 Add position 3 perft tests
16.5 Add position 4 perft tests
16.6 Add position 5 perft tests
16.7 Run: cargo test perft
16.8 EXPECT: Some tests will fail - that's OK!
```

**Session 17: Debug Failing Perft (4-6 hours)**

```
17.1 Run perft_divide to find divergence
17.2 Identify which move causes mismatch
17.3 Debug move generation for that piece/case
17.4 Fix bug
17.5 Re-run perft tests
17.6 Repeat until all pass
17.7 MILESTONE: All perft tests passing!

Common bugs to check:
- En passant legality (pinned pawns)
- Castling through check
- Promotion on capture
- Double pawn push conditions
```

**Session 18: Performance Tuning (4 hours)**

```
18.1 Create benches/perft_bench.rs
18.2 Add Criterion benchmarks
18.3 Run: cargo bench
18.4 Profile with flamegraph
18.5 Optimize hot paths:
     - Inline critical functions
     - Optimize bit operations
     - Reduce allocations
18.6 Re-run benchmarks
18.7 Verify ≥3M nodes/s on perft depth 5
```

**Validation Checkpoint 5:**

```bash
cargo test perft
cargo bench
# Should have: All perft tests passing
# Should have: ≥3M nodes/s performance
```

---

## Phase 6: Polish & Documentation (Days 14-15, ~8 hours)

### Objectives

✅ All edge cases covered
✅ Code documented
✅ CI passing
✅ M2 complete!

### Implementation Order

**Session 19: Edge Cases & Fuzz Testing (3 hours)**

```
19.1 Add en passant pin test
19.2 Add castling rights edge cases
19.3 Add promotion capture tests
19.4 Add fuzz testing (random FEN)
19.5 Fix any discovered bugs
19.6 Run: cargo test
```

**Session 20: Documentation (2 hours)**

```
20.1 Add module-level documentation
20.2 Add doc comments to all public items
20.3 Add usage examples in lib.rs
20.4 Generate docs: cargo doc --open
20.5 Review and improve
```

**Session 21: Code Quality (1.5 hours)**

```
21.1 Run: cargo clippy --all-targets -- -D warnings
21.2 Fix all clippy warnings
21.3 Run: cargo fmt
21.4 Review all TODO comments
21.5 Clean up debug code
```

**Session 22: Final Validation (1.5 hours)**

```
22.1 Run full test suite
22.2 Run all perft tests (including depth 6)
22.3 Run benchmarks
22.4 Run CI locally: make ci
22.5 Create benchmark report
22.6 Verify all DoD criteria
```

**Final Checkpoint:**

```bash
make ci
cargo test -- --ignored  # Run slow tests
cargo bench

# Verify DoD:
✓ Perft d1-d6 match canonical values
✓ No panics under fuzz testing
✓ cargo test --workspace passes
✓ Nodes/s ≥ 3M single-threaded
✓ All modules documented
✓ Code is lint-clean
```

---

## Implementation Schedule (Gantt-style)

```
Day 1:  [Session 1-2: Basic types, Bitboards]
Day 2:  [Session 3-4: Board, Move structs]
        └─ Checkpoint 1 ✓
Day 3:  [Session 5: Attack tables]
Day 4:  [Session 6-7: Basic movegen, Sliding pieces]
Day 5:  [Session 8: Make/Unmake]
        └─ Checkpoint 2 ✓
Day 6:  [Session 9-10: Special moves]
Day 7:  [Session 11-12: FEN I/O]
        └─ Checkpoint 3 ✓
Day 8:  [Session 13: Legality]
Day 9:  [Session 14: Zobrist]
        └─ Checkpoint 4 ✓
Day 10: [Session 15-16: Perft implementation]
Day 11: [Session 17: Debug perft] ⚠️ CRITICAL
Day 12: [Session 18: Performance tuning]
        └─ Checkpoint 5 ✓
Day 13: [Session 19: Edge cases]
Day 14: [Session 20-21: Docs & quality]
Day 15: [Session 22: Final validation]
        └─ M2 COMPLETE! 🎉
```

---

## Session Workflow (Repeat for each session)

### Before Starting

```bash
# 1. Review session tasks
# 2. Ensure previous session tests pass
cargo test
# 3. Create focused branch (optional)
git checkout -b m2/session-N
```

### During Session

```bash
# 1. Write failing test first (if applicable)
# 2. Implement feature
# 3. Make test pass
# 4. Run all tests
cargo test
# 5. Commit
git add .
git commit -m "feat(engine): [Session N] Feature description"
```

### After Session

```bash
# 1. Verify no regressions
cargo test
cargo clippy
# 2. Document progress
# 3. Push work
git push origin m2/session-N
# 4. Take a break!
```

---

## Dependency Management

### Add Dependencies As Needed

**After Session 5 (Attack tables):**

```toml
# crates/engine/Cargo.toml
[dependencies]
once_cell = "1.19"
```

**After Session 14 (Zobrist):**

```toml
[dependencies]
rand = "0.8"
once_cell = "1.19"
```

**After Session 18 (Benchmarks):**

```toml
[dev-dependencies]
criterion = "0.5"

[[bench]]
name = "perft_bench"
harness = false
```

---

## Testing Strategy Per Phase

### Phase 1-2: Unit Tests

```bash
# Test individual components
cargo test square
cargo test bitboard
cargo test board
cargo test move
```

### Phase 3-4: Integration Tests

```bash
# Test component interactions
cargo test movegen
cargo test make_unmake
cargo test fen
```

### Phase 5: Validation Tests

```bash
# Test correctness
cargo test perft
cargo test --release perft_startpos_depth_6 -- --ignored
```

### Phase 6: Final Tests

```bash
# Test everything
cargo test --workspace --all-features
make ci
```

---

## Success Metrics Per Phase

| Phase | Key Metric    | Target            |
| ----- | ------------- | ----------------- |
| 1     | Tests passing | ≥15               |
| 2     | Tests passing | ≥30               |
| 3     | Tests passing | ≥45               |
| 4     | Tests passing | ≥55               |
| 5     | Perft depth 6 | 119,060,324 nodes |
| 6     | Performance   | ≥3M nodes/s       |

---

## Risk Management & Contingencies

### If Behind Schedule

**Priority Cuts (in order):**

1. Skip depth 6 perft initially (do depth 5 only)
2. Reduce documentation detail
3. Skip fuzz testing
4. Defer some edge case tests

**Never Skip:**

- Basic types
- Move generation
- Make/unmake moves
- Legality checking
- Perft depth 1-5

### If Perft Fails (Day 11)

**Debugging Strategy:**

```bash
# 1. Start with depth 1 (should pass easily)
cargo test perft_startpos_depth_1

# 2. Move to depth 2 (common bugs appear)
cargo test perft_startpos_depth_2

# 3. Use divide to find divergence
cargo run --bin perft_divide -- "startpos" 3

# 4. Compare with reference engine
# 5. Fix one bug at a time
# 6. Re-test all depths
```

**Common Bugs:**

- En passant: Check if capturing pawn is pinned
- Castling: Verify intermediate squares not attacked
- Promotions: Need 4 piece types × (quiet + capture)
- Double pawn push: Only from rank 2/7

### If Performance Too Slow

**Optimization Checklist:**

```
□ Inline hot functions (#[inline(always)])
□ Use const lookup tables
□ Optimize bitboard operations (use intrinsics)
□ Reduce allocations (stack instead of heap)
□ Profile with flamegraph
□ Check compiler optimizations (--release)
```

---

## Git Strategy

### Branch Structure

```
main
  └─ feature/m2-engine-core
       ├─ m2/session-1   (basic types)
       ├─ m2/session-2   (bitboards)
       ├─ m2/session-3   (board)
       ...
       └─ m2/final       (merge point)
```

### Commit Message Format

```
feat(engine): [Session N] Brief description

- Bullet point 1
- Bullet point 2

Tests: X passing
```

### Merge Strategy

```bash
# Merge session branches into main feature branch
git checkout feature/m2-engine-core
git merge m2/session-1
git merge m2/session-2
# ... etc

# Final PR to main
git checkout main
git merge feature/m2-engine-core
```

---

## Ready to Start?

### Immediate Next Steps (Right Now!)

**Step 1: Create branch and structure** (5 min)

```bash
git checkout -b feature/m2-engine-core
cd crates/engine
mkdir -p tests benches
```

**Step 2: Create all module files** (2 min)

```bash
touch src/square.rs
touch src/piece.rs
touch src/color.rs
touch src/bitboard.rs
touch src/board.rs
touch src/move.rs
touch src/movelist.rs
touch src/movegen.rs
touch src/attacks.rs
touch src/fen.rs
touch src/castling.rs
touch src/zobrist.rs
touch src/perft.rs
touch tests/perft.rs
touch benches/perft_bench.rs
```

**Step 3: Begin Session 1, Task 1.2** (Now!)

```bash
# Open Square module
code src/square.rs

# Start implementing...
```

---

## Questions Before Starting?

- [ ] Understand the bottom-up approach?
- [ ] Know which session to start with?
- [ ] Have environment set up (Rust, tools)?
- [ ] Ready to commit frequently?
- [ ] Understand validation checkpoints?

---

**Let's build this! 🚀**

When ready, say "Start M2 implementation" and I'll begin with Session 1, Task 1.1.
