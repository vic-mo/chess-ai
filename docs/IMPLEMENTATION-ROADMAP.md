# Play vs Engine - Implementation Roadmap

**Based on**: PLAY-VS-ENGINE-PLAN.md (Revised 2025-10-26)
**Approach**: Engine-based (no chess.js)
**Total Time**: ~12 hours

---

## Phase 0: Engine API Extensions (30 min) ⭐

**Goal**: Add game-specific methods to engine core

### 0.1: Core Engine Methods (15 min)

**File**: `crates/engine/src/lib.rs`

```bash
# Add these 4 methods to EngineImpl:
1. is_move_legal(fen, uci_move) -> bool
2. make_move(fen, uci_move) -> Result<String>
3. legal_moves(fen) -> Vec<String>
4. is_game_over(fen) -> (bool, Option<String>)
```

**Tasks**:

- [ ] Add `is_move_legal()` method (uses existing `generate_legal_moves()`)
- [ ] Add `make_move()` method (validates + applies move, returns new FEN)
- [ ] Add `legal_moves()` method (returns UCI strings)
- [ ] Add `is_game_over()` method (check for checkmate/stalemate)
- [ ] Test with `cargo test -p engine`

---

### 0.2: WASM Bindings (10 min)

**File**: `crates/engine-bridge-wasm/src/lib.rs`

```bash
# Expose 4 methods to JavaScript:
1. isMoveLegal(fen, uciMove) -> bool
2. makeMove(fen, uciMove) -> Result<String, JsValue>
3. legalMoves(fen) -> Vec<JsValue>
4. isGameOver(fen) -> (bool, Option<String>)
```

**Tasks**:

- [ ] Add `#[wasm_bindgen(js_name = isMoveLegal)]` wrapper
- [ ] Add `#[wasm_bindgen(js_name = makeMove)]` wrapper
- [ ] Add `#[wasm_bindgen(js_name = legalMoves)]` wrapper
- [ ] Add `#[wasm_bindgen(js_name = isGameOver)]` wrapper
- [ ] Build WASM: `./scripts/build-wasm.sh`
- [ ] Verify build completes without errors

---

### 0.3: WebSocket Server Endpoints (5 min)

**File**: `apps/uci-server/src/connection.rs`

```bash
# Add 3 new message handlers:
1. validateMove -> moveValidation response
2. makeMove -> newPosition response
3. legalMoves -> legalMoves response
```

**Tasks**:

- [ ] Add `ValidateMoveMessage` struct
- [ ] Add `MakeMoveMessage` struct
- [ ] Add `"validateMove"` handler in `handle_client_message`
- [ ] Add `"makeMove"` handler in `handle_client_message`
- [ ] Build server: `cargo build -p uci-server`
- [ ] Test server starts: `cargo run -p uci-server`

---

## Phase 1: Dependencies (5 min)

**Goal**: Install react-chessboard (NO chess.js!)

### 1.1: Install Package

```bash
cd apps/web
pnpm add react-chessboard
```

**Tasks**:

- [ ] Run `pnpm add react-chessboard`
- [ ] Verify `package.json` updated
- [ ] Run `pnpm install` to ensure lock file updated
- [ ] Verify no errors

**Success**: `react-chessboard` in `package.json` dependencies

---

## Phase 2: Game State Management (2 hours)

**Goal**: Create Zustand store using engine APIs

### 2.1: Engine Client Extensions (45 min)

**File**: `apps/web/src/engine/engineClient.ts`

```typescript
// Add GameEngineClient interface + implementations
```

**Tasks**:

- [ ] Define `GameEngineClient` interface (validateMove, makeMove, legalMoves, checkGameOver)
- [ ] Implement fake mode client (always valid, flip FEN turn)
- [ ] Implement WASM mode client (call WASM methods)
- [ ] Implement remote mode client (fetch to WebSocket server)
- [ ] Export `useGameEngine()` hook
- [ ] Test in browser console with fake mode

---

### 2.2: Game Store (75 min)

**File**: `apps/web/src/store/gameStore.ts`

```typescript
// Create Zustand store with FEN-based state
```

**Tasks**:

- [ ] Define `GameState` interface
- [ ] Implement `newGame()` action (reset to startpos, engine moves if black)
- [ ] Implement `makeMove()` action (validate + apply + check game over)
- [ ] Implement `makeEngineMove()` action (analyze + apply move)
- [ ] Implement `resign()` action
- [ ] Implement `setDifficulty()` action
- [ ] Implement `resetGame()` action
- [ ] Test store in isolation (manual browser testing)

