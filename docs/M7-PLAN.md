# M7: Advanced Search Techniques - Implementation Plan

**Date:** 2025-10-26
**Status:** Planning
**Branch:** `feature/m7-advanced-search`
**Estimated Time:** 2-3 weeks

---

## Executive Summary

M7 will significantly enhance the chess engine's tactical and strategic strength by implementing advanced search techniques:

- **Selective extensions** (singular, check, recapture)
- **Advanced pruning** (futility, razoring, late move, multi-cut, probcut)
- **Enhanced move ordering** (countermove history, SEE, continuation history)
- **Internal iterative deepening/reduction** (IID/IIR)
- **Search optimizations** (PVS improvements, aspiration window tuning)

Current state: Basic alpha-beta with null move pruning, LMR, PVS, aspiration windows
Target state: Advanced search competitive with ~2400 ELO engines

Expected gain: +150 to +250 ELO over M6

---

## Current Search System (M6)

```rust
// crates/engine/src/search.rs
fn alpha_beta(board: &mut Board, depth: i32, alpha: i32, beta: i32) -> i32 {
    // Basic features:
    // ✅ Alpha-beta pruning
    // ✅ Null move pruning (R=2)
    // ✅ Late move reductions (LMR)
    // ✅ Principal variation search (PVS)
    // ✅ Aspiration windows
    // ✅ Quiescence search
    // ✅ Transposition table
    // ✅ Basic move ordering (TT move, MVV-LVA, killers, history)

    // Missing:
    // ❌ Selective extensions
    // ❌ Advanced pruning techniques
    // ❌ SEE-based move ordering
    // ❌ Countermove history
    // ❌ Multi-cut pruning
    // ❌ Probcut
}
```

---

## M7 Architecture

### Module Structure

```
crates/engine/src/
├── search.rs              # Core search (MAJOR REFACTOR)
├── search/
│   ├── mod.rs             # Search orchestration
│   ├── extensions.rs      # NEW: Selective extensions
│   ├── pruning.rs         # NEW: Advanced pruning
│   ├── ordering.rs        # REFACTOR: Enhanced move ordering
│   ├── history.rs         # REFACTOR: History heuristics + countermoves
│   ├── see.rs             # NEW: Static Exchange Evaluation
│   └── params.rs          # NEW: Search parameters and tuning
├── move_order.rs          # EXISTING - integrate with new ordering
└── tt.rs                  # EXISTING - minor enhancements
```

---

## Implementation Sessions

### Session 1-2: Static Exchange Evaluation (SEE) (1-2 days)

**Goal:** Implement SEE for accurate capture evaluation

#### What is SEE?

SEE evaluates the outcome of a capture sequence:

- Simulates all captures on a square
- Calculates material balance
- Used for move ordering and pruning decisions

#### Algorithm

```rust
fn see(board: &Board, mv: Move, threshold: i32) -> bool {
    // 1. Find all attackers/defenders of target square
    // 2. Simulate capture sequence (lowest value attacker first)
    // 3. Return net material gain
    // 4. Compare against threshold
}
```

#### Use Cases

1. **Move Ordering**: Order captures by SEE value
2. **Pruning**: Skip bad captures (SEE < 0)
3. **Quiescence**: Prune losing captures
4. **Extension Logic**: Extend good captures

#### Files

```
crates/engine/src/search/
└── see.rs                 # NEW (200 lines, 10 tests)
```

#### Tests

- Test simple capture (PxP)
- Test recapture sequence (PxP, PxP)
- Test underpromotion captures
- Test discovered attacks
- Test en passant captures
- Test multiple attackers/defenders
- Test SEE thresholds
- Test symmetry

#### Success Criteria

- ✅ Correctly evaluates all capture sequences
- ✅ Fast (<1µs per SEE call)
- ✅ Improves move ordering quality
- ✅ Handles edge cases (pins, discoveries)

---

### Session 3-4: Selective Extensions (1-2 days)

**Goal:** Extend search for critical positions

#### Extension Types

**1. Check Extensions**

```rust
// Extend when in check
if board.is_in_check() {
    depth += 1;
}
```

