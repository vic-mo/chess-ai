# M6: Advanced Evaluation - Implementation Plan

**Date:** 2025-10-26
**Status:** Planning
**Branch:** `feature/m6-advanced-eval`
**Estimated Time:** 2-3 weeks

---

## Executive Summary

M6 will enhance the chess engine's positional understanding by implementing:

- **Pawn structure evaluation** (doubled, isolated, passed, chains)
- **King safety assessment** (pawn shield, attacking pieces)
- **Piece activity bonuses** (rooks, bishops, knights)
- **Game phase interpolation** (smooth middlegame → endgame)

Current state: Basic evaluation (material + PST + mobility)
Target state: Advanced positional evaluation competitive with ~2000 ELO engines

---

## Current Evaluation System (M5)

```rust
// crates/engine/src/eval/mod.rs
pub fn evaluate(board: &Board) -> i32 {
    let material = evaluate_material(board);
    let pst = evaluate_pst(board);
    let mobility = evaluate_mobility(board);

    material + pst + mobility
}
```

**Strengths:**

- ✅ Material counting correct
- ✅ Piece-square tables (middlegame/endgame)
- ✅ Basic mobility

**Weaknesses:**

- ❌ No pawn structure awareness
- ❌ No king safety evaluation
- ❌ No piece coordination bonuses
- ❌ Phase interpolation is basic

---

## M6 Architecture

### Module Structure

```
crates/engine/src/eval/
├── mod.rs                 # Main evaluator orchestration (MODIFY)
├── material.rs            # Material counting (EXISTING - no changes)
├── pst.rs                 # Piece-square tables (EXISTING - no changes)
├── positional.rs          # Mobility (EXISTING - minor enhancements)
├── pawns.rs               # NEW: Pawn structure evaluation + hash table
├── king.rs                # NEW: King safety evaluation
├── pieces.rs              # NEW: Piece activity (rooks, bishops, knights)
└── phase.rs               # NEW: Game phase calculation
```

### Evaluation Flow

```rust
pub fn evaluate(board: &Board) -> i32 {
    // 1. Calculate game phase (0 = opening, 256 = endgame)
    let phase = calculate_phase(board);

    // 2. Evaluate components
    let material = evaluate_material(board);
    let pst = evaluate_pst(board, phase);  // Interpolated
    let pawns = evaluate_pawns(board);     // With hash table
    let king = evaluate_king_safety(board, phase);
    let pieces = evaluate_pieces(board);
    let mobility = evaluate_mobility(board);

    // 3. Combine with phase-based weights
    interpolate(material, pst, pawns, king, pieces, mobility, phase)
}
```

---

## Implementation Sessions

### Session 1-2: Game Phase Calculation (1-2 days)

**Goal:** Implement smooth middlegame → endgame phase detection

#### Features

**Phase Calculation:**

- Based on non-pawn material
- Opening: All pieces on board (~24 pieces = phase 0)
- Middlegame: Some trades made (phase 64-192)
- Endgame: Few pieces left (phase 224-256)

**Formula:**

```rust
const TOTAL_PHASE: i32 = 24; // Opening material
phase = 256 - (current_material * 256 / TOTAL_PHASE);
phase = phase.clamp(0, 256);
```

**Material Weights:**