**Success**: Store manages game state using FEN + engine APIs

---

## Phase 3: Interactive Chessboard (2 hours)

**Goal**: Render board with drag-and-drop

### 3.1: Game Component (90 min)

**File**: `apps/web/src/components/Game.tsx`

```typescript
// Interactive chessboard with move validation
```

**Tasks**:

- [ ] Create `Game.tsx` component
- [ ] Import `react-chessboard` and `useGameStore`
- [ ] Implement `onDrop` handler (async move validation)
- [ ] Implement `isPlayerTurn()` helper (parse FEN turn)
- [ ] Add `customSquareStyles` for last move highlight
- [ ] Set `boardOrientation` from `playerColor`
- [ ] Set `arePiecesDraggable` based on turn + engine thinking
- [ ] Test in browser (should see board, can drag pieces)

---

### 3.2: Board Styling (30 min)

**File**: `apps/web/src/styles.css`

```css
/* Chessboard styles */
```

**Tasks**:

- [ ] Add `.game-container` grid layout (sidebar + board)
- [ ] Add `.chessboard` size (600x600px)
- [ ] Add responsive styles for mobile
- [ ] Test in browser (board renders correctly)

**Success**: Functional chessboard with drag-and-drop

---

## Phase 4: Engine Integration (2 hours)

**Goal**: Connect game store to engine for move requests

### 4.1: Integrate Engine Client (60 min)

**Tasks**:

- [ ] Update `gameStore.ts` to call `useGameEngine()`
- [ ] Wire `makeMove()` to engine validation
- [ ] Wire `makeEngineMove()` to analysis
- [ ] Test fake mode end-to-end (new game → make move → engine responds)
- [ ] Test WASM mode (if available)
- [ ] Test remote mode (server must be running)

---

### 4.2: Error Handling (30 min)

**Tasks**:

- [ ] Add error states to store
- [ ] Handle engine errors gracefully (show toast/alert)
- [ ] Handle network errors (remote mode)
- [ ] Test error scenarios (invalid FEN, engine crash)

---

### 4.3: Performance Testing (30 min)

**Tasks**:

- [ ] Test move validation latency (should be <10ms in WASM)
- [ ] Test engine move time at different depths (1-20)
- [ ] Verify UI doesn't freeze during engine thinking
- [ ] Test stop functionality during game

**Success**: Smooth gameplay with engine integration

---

## Phase 5: Game UI Components (2.5 hours)

**Goal**: Add game controls, status, move history

### 5.1: Game Controls (45 min)

**File**: `apps/web/src/components/GameControls.tsx`

**Tasks**:

- [ ] Create `GameControls` component
- [ ] Add "New Game" button
- [ ] Add "Resign" button
- [ ] Add "Play as" selector (white/black)
- [ ] Add "Difficulty" slider (1-20 depth)
- [ ] Wire to game store actions
- [ ] Test all controls work

---

### 5.2: Game Status (30 min)

**File**: `apps/web/src/components/GameStatus.tsx`

**Tasks**:

- [ ] Create `GameStatus` component
- [ ] Show "Engine thinking..." when `isEngineThinking`
- [ ] Show "Check!" when in check (use engine API)
- [ ] Show "Checkmate!" with winner
- [ ] Show "Stalemate - Draw"
- [ ] Show current turn (parse FEN)
- [ ] Test all statuses display correctly

---

### 5.3: Move History (45 min)

**File**: `apps/web/src/components/MoveHistory.tsx`

**Tasks**:

- [ ] Create `MoveHistory` component
- [ ] Display UCI moves from `moveHistory` array
- [ ] Format as 1. e2e4 e7e5 2. g1f3 ...
- [ ] Add scroll container (max height 400px)
- [ ] Style with monospace font
- [ ] Test with long game (20+ moves)

---

### 5.4: Captured Pieces (Optional, 30 min)

**File**: `apps/web/src/components/CapturedPieces.tsx`

**Tasks**:

- [ ] Parse FEN to determine material difference
- [ ] Display captured pieces with Unicode symbols
- [ ] Show material advantage (+3 for 3 pawns)
- [ ] Style nicely

---

## Phase 6: App Integration (1 hour)

**Goal**: Add "Play" mode to existing app

### 6.1: Mode Switcher (30 min)

**File**: `apps/web/src/App.tsx`

**Tasks**:

- [ ] Add `appMode` state (`'analysis' | 'play'`)
- [ ] Add mode switcher buttons (Analysis / Play vs Engine)
- [ ] Render `<AnalysisMode />` or `<PlayMode />` based on mode
- [ ] Style switcher (tabs or buttons)
- [ ] Test switching preserves existing analysis functionality

---

### 6.2: Play Mode Layout (30 min)

**File**: `apps/web/src/components/PlayMode.tsx`

**Tasks**:

- [ ] Create `PlayMode` component
- [ ] Layout: sidebar (controls + history) + board
- [ ] Import `Game`, `GameControls`, `GameStatus`, `MoveHistory`
- [ ] Arrange in grid layout
- [ ] Test full layout renders correctly

**Success**: Two distinct modes (Analysis + Play) in same app

---

## Phase 7: Polish & Testing (2 hours)

**Goal**: Ensure smooth UX and bug-free gameplay

### 7.1: Visual Polish (45 min)

**Tasks**:

- [ ] Improve button styles (hover, active, disabled states)
- [ ] Add transitions for smooth state changes
- [ ] Add loading indicators (engine thinking)
- [ ] Polish chessboard appearance
- [ ] Add game result modal/banner
- [ ] Test on mobile (responsive)

---

### 7.2: Manual Testing (60 min)

**Test Scenarios**:

- [ ] **New game as white**: e4 → engine responds → play to checkmate
- [ ] **New game as black**: Engine opens → respond → play to stalemate
- [ ] **Resign**: Click resign → game ends correctly
- [ ] **Difficulty**: Change depth mid-game → next move uses new depth
- [ ] **Illegal moves**: Try dragging to illegal square → rejected
- [ ] **Promotion**: Pawn to 8th rank → TODO: add promotion UI (Phase 8?)
- [ ] **Castling**: O-O and O-O-O work correctly
- [ ] **En passant**: Test edge case
- [ ] **Fake mode**: Plays fake moves, no errors
- [ ] **WASM mode**: (if available) Fast, offline gameplay
- [ ] **Remote mode**: (server running) Correct analysis, SearchInfo streams
- [ ] **Mode switching**: Switch between Analysis and Play modes → no crashes

---

### 7.3: Bug Fixes (15 min)

**Tasks**:

- [ ] Fix any issues found during testing
- [ ] Handle edge cases (empty FEN, invalid moves)
- [ ] Improve error messages

---

## Success Criteria ✅

### MVP Complete When:

- [x] User can start a new game as white or black
- [x] Interactive board allows piece movement via drag-and-drop
- [x] Engine responds with moves based on selected difficulty
- [x] Game detects checkmate, stalemate, and draw conditions
- [x] Move history displays (UCI notation for MVP)
- [x] Works with all three engine modes (fake/WASM/remote)
- [x] Users can resign and start a new game
- [x] Existing analysis mode still works
- [x] UI is responsive and polished

---

## Implementation Notes

### **Critical Path**:

1. Phase 0 (engine APIs) **must** be done first
2. Phase 1-4 are sequential (dependencies)
3. Phase 5-7 can be done in parallel (UI components)

### **Testing Strategy**:

- Test each phase in isolation before moving to next
- Use fake mode for rapid iteration
- Test WASM mode if available (mate detection bug may be fixed)
- Test remote mode with `cargo run -p uci-server` running

### **Gotchas**:

- **Promotion**: react-chessboard has built-in promotion dialog, but you need to handle the 5th character (e7e8q)
- **FEN parsing**: Turn color is in position [1] after split by space
- **Engine thinking**: Disable dragging during `isEngineThinking`
- **Stop functionality**: Already implemented from WebSocket server work

### **Post-MVP Enhancements** (Not in this plan):

- [ ] Algebraic notation (SAN) instead of UCI
- [ ] Time controls (blitz, rapid, classical)
- [ ] Opening book display
- [ ] Undo move
- [ ] PGN export
- [ ] Game analysis after playing

---

## Commands Reference

```bash
# Phase 0: Build engine + server
cargo build -p engine
./scripts/build-wasm.sh
cargo build -p uci-server
cargo run -p uci-server  # Start server

# Phase 1-7: Frontend
cd apps/web
pnpm install
pnpm dev  # Start dev server

# Testing
cargo test -p engine  # Test engine APIs
```

---

**Created**: 2025-10-26
**Status**: Ready to implement
**Estimated Time**: 12 hours
**Approach**: Engine-based (no chess.js)