- Finds forced mates faster
- Prevents horizon effect
- Limit: Max 1 ply per path

**2. Singular Extensions**

```rust
// Extend if one move is much better than others
if is_singular_move(tt_move, beta, depth) {
    depth += 1;
}
```

- TT move is "singular" if all other moves fail low by margin
- Requires reduced depth verification search
- Very strong for tactical positions

**3. Recapture Extensions**

```rust
// Extend immediate recaptures
if is_recapture(board, prev_move, current_move) {
    depth += 1;
}
```

- Avoids horizon effect in exchanges
- Fractional extension (0.5 ply)

**4. Passed Pawn Extensions**

```rust
// Extend passed pawns to 6th/7th rank
if is_passed_pawn_push(board, mv) && rank >= RANK_6 {
    depth += 1;
}
```

- Important for endgames
- Fractional extension (0.5 ply)

#### Extension Limits

```rust
const MAX_EXTENSIONS_PER_PATH: i32 = 16;
let extension = calculate_extension(board, mv, depth);
extension = extension.min(MAX_EXTENSIONS_PER_PATH - path_extensions);
```

#### Files

```
crates/engine/src/search/
└── extensions.rs          # NEW (300 lines, 12 tests)
```

#### Tests

- Test check extension
- Test singular extension verification
- Test recapture extension
- Test passed pawn extension
- Test extension limits
- Test fractional extensions
- Test extension interactions
- Test mate-finding improvement

#### Success Criteria

- ✅ Finds mates 2-3 plies deeper
- ✅ No search explosions (extensions controlled)
- ✅ Tactical strength improved
- ✅ Extension limit prevents pathological cases

---

### Session 5-7: Advanced Pruning (2-3 days)

**Goal:** Prune more non-critical branches safely

#### Pruning Techniques

**1. Futility Pruning (FP)**

```rust
// Skip quiet moves in shallow nodes if eval + margin < alpha
if depth <= 3 && !in_check && !gives_check {
    let eval = evaluate(board);
    let margin = FUTILITY_MARGINS[depth];
    if eval + margin < alpha {
        // Skip quiet moves
        continue;
    }
}
```

Margins: [0, 100, 200, 300] by depth

**2. Reverse Futility Pruning (RFP)**

```rust
// Cut node early if eval - margin >= beta
if depth <= 5 && !in_check && !is_pv {
    let eval = evaluate(board);
    let margin = RFP_MARGINS[depth];
    if eval - margin >= beta {
        return eval; // Beta cutoff
    }
}
```

Margins: [0, 100, 200, 300, 400, 500] by depth

**3. Razoring**

```rust
// Drop into qsearch if eval + margin < alpha
if depth <= 3 && !in_check && !is_pv {
    let eval = evaluate(board);
    let margin = RAZOR_MARGINS[depth];
    if eval + margin < alpha {
        let q_score = qsearch(board, alpha, beta);
        if q_score < alpha {
            return q_score;
        }
    }
}
```

**4. Late Move Pruning (LMP)**

```rust
// Skip late quiet moves at shallow depths
if depth <= 3 && !in_check && move_count > LMP_THRESHOLDS[depth] {
    // Skip remaining quiet moves
    break;
}
```

Thresholds: [0, 3, 6, 12] by depth

**5. SEE Pruning**

```rust
// Skip bad captures
if is_capture && !see(board, mv, -SEE_QUIET_THRESHOLD) {
    continue; // Skip losing capture
}
```

**6. Multi-Cut Pruning**

```rust
// If M >= 3 moves cause beta cutoff at reduced depth, cut node
let mut cutoff_count = 0;
for mv in moves {
    score = -search(board, depth - 1 - R, -beta, -beta + 1);
    if score >= beta {
        cutoff_count += 1;
        if cutoff_count >= 3 {
            return beta; // Multi-cut
        }
    }
}
```

**7. Probcut**

```rust
// If shallow search proves score >> beta, cut node
let probcut_beta = beta + PROBCUT_MARGIN;
if depth >= 5 {
    for mv in good_captures {
        score = -search(board, depth - 4, -probcut_beta, -probcut_beta + 1);
        if score >= probcut_beta {
            return score; // Probcut
        }
    }
}
```