- Pawn: 0 (doesn't affect phase)
- Knight: 1
- Bishop: 1
- Rook: 2
- Queen: 4

#### Files

```
crates/engine/src/eval/
└── phase.rs               # NEW (100 lines, 5 tests)
```

#### Tests

- Test phase 0 (all pieces)
- Test phase 256 (bare kings)
- Test phase interpolation (middlegame)
- Test symmetry (white/black same phase)
- Test phase boundaries

#### Success Criteria

- ✅ Phase 0 for starting position
- ✅ Phase 256 for bare kings
- ✅ Smooth interpolation between phases
- ✅ Symmetric for both colors

---

### Session 3-5: Pawn Structure Evaluation (2-3 days)

**Goal:** Evaluate pawn weaknesses and strengths

#### Features

**1. Doubled Pawns**

- Penalty: -10 to -20 cp per doubled pawn
- Worse on central files
- Less important in endgame

**2. Isolated Pawns**

- Penalty: -15 to -25 cp
- Worse in endgame (can't support passed pawns)
- Central isolation less bad

**3. Backward Pawns**

- Penalty: -10 to -15 cp
- Can't advance without losing control
- Check if square in front is weak

**4. Passed Pawns**

- Bonus: +20 to +150 cp (by rank)
- 7th rank pawn: +150 cp
- 6th rank pawn: +80 cp
- 5th rank pawn: +40 cp
- Unstoppable passed pawns: huge bonus
- Blockaded passed pawns: reduced bonus

**5. Pawn Chains**

- Bonus: +5 cp per protected pawn
- Diagonal pawn protection

**6. Pawn Islands**

- Penalty: -10 cp per island beyond 1
- Islands = groups of connected pawns

#### Pawn Hash Table

```rust
struct PawnEntry {
    key: u64,          // Zobrist key (pawns only)
    mg_score: i16,     // Middlegame score
    eg_score: i16,     // Endgame score
}

struct PawnHashTable {
    entries: Vec<PawnEntry>,
    size: usize,
}
```

**Cache Strategy:**

- Hash only pawn positions (very stable)
- 16K entries (~512KB memory)
- ~80% cache hit rate expected

#### Files

```
crates/engine/src/eval/
└── pawns.rs               # NEW (400 lines, 15 tests)
```

#### Tests

- Test doubled pawns detection
- Test isolated pawns detection
- Test backward pawns detection
- Test passed pawns detection (all ranks)
- Test pawn chains
- Test pawn islands counting
- Test pawn hash table (store/probe)
- Test pawn evaluation symmetry
- Test specific positions (doubled, isolated, passed)

#### Success Criteria

- ✅ Correctly detects all pawn structure features
- ✅ Pawn hash table gives 20%+ speedup
- ✅ Cache hit rate > 70%
- ✅ Evaluation is symmetric (mirror positions)
- ✅ Passed pawn values increase by rank

---

### Session 6-8: King Safety Evaluation (2-3 days)

**Goal:** Assess king vulnerability to attack

#### Features

**1. Pawn Shield**

- Bonus: +15 to +30 cp per pawn in front of king
- Check pawns on files f/g/h for kingside castle
- Check pawns on files a/b/c for queenside castle
- Penalty for missing/advanced shield pawns

**2. King Zone**

- Define 3x3 area around king (or 3x4 for edge)
- Count attacking pieces in zone

**3. Attack Weights**

- Queen attack: 4 points
- Rook attack: 3 points
- Bishop attack: 2 points
- Knight attack: 2 points
- Pawn attack: 1 point

**4. Attack Score → Penalty**

```rust
let attack_score = count_attackers_weighted(board, king_square);
let penalty = match attack_score {
    0..=2 => 0,
    3..=5 => -20,
    6..=9 => -50,
    10..=15 => -100,
    16+ => -200,
};
```

**5. Open Files Near King**

- Penalty: -20 cp for open file adjacent to king
- Penalty: -40 cp for open file king is on
- Half-open files: -10 cp penalty

**6. King Tropism**

- Enemy pieces near king get bonus
- Distance-based: closer = higher bonus
- Only in middlegame (phase < 200)

#### Files

```
crates/engine/src/eval/
└── king.rs                # NEW (350 lines, 12 tests)
```

#### Tests

- Test pawn shield detection (kingside castle)
- Test pawn shield detection (queenside castle)
- Test king zone definition
- Test attacker counting (all piece types)
- Test attack weight calculation
- Test open file detection near king
- Test king safety in middlegame vs endgame
- Test king tropism calculation
- Test symmetry (white/black kings)
- Test specific positions (exposed king, safe king)

#### Success Criteria

- ✅ Correctly identifies pawn shield quality
- ✅ Counts attackers accurately
- ✅ King safety negligible in endgame (phase > 200)
- ✅ Exposed king gets large penalty
- ✅ Castled king gets bonus
- ✅ Evaluation is symmetric

---

### Session 9-11: Piece Activity Evaluation (2-3 days)

**Goal:** Reward well-placed and coordinated pieces

#### Features

**1. Rook Activity**

**Rook on Open File:**

- Bonus: +30 cp
- Open = no pawns (either color)

**Rook on Semi-Open File:**

- Bonus: +15 cp
- Semi-open = no friendly pawns, has enemy pawns

**Rook on 7th Rank:**

- Bonus: +25 cp (if enemy king on 8th or enemy pawns on 7th)
- Bonus: +40 cp for two rooks on 7th rank

**2. Bishop Activity**

**Bishop Pair:**

- Bonus: +50 cp (in middlegame)
- Bonus: +60 cp (in endgame)
- Only if both bishops alive

**Bad Bishop:**

- Penalty: -15 cp
- Bad if >50% of pawns on bishop's color
- Less important in endgame

**Trapped Bishop:**

- Penalty: -150 cp
- Check for trapped bishops (a7, h7, a2, h2)

**3. Knight Activity**

**Knight Outpost:**

- Bonus: +20 to +40 cp
- Outpost = protected by pawn, can't be attacked by enemy pawns
- Must be on 4th rank or higher (for white)
- Central outposts worth more

**Trapped Knight:**

- Penalty: -100 cp
- Knight on edge with no legal moves

**4. Piece Mobility (Enhancement)**

- Already implemented in M3
- Add bonus for centralized pieces
- Add penalty for pieces on back rank (in middlegame)

#### Files

```
crates/engine/src/eval/
└── pieces.rs              # NEW (400 lines, 15 tests)
```

#### Tests

- Test rook on open file detection
- Test rook on semi-open file detection
- Test rook on 7th rank bonus
- Test bishop pair detection
- Test bad bishop detection
- Test trapped bishop detection
- Test knight outpost detection
- Test trapped knight detection
- Test piece activity by game phase
- Test symmetry

#### Success Criteria

- ✅ Correctly identifies open/semi-open files
- ✅ Rooks on 7th rank get bonus
- ✅ Bishop pair bonus applied correctly
- ✅ Bad bishop detected accurately
- ✅ Knight outposts identified
- ✅ Activity bonuses phase-dependent

---

### Session 12-13: Integration & Tuning (2-3 days)

**Goal:** Integrate all evaluation components with proper weighting

#### Integration

**Update eval/mod.rs:**

```rust
pub struct Evaluator {
    pawn_hash: PawnHashTable,
}

impl Evaluator {
    pub fn evaluate(&mut self, board: &Board) -> i32 {
        // 1. Game phase
        let phase = phase::calculate_phase(board);

        // 2. Material (unchanged)
        let material = material::evaluate_material(board);

        // 3. Piece-square tables (already has MG/EG)
        let pst = pst::evaluate_pst(board);

        // 4. Pawn structure (cached)
        let pawns = self.evaluate_pawns_cached(board);

        // 5. King safety (phase-dependent)
        let king = king::evaluate_king_safety(board, phase);

        // 6. Piece activity
        let pieces = pieces::evaluate_pieces(board, phase);

        // 7. Mobility (existing)
        let mobility = positional::evaluate_mobility(board);

        // 8. Combine scores
        let mg_score = material + pst.mg + pawns.mg + king.mg + pieces.mg + mobility;
        let eg_score = material + pst.eg + pawns.eg + king.eg + pieces.eg + mobility;

        // 9. Interpolate based on phase
        interpolate(mg_score, eg_score, phase)
    }
}
```

#### Tuning

**Initial Parameter Values:**

```rust
// Pawn structure
const DOUBLED_PAWN_PENALTY: i32 = -15;
const ISOLATED_PAWN_PENALTY: i32 = -20;
const BACKWARD_PAWN_PENALTY: i32 = -12;
const PASSED_PAWN_BONUS: [i32; 8] = [0, 10, 20, 40, 80, 150, 250, 0];
const PAWN_CHAIN_BONUS: i32 = 5;
const PAWN_ISLAND_PENALTY: i32 = -10;

// King safety
const PAWN_SHIELD_BONUS: i32 = 20;
const OPEN_FILE_KING_PENALTY: i32 = -40;
const SEMI_OPEN_FILE_KING_PENALTY: i32 = -20;

// Piece activity
const ROOK_OPEN_FILE_BONUS: i32 = 30;
const ROOK_SEMI_OPEN_FILE_BONUS: i32 = 15;
const ROOK_SEVENTH_RANK_BONUS: i32 = 25;
const BISHOP_PAIR_BONUS: i32 = 50;
const BAD_BISHOP_PENALTY: i32 = -15;
const KNIGHT_OUTPOST_BONUS: i32 = 30;
```

**Test Positions:**

1. Doubled pawns position
2. Isolated queen pawn
3. Passed pawn endgame
4. Exposed king position
5. Rooks on 7th rank
6. Bishop pair advantage
7. Knight outpost position

#### Files Modified

```
crates/engine/src/eval/
├── mod.rs                 # Major refactor (~300 lines)
└── positional.rs          # Minor enhancements
```

#### Tests

- Test full evaluation integration
- Test evaluation symmetry (mirror positions)
- Test evaluation speed (<30µs per call)
- Test pawn hash effectiveness
- Test specific tactical positions
- Test endgame evaluation
- Test middlegame evaluation
- Regression tests (all previous positions still work)

#### Success Criteria

- ✅ All 252+ previous tests still pass
- ✅ 50+ new evaluation tests pass
- ✅ Evaluation time <30µs
- ✅ Pawn hash hit rate >70%
- ✅ Symmetry tests pass
- ✅ No evaluation blind spots

---

### Session 14: Testing & Validation (1-2 days)

**Goal:** Validate strength improvement and fix bugs

#### Test Suites

**1. Positional Test Suite**

- 50 positions with known best moves
- Verify engine finds correct plan
- Compare M5 vs M6 performance

**2. Tactical Test Suite**

- Ensure tactics still work
- No regression in tactical strength
- WAC (Win at Chess) positions

**3. Endgame Test Suite**

- Basic endgames (KPK, KQKR, etc.)
- Passed pawn races
- Opposition and key squares

**4. Self-Play Testing**

- M6 vs M5: 100 games
- Expected win rate: 65-75%
- Measure ELO gain (+100 to +200)

#### Performance Benchmarks

```
Metric                  M5        M6 Target
-----------------------------------------
Eval time (avg)        15µs      <25µs
Pawn hash hit rate     N/A       >70%
Nodes/second           1.2M      >1.0M (slight decrease OK)
Depth in 5s            9-10      8-10 (eval more expensive)
```

#### Bug Fixes

- Fix any evaluation asymmetries
- Fix any crashes in edge cases
- Optimize hot paths if needed

#### Files

```
tests/
├── positional_suite.rs    # NEW (100 lines, 50 test positions)
├── endgame_suite.rs       # NEW (80 lines, 20 test positions)
└── regression_suite.rs    # MODIFY (add M6 tests)
```

#### Success Criteria

- ✅ M6 beats M5 in 65%+ of games
- ✅ No tactical regression
- ✅ Positional play visibly improved
- ✅ All test suites pass
- ✅ Performance acceptable

---

## Summary: Session Breakdown

| Session   | Focus                 | Duration       | Files            | Tests   | LOC      |
| --------- | --------------------- | -------------- | ---------------- | ------- | -------- |
| 1-2       | Game Phase            | 1-2 days       | 1 new            | 5       | 100      |
| 3-5       | Pawn Structure + Hash | 2-3 days       | 1 new            | 15      | 400      |
| 6-8       | King Safety           | 2-3 days       | 1 new            | 12      | 350      |
| 9-11      | Piece Activity        | 2-3 days       | 1 new            | 15      | 400      |
| 12-13     | Integration & Tuning  | 2-3 days       | 2 mod            | 20      | 300      |
| 14        | Testing & Validation  | 1-2 days       | 3 test           | 70      | 180      |
| **Total** | **M6 Complete**       | **10-16 days** | **4 new, 3 mod** | **137** | **1730** |

---

## Expected Outcomes

### Quantitative

- **ELO Gain:** +100 to +200 ELO over M5
- **Tests:** 137 new tests (252 existing → 389 total)
- **Code:** ~1,730 lines of new evaluation code
- **Performance:** Evaluation time 20-25µs (vs 15µs in M5)
- **Pawn Hash:** 70-80% hit rate, ~20% speedup on pawn eval

### Qualitative

- ✅ Engine understands pawn structure
- ✅ Engine avoids weakening king position
- ✅ Engine values piece coordination
- ✅ Engine plays stronger positional chess
- ✅ Smooth evaluation across game phases

---

## Risks & Mitigations

### Risk 1: Evaluation Too Slow

**Impact:** High (affects search depth)
**Mitigation:**

- Profile hot paths
- Optimize pawn hash table
- Use incremental evaluation if needed

### Risk 2: Parameter Tuning Required

**Impact:** Medium (affects strength)
**Mitigation:**

- Use reasonable initial values from strong engines
- Plan for Texel tuning in M8
- Validate with test positions

### Risk 3: Evaluation Bugs

**Impact:** High (wrong moves)
**Mitigation:**

- Extensive testing with known positions
- Symmetry tests
- Visual debugging of evaluation breakdown

### Risk 4: Complexity Creep

**Impact:** Medium (maintainability)
**Mitigation:**

- Keep modules small and focused
- Document all evaluation terms
- Use clear, readable code over clever tricks

---

## Post-M6: What's Next?

### M7: Opening Book & Endgame Tablebases (1-2 weeks)

- Polyglot opening book integration
- Syzygy tablebase probing

### M8: Tuning & Strength Testing (2-3 weeks)

- Texel tuning for all evaluation parameters
- Tactical test suites (WAC, Bratko-Kopec)
- ELO estimation vs known engines

### M9: WASM & Web Integration (2 weeks)

- Compile to WebAssembly
- Web Worker integration

### M10: Server Mode & Deployment (2 weeks)

- HTTP API
- Docker deployment

---

## Success Criteria: M6 Complete

- ✅ All 4 new modules implemented (phase, pawns, king, pieces)
- ✅ Pawn hash table working with >70% hit rate
- ✅ 137 new tests passing (389 total)
- ✅ Evaluation time <30µs
- ✅ M6 beats M5 in 65%+ of games
- ✅ No tactical regression
- ✅ Positional play improved
- ✅ All code documented and clean
- ✅ Ready for M7 (opening book & tablebases)

---

**Document Version:** 1.0
**Created:** 2025-10-26
**Status:** Planning Complete - Ready for Implementation