#### Safety Conditions

All pruning disabled if:

- In check
- PV node
- Mate score in TT
- Endgame (except SEE pruning)

#### Files

```
crates/engine/src/search/
└── pruning.rs             # NEW (400 lines, 15 tests)
```

#### Tests

- Test futility pruning correctness
- Test reverse futility pruning
- Test razoring
- Test late move pruning
- Test SEE pruning
- Test multi-cut detection
- Test probcut
- Test pruning safety (no tactical errors)
- Test pruning in endgames
- Test PV exemption from pruning

#### Success Criteria

- ✅ Search 1-2 plies deeper at same time
- ✅ No tactical regressions (test suites pass)
- ✅ Node count reduced by 30-50%
- ✅ Pruning disabled correctly in critical positions

---

### Session 8-10: Enhanced Move Ordering (2-3 days)

**Goal:** Improve move ordering for better alpha-beta cutoffs

#### Current Move Ordering

```
1. TT move (from hash table)
2. Winning captures (MVV-LVA)
3. Killer moves (2 per ply)
4. History heuristic
5. Losing captures
6. Quiet moves
```

#### Enhanced Move Ordering

**1. Add SEE to Capture Ordering**

```rust
// Good captures (SEE >= 0)
captures.sort_by(|a, b| see_value(b).cmp(&see_value(a)));

// Bad captures (SEE < 0)
// Ordered later
```

**2. Countermove Heuristic**

```rust
// Track last move's best refutation
struct CountermoveTable {
    table: [[Option<Move>; 64]; 64],  // [from][to] -> countermove
}

// When move fails high:
countermoves.store(prev_move, refutation_move);

// In move ordering:
if let Some(cm) = countermoves.get(prev_move) {
    // Order countermove high
}
```

**3. Continuation History**

```rust
// Track move pairs (prev_move, current_move)
struct ContinuationHistory {
    table: [[[[i32; 64]; 64]; 64]; 64],  // [from1][to1][from2][to2]
}

// Update on cutoff:
continuation_history.update(prev_move, current_move, depth);
```

**4. Capture History**

```rust
// Separate history for captures
struct CaptureHistory {
    table: [[[i32; 6]; 64]; 64],  // [from][to][captured_piece]
}
```

**5. Enhanced Killer Moves**

```rust
// 3 killer moves per ply instead of 2
const MAX_KILLERS: usize = 3;
```

#### New Move Ordering

```
1. TT move
2. Good captures (SEE >= 0, sorted by SEE + capture history)
3. Killer moves (3 per ply)
4. Countermove
5. Quiet moves (history + continuation history)
6. Bad captures (SEE < 0)
```

#### Files

```
crates/engine/src/search/
├── ordering.rs            # NEW (350 lines)
└── history.rs             # REFACTOR (400 lines, 12 tests)
```

#### Tests

- Test SEE-based capture ordering
- Test countermove tracking
- Test continuation history updates
- Test capture history
- Test move ordering quality (beta cutoff rate)
- Test ordering correctness
- Test memory usage

#### Success Criteria

- ✅ Beta cutoffs improve by 10-20%
- ✅ First move is best move 90%+ of time
- ✅ Ordering overhead <5% of search time
- ✅ Memory usage reasonable (<10MB)

---

### Session 11-12: Internal Iterative Deepening/Reduction (1-2 days)

**Goal:** Handle positions without TT move better

#### Internal Iterative Deepening (IID)

```rust
// If no TT move in PV node, do shallow search to get one
if depth >= 4 && tt_move.is_none() && is_pv {
    let iid_depth = depth - 2;
    search(board, iid_depth, alpha, beta);
    // Now TT has move for this position
    tt_move = tt.probe(board.zobrist_hash()).move;
}
```

#### Internal Iterative Reduction (IIR)

```rust
// If no TT move, reduce depth slightly
if tt_move.is_none() && depth >= 4 && !is_pv {
    depth -= 1;  // Reduce by 1 ply
}
```

#### When to Use

- **IID**: PV nodes, depth >= 4
- **IIR**: Non-PV nodes, depth >= 4
- Skip if in check

#### Files

```
crates/engine/src/search/
└── mod.rs                 # Modify search function (50 lines added)
```

#### Tests

- Test IID in PV nodes
- Test IIR in non-PV nodes
- Test TT interaction
- Test depth requirements
- Test performance improvement

#### Success Criteria

- ✅ Improves node efficiency when TT empty
- ✅ Small overhead (<5%)
- ✅ Better move ordering in novel positions

---

### Session 13-14: Integration & Tuning (2-3 days)

**Goal:** Integrate all techniques and tune parameters

#### Integration Checklist

1. **Search Flow**
   - Extensions before recursion
   - Pruning after move ordering
   - IID/IIR for missing TT moves
   - SEE integrated throughout

2. **Move Ordering Pipeline**
   - SEE for captures
   - Countermoves + continuation history
   - History tables unified

3. **Parameter Tuning**
   - Extension limits
   - Pruning margins
   - Reduction tables (LMR)
   - History weights

#### Search Pseudocode

```rust
fn alpha_beta(board: &mut Board, depth: i32, alpha: i32, beta: i32,
              pv: bool, in_null: bool) -> i32 {
    // 1. Check time, nodes, stop flag

    // 2. Draw detection

    // 3. TT probe

    // 4. Mate distance pruning

    // 5. Evaluation + stand pat (at frontier)

    // 6. Reverse futility pruning (RFP)
    if depth <= 5 && !in_check && !pv {
        let eval = evaluate(board);
        if eval - rfp_margin(depth) >= beta {
            return eval;
        }
    }

    // 7. Razoring
    if depth <= 3 && !in_check && !pv {
        let eval = evaluate(board);
        if eval + razor_margin(depth) < alpha {
            let q = qsearch(board, alpha, beta);
            if q < alpha { return q; }
        }
    }

    // 8. Null move pruning
    if depth >= 2 && !in_check && !in_null && has_non_pawns(board) {
        let R = 2 + (depth > 6) as i32;
        let null_score = -alpha_beta(board, depth - 1 - R, -beta, -beta + 1, false, true);
        if null_score >= beta { return beta; }
    }

    // 9. Probcut
    if depth >= 5 && !pv {
        let probcut_beta = beta + 200;
        for mv in good_captures {
            board.make_move(mv);
            let score = -alpha_beta(board, depth - 4, -probcut_beta, -probcut_beta + 1, false, false);
            board.unmake_move();
            if score >= probcut_beta { return score; }
        }
    }

    // 10. IID/IIR
    if tt_move.is_none() && depth >= 4 {
        if pv {
            // IID
            alpha_beta(board, depth - 2, alpha, beta, pv, false);
            tt_move = tt.probe(...).move;
        } else {
            // IIR
            depth -= 1;
        }
    }

    // 11. Move loop
    let moves = generate_moves(board);
    order_moves(&mut moves, tt_move, killers, history, countermoves);

    let mut best_score = -INFINITY;
    let mut move_count = 0;
    let futility_pruning = depth <= 3 && !in_check && !pv;
    let lmp_threshold = lmp_table[depth];

    for mv in moves {
        // Skip pruned moves
        if futility_pruning && !is_tactical(mv) {
            let eval = evaluate(board);
            if eval + futility_margin(depth) < alpha {
                continue;
            }
        }

        if move_count >= lmp_threshold && !is_tactical(mv) {
            break; // LMP
        }

        if is_capture(mv) && !see(board, mv, -100) {
            continue; // SEE pruning
        }

        move_count += 1;
        board.make_move(mv);

        // Extensions
        let extension = calculate_extension(board, mv, depth);

        // Reductions (LMR)
        let reduction = calculate_reduction(depth, move_count, pv, mv);

        // Recursive search
        let score = if move_count == 1 {
            -alpha_beta(board, depth - 1 + extension, -beta, -alpha, pv, false)
        } else {
            // LMR + PVS
            let r = reduction;
            let mut score = -alpha_beta(board, depth - 1 - r, -alpha - 1, -alpha, false, false);
            if score > alpha && r > 0 {
                score = -alpha_beta(board, depth - 1, -alpha - 1, -alpha, false, false);
            }
            if score > alpha && pv {
                score = -alpha_beta(board, depth - 1, -beta, -alpha, pv, false);
            }
            score
        };

        board.unmake_move();

        if score > best_score {
            best_score = score;
            if score > alpha {
                alpha = score;
                // Update history, killers, countermoves
                if score >= beta {
                    // Beta cutoff
                    return beta;
                }
            }
        }
    }

    // 12. Store in TT
    tt.store(board.zobrist_hash(), depth, best_score, alpha_orig, beta, best_move);

    best_score
}
```

#### Tuning Parameters

```rust
// Search parameters
pub struct SearchParams {
    // Extensions
    pub check_extension: i32,
    pub singular_extension_margin: i32,
    pub recapture_extension: i32,

    // Pruning margins
    pub futility_margins: [i32; 4],
    pub rfp_margins: [i32; 6],
    pub razor_margins: [i32; 4],
    pub lmp_thresholds: [usize; 4],

    // Reduction parameters
    pub lmr_base: i32,
    pub lmr_divisor: i32,

    // Probcut
    pub probcut_margin: i32,
    pub probcut_depth: i32,

    // Other
    pub see_quiet_threshold: i32,
    pub max_extensions: i32,
}
```

#### Test Positions

- Tactical puzzles (WAC, WCSAC)
- Mate problems (depth requirements)
- Positional games (pruning safety)
- Endgames (king activity)

#### Files Modified

```
crates/engine/src/
├── search.rs              # Major refactor (integrate all techniques)
└── search/params.rs       # NEW (parameter management)
```

#### Success Criteria

- ✅ All techniques integrated cleanly
- ✅ No search instabilities
- ✅ Parameters tuned for strength
- ✅ Code maintainable and documented

---

### Session 15-16: Testing & Validation (2-3 days)

**Goal:** Validate strength improvement and stability

#### Test Suites

**1. Tactical Test Suite**

- WAC (Win at Chess): 300 positions
- WCSAC (Win at Chess Simplified): 20 critical positions
- Target: 280/300 solved (M6: ~240/300)

**2. Strategic Test Suite**

- Bratko-Kopec: 24 positions
- Target: 18/24 solved (M6: ~14/24)

**3. Endgame Test Suite**

- Basic mates (KQK, KRK, KBBK)
- Pawn endgames (opposition, key squares)
- Target: 95%+ solved (M6: ~90%)

**4. Self-Play**

- M7 vs M6: 500 games (various time controls)
- Expected: 65-75% win rate
- ELO gain: +150 to +250

**5. Perft Regression**

- Ensure move generation still correct
- All perft tests must pass

#### Performance Benchmarks

```
Metric                    M6          M7 Target
---------------------------------------------------
Nodes/second              1.0M        0.9-1.1M (similar)
Tactical (WAC)            240/300     280/300
Strategic (Bratko-Kopec)  14/24       18/24
Average depth (5s)        10          11-12
Self-play vs M6           50%         70%
```

#### Stability Testing

- 1000 random positions (no crashes)
- Time management (stops within deadline)
- TT stability (no corruption)
- Search determinism (same position → same move)

#### Regression Testing

- All previous tests (321 tests) must pass
- No evaluation regressions
- No move generation bugs

#### Files

```
tests/
├── tactical_suite.rs      # NEW (300 WAC positions)
├── strategic_suite.rs     # NEW (24 Bratko-Kopec positions)
└── search_tests.rs        # MODIFY (add M7 tests)
```

#### Success Criteria

- ✅ M7 beats M6 by 65-75% in self-play
- ✅ Tactical suite improved by 15-20%
- ✅ Search 1-2 plies deeper at same time
- ✅ No regressions or crashes
- ✅ All 350+ tests passing

---

## Summary: Session Breakdown

| Session   | Focus                      | Duration       | Files            | Tests   | LOC      |
| --------- | -------------------------- | -------------- | ---------------- | ------- | -------- |
| 1-2       | Static Exchange Eval (SEE) | 1-2 days       | 1 new            | 10      | 200      |
| 3-4       | Selective Extensions       | 1-2 days       | 1 new            | 12      | 300      |
| 5-7       | Advanced Pruning           | 2-3 days       | 1 new            | 15      | 400      |
| 8-10      | Enhanced Move Ordering     | 2-3 days       | 2 new            | 12      | 750      |
| 11-12     | IID/IIR                    | 1-2 days       | 1 mod            | 5       | 50       |
| 13-14     | Integration & Tuning       | 2-3 days       | 2 mod            | 20      | 300      |
| 15-16     | Testing & Validation       | 2-3 days       | 3 test           | 350     | 500      |
| **Total** | **M7 Complete**            | **12-18 days** | **5 new, 4 mod** | **424** | **2500** |

---

## Expected Outcomes

### Quantitative

- **ELO Gain:** +150 to +250 ELO over M6
- **Search Depth:** +1 to +2 plies at same time
- **Node Count:** Reduced by 30-40% (better pruning)
- **Tactical Strength:** 280/300 WAC (was 240/300)
- **Tests:** 424 new tests (321 existing → 745 total)
- **Code:** ~2,500 lines of new search code

### Qualitative

- ✅ Finds mates deeper and faster
- ✅ Handles tactical positions better
- ✅ More efficient search (better pruning)
- ✅ Better move ordering (faster cutoffs)
- ✅ More stable search scores
- ✅ Competitive with intermediate engines

---

## Risks & Mitigations

### Risk 1: Search Instability

**Impact:** High (causes blunders)
**Mitigation:**

- Extensive testing with test suites
- Conservative pruning margins initially
- Gradual tuning with validation
- Safety conditions (disable in PV, check, etc.)

### Risk 2: Implementation Complexity

**Impact:** Medium (bugs, maintainability)
**Mitigation:**

- Modular design (separate files per technique)
- Comprehensive unit tests
- Clear documentation
- Incremental integration (one technique at a time)

### Risk 3: Performance Regression

**Impact:** Medium (slower search)
**Mitigation:**

- Profile before/after each session
- Optimize hot paths
- Keep overhead low (<5% per technique)
- Measure nodes/second continuously

### Risk 4: Over-Pruning

**Impact:** High (tactical errors)
**Mitigation:**

- Conservative margins initially
- Disable in critical positions (PV, check)
- Extensive tactical test suites
- Validate with known positions

---

## Post-M7: What's Next?

### M8: Opening Book & Endgame Tablebases (2-3 weeks)

- Polyglot opening book integration
- Syzygy tablebase probing
- Book learning/statistics

### M9: Parallel Search (2-3 weeks)

- Lazy SMP (multi-threaded search)
- Shared transposition table
- Split point management

### M10: Evaluation Tuning (2-3 weeks)

- Texel tuning for all parameters
- SPSA tuning for search parameters
- Regression testing framework

---

## Success Criteria: M7 Complete

- ✅ All 7 advanced techniques implemented:
  - ✅ SEE (Static Exchange Evaluation)
  - ✅ Selective extensions (check, singular, recapture)
  - ✅ Advanced pruning (futility, razoring, LMP, multi-cut, probcut)
  - ✅ Enhanced move ordering (SEE, countermoves, continuation history)
  - ✅ IID/IIR
  - ✅ Full integration
  - ✅ Parameter tuning

- ✅ Performance targets met:
  - ✅ M7 beats M6 by 65-75% (500 game match)
  - ✅ Search 1-2 plies deeper
  - ✅ 280/300 WAC solved
  - ✅ 18/24 Bratko-Kopec solved

- ✅ Quality targets met:
  - ✅ 745 total tests passing
  - ✅ No crashes or instabilities
  - ✅ Code documented and maintainable
  - ✅ No tactical regressions

---

## Estimated Strength After M7

- **Current (M6):** ~2000-2200 ELO
- **Target (M7):** ~2200-2400 ELO
- **Comparable to:** Intermediate engines, strong club players

Next milestone (M8) with opening book and tablebases should reach 2400-2500 ELO.

---

**Document Version:** 1.0
**Created:** 2025-10-26
**Status:** Planning Complete - Ready for Implementation
