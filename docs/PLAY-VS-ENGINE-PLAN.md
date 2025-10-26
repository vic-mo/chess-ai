# Play vs Engine Feature - Implementation Plan

**Status**: 📋 Planning
**Priority**: High
**Estimated Time**: 12 hours

---

## Objectives

Implement an interactive chess game mode where users can play against the AI engine in the browser.

---

## Overhead Analysis: Move Validation

### **Option 1: chess.js (Client-side)**

- **Bundle size**: ~40KB (minified)
- **Validation time**: <1ms per move (in-memory)
- **Pros**: Instant, works offline
- **Cons**: 40KB bundle size, **duplicates chess logic we already have**, risk of divergence from engine

### **Option 2: Use Our Engine (WASM/Remote)**

- **WASM overhead**: 2-5ms per validation (worker message passing)
- **Remote overhead**: 10-100ms (network RTT)
- **Pros**: Single source of truth, zero bundle cost (engine already loaded), guaranteed consistency
- **Cons**: Slightly slower than chess.js, requires engine extensions

### **Recommendation: Use Our Engine** ✅

- **Zero bundle cost** - engine already in WASM/remote modes
- **Single source of truth** - same engine for analysis and gameplay
- **Guaranteed consistency** - can't diverge from engine logic
- **2-5ms WASM latency** is imperceptible for human play
- **Simple extensions** - add 3 methods to engine (~30 min work)
- **Works in all modes** - fake/WASM/remote (existing infrastructure)

---

## Implementation Plan

### **Phase 0: Engine API Extensions** (30 min) ⭐ NEW

#### Add game-specific methods to engine core

**File**: `crates/engine/src/lib.rs`

```rust
impl EngineImpl {
    /// Validate if a UCI move is legal in the given position
    pub fn is_move_legal(&self, fen: &str, uci_move: &str) -> bool {
        let board = match parse_fen(fen) {
            Ok(b) => b,
            Err(_) => return false,
        };

        let legal_moves = board.generate_legal_moves();
        legal_moves.iter().any(|m| m.to_uci() == uci_move)
    }

    /// Apply a UCI move and return the new FEN
    pub fn make_move(&mut self, fen: &str, uci_move: &str) -> Result<String, String> {
        let mut board = parse_fen(fen).map_err(|e| format!("Invalid FEN: {}", e))?;

        let legal_moves = board.generate_legal_moves();
        let mv = legal_moves.iter()
            .find(|m| m.to_uci() == uci_move)
            .ok_or("Illegal move")?;

        board.make_move(*mv);
        Ok(board.to_fen())
    }

    /// Get all legal moves for a position
    pub fn legal_moves(&self, fen: &str) -> Vec<String> {
        match parse_fen(fen) {
            Ok(board) => board.generate_legal_moves()
                .iter()
                .map(|m| m.to_uci())
                .collect(),
            Err(_) => vec![],
        }
    }

    /// Check if position is game over (checkmate, stalemate)
    pub fn is_game_over(&self, fen: &str) -> (bool, Option<String>) {
        match parse_fen(fen) {
            Ok(board) => {
                if board.generate_legal_moves().len() == 0 {
                    if board.is_in_check() {
                        (true, Some("checkmate".to_string()))
                    } else {
                        (true, Some("stalemate".to_string()))
                    }
                } else {
                    (false, None)
                }
            }
            Err(_) => (false, None),
        }
    }
}
```

#### Expose in WASM bindings

**File**: `crates/engine-bridge-wasm/src/lib.rs`

Add to existing WASM API:

```rust
#[wasm_bindgen]
impl WasmEngine {
    #[wasm_bindgen(js_name = isMoveLegal)]
    pub fn is_move_legal(&self, fen: &str, uci_move: &str) -> bool {
        let eng = self.engine.lock().unwrap();
        eng.is_move_legal(fen, uci_move)
    }

    #[wasm_bindgen(js_name = makeMove)]
    pub fn make_move(&mut self, fen: &str, uci_move: &str) -> Result<String, JsValue> {
        let mut eng = self.engine.lock().unwrap();
        eng.make_move(fen, uci_move)
            .map_err(|e| JsValue::from_str(&e))
    }

    #[wasm_bindgen(js_name = legalMoves)]
    pub fn legal_moves(&self, fen: &str) -> Vec<JsValue> {
        let eng = self.engine.lock().unwrap();
        eng.legal_moves(fen)
            .into_iter()
            .map(|s| JsValue::from_str(&s))
            .collect()
    }
}
```

#### Add WebSocket endpoints

**File**: `apps/uci-server/src/connection.rs`

```rust
// Add new message types
#[derive(Debug, Deserialize)]
struct ValidateMoveMessage {
    fen: String,
    uci_move: String,
}

#[derive(Debug, Deserialize)]
struct MakeMoveMessage {
    fen: String,
    uci_move: String,
}

// In handle_client_message:
"validateMove" => {
    let msg: ValidateMoveMessage = serde_json::from_value(msg.payload)?;
    let valid = engine.is_move_legal(&msg.fen, &msg.uci_move);
    let response = ServerMessage {
        msg_type: "moveValidation".to_string(),
        id: msg.id,
        payload: serde_json::json!({ "valid": valid }),
    };
    tx.send(response)?;
}
"makeMove" => {
    let msg: MakeMoveMessage = serde_json::from_value(msg.payload)?;
    match engine.make_move(&msg.fen, &msg.uci_move) {
        Ok(new_fen) => {
            let response = ServerMessage {
                msg_type: "newPosition".to_string(),
                id: msg.id,
                payload: serde_json::json!({ "fen": new_fen }),
            };
            tx.send(response)?;
        }
        Err(e) => {
            let response = ServerMessage {
                msg_type: "error".to_string(),
                id: msg.id,
                payload: serde_json::json!({ "message": e }),
            };
            tx.send(response)?;
        }
    }
}
```

---

### **Phase 1: Setup Dependencies** (5 min)

#### Install packages:

```bash
pnpm add react-chessboard
```

#### Dependencies:

- **react-chessboard** (~200KB): Interactive board UI with drag-and-drop
- **NO chess.js** - using our engine instead!

---

### **Phase 2: Game State Management** (2 hours)

#### Create `apps/web/src/store/gameStore.ts`

```typescript
import { create } from 'zustand';
import { useEngine } from '../engine/engineClient';

interface GameState {
  // Game state (from engine)
  currentFen: string;
  playerColor: 'white' | 'black';
  difficulty: number; // search depth 1-20
  gameStatus: 'playing' | 'checkmate' | 'stalemate' | 'draw' | 'resigned';
  winner: 'white' | 'black' | 'draw' | null;
  moveHistory: string[]; // UCI moves

  // UI state
  isEngineThinking: boolean;
  lastMove: { from: string; to: string } | null;
  legalMoves: string[]; // For highlighting

  // Actions
  newGame: (playerColor: 'white' | 'black', difficulty: number) => Promise<void>;
  makeMove: (from: string, to: string, promotion?: string) => Promise<boolean>;
  makeEngineMove: () => Promise<void>;
  resign: () => void;
  setDifficulty: (depth: number) => void;
  resetGame: () => void;
}

export const useGameStore = create<GameState>((set, get) => ({
  currentFen: 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1',
  playerColor: 'white',
  difficulty: 8,
  gameStatus: 'playing',
  winner: null,
  moveHistory: [],
  isEngineThinking: false,
  lastMove: null,
  legalMoves: [],

  newGame: async (playerColor, difficulty) => {
    const startFen = 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1';
    set({
      currentFen: startFen,
      playerColor,
      difficulty,
      gameStatus: 'playing',
      winner: null,
      moveHistory: [],
      lastMove: null,
      isEngineThinking: false,
    });

    // If playing as black, engine goes first
    if (playerColor === 'black') {
      await get().makeEngineMove();
    }
  },

  makeMove: async (from, to, promotion) => {
    const { currentFen } = get();
    const engine = useEngine(); // Get engine client

    const uciMove = `${from}${to}${promotion || ''}`;

    // Validate move with engine
    const valid = await engine.validateMove(currentFen, uciMove);
    if (!valid) return false;

    // Apply move via engine
    const newFen = await engine.makeMove(currentFen, uciMove);

    // Check game status
    const { isOver, status } = await engine.checkGameOver(newFen);

    set({
      currentFen: newFen,
      lastMove: { from, to },
      moveHistory: [...get().moveHistory, uciMove],
      gameStatus: isOver ? status || 'playing' : 'playing',
    });

    // If not game over, engine moves
    if (!isOver && get().gameStatus === 'playing') {
      await get().makeEngineMove();
    }

    return true;
  },

  makeEngineMove: async () => {
    set({ isEngineThinking: true });

    const { currentFen, difficulty } = get();
    const engine = useEngine();

    // Request best move from engine
    const bestMove = await engine.analyze(currentFen, difficulty);

    // Apply engine's move
    const newFen = await engine.makeMove(currentFen, bestMove);

    // Check game status
    const { isOver, status } = await engine.checkGameOver(newFen);

    set({
      currentFen: newFen,
      lastMove: { from: bestMove.slice(0, 2), to: bestMove.slice(2, 4) },
      moveHistory: [...get().moveHistory, bestMove],
      gameStatus: isOver ? status || 'playing' : 'playing',
      isEngineThinking: false,
    });
  },

  resign: () => {
    const { playerColor } = get();
    set({
      gameStatus: 'resigned',
      winner: playerColor === 'white' ? 'black' : 'white',
    });
  },

  setDifficulty: (depth) => set({ difficulty: depth }),

  resetGame: () => get().newGame(get().playerColor, get().difficulty),
}));
```

#### Key functionality:

- Store current FEN position (from engine)
- Use engine APIs for move validation and application
- Handle user moves with async engine validation
- Request and apply engine moves
- Detect game over via engine

---

### **Phase 3: Interactive Chessboard** (2 hours)

#### Create `apps/web/src/components/Game.tsx`

```typescript
import { Chessboard } from 'react-chessboard';
import { useGameStore } from '../store/gameStore';

export function Game() {
  const { currentFen, playerColor, makeMove, isEngineThinking, lastMove } = useGameStore();

  const onDrop = async (sourceSquare: string, targetSquare: string) => {
    // Validate and make move (engine handles validation)
    const success = await makeMove(sourceSquare, targetSquare);
    return success;
  };

  // Determine if it's player's turn
  const isPlayerTurn = () => {
    const turnColor = currentFen.split(' ')[1]; // 'w' or 'b'
    return (turnColor === 'w' && playerColor === 'white') ||
           (turnColor === 'b' && playerColor === 'black');
  };

  return (
    <Chessboard
      position={currentFen}
      onPieceDrop={onDrop}
      boardOrientation={playerColor}
      arePiecesDraggable={!isEngineThinking && isPlayerTurn()}
      customBoardStyle={{
        borderRadius: '4px',
        boxShadow: '0 2px 10px rgba(0,0,0,0.3)'
      }}
      customSquareStyles={lastMove ? {
        [lastMove.from]: { backgroundColor: 'rgba(255, 255, 0, 0.4)' },
        [lastMove.to]: { backgroundColor: 'rgba(255, 255, 0, 0.4)' },
      } : {}}
    />
  );
}
```

#### Features:

- Drag-and-drop piece movement with async validation
- Board orientation based on player color
- Disable dragging during engine thinking
- Highlight last move automatically
- Position from FEN (engine source of truth)

---

### **Phase 4: Engine Integration** (2 hours)

#### Extend `apps/web/src/engine/engineClient.ts`

Add game-specific methods to the existing engine client:

```typescript
// Add to existing engineClient.ts

export interface GameEngineClient {
  validateMove(fen: string, uciMove: string): Promise<boolean>;
  makeMove(fen: string, uciMove: string): Promise<string>;
  legalMoves(fen: string): Promise<string[]>;
  checkGameOver(fen: string): Promise<{ isOver: boolean; status?: string }>;
}

// Implement for each engine mode (fake/WASM/remote)
function createGameClient(mode: EngineMode): GameEngineClient {
  if (mode === 'fake') {
    return {
      async validateMove(fen, uciMove) {
        // Fake: always return true
        return true;
      },
      async makeMove(fen, uciMove) {
        // Fake: just return FEN with flipped turn
        return fen.replace(' w ', ' b ').replace(' b ', ' w ');
      },
      async legalMoves(fen) {
        return ['e2e4', 'e7e5']; // Fake moves
      },
      async checkGameOver(fen) {
        return { isOver: false };
      },
    };
  }

  if (mode === 'wasm') {
    return {
      async validateMove(fen, uciMove) {
        const worker = await getWasmWorker();
        return worker.isMoveLegal(fen, uciMove);
      },
      async makeMove(fen, uciMove) {
        const worker = await getWasmWorker();
        return worker.makeMove(fen, uciMove);
      },
      async legalMoves(fen) {
        const worker = await getWasmWorker();
        return worker.legalMoves(fen);
      },
      async checkGameOver(fen) {
        const worker = await getWasmWorker();
        const [isOver, status] = worker.isGameOver(fen);
        return { isOver, status };
      },
    };
  }

  // mode === 'remote'
  return {
    async validateMove(fen, uciMove) {
      const res = await fetch(WS_BASE + '/validateMove', {
        method: 'POST',
        body: JSON.stringify({ fen, uci_move: uciMove }),
      });
      const { valid } = await res.json();
      return valid;
    },
    async makeMove(fen, uciMove) {
      const res = await fetch(WS_BASE + '/makeMove', {
        method: 'POST',
        body: JSON.stringify({ fen, uci_move: uciMove }),
      });
      const { fen: newFen } = await res.json();
      return newFen;
    },
    async legalMoves(fen) {
      const res = await fetch(WS_BASE + '/legalMoves', {
        method: 'POST',
        body: JSON.stringify({ fen }),
      });
      const { moves } = await res.json();
      return moves;
    },
    async checkGameOver(fen) {
      const res = await fetch(WS_BASE + '/gameStatus', {
        method: 'POST',
        body: JSON.stringify({ fen }),
      });
      const { isOver, status } = await res.json();
      return { isOver, status };
    },
  };
}

// Export for game store
export function useGameEngine(): GameEngineClient {
  const mode = getEngineMode();
  return createGameClient(mode);
}
```

#### Key considerations:

- Reuse existing engine infrastructure (fake/WASM/remote)
- Add async wrapper methods for game operations
- Handle all three engine modes uniformly
- Parse UCI moves (e.g., "e2e4", "e7e8q") in engine core

---

### **Phase 5: Game UI Components** (2.5 hours)

#### Create `apps/web/src/components/GameControls.tsx`

```typescript
export function GameControls() {
  return (
    <div className="game-controls">
      <button onClick={newGame}>New Game</button>
      <button onClick={resign}>Resign</button>

      <label>
        Play as:
        <select value={playerColor} onChange={...}>
          <option value="white">White</option>
          <option value="black">Black</option>
        </select>
      </label>

      <label>
        Difficulty (depth):
        <input type="range" min="1" max="20" value={difficulty} />
        <span>{difficulty}</span>
      </label>
    </div>
  );
}
```

#### Create `apps/web/src/components/MoveHistory.tsx`

```typescript
export function MoveHistory() {
  const { moveHistory } = useGameStore();

  // Convert UCI moves to simple notation (e2e4 → e4, g1f3 → Nf3)
  const formatMove = (uciMove: string) => {
    // Simplified: just show UCI for now
    // TODO: Convert to SAN using engine
    return uciMove;
  };

  return (
    <div className="move-history">
      <h3>Moves</h3>
      <div className="moves-list">
        {moveHistory.map((move, i) => (
          <div key={i}>
            {i % 2 === 0 && <span>{Math.floor(i/2) + 1}.</span>}
            <span>{formatMove(move)}</span>
          </div>
        ))}
      </div>
    </div>
  );
}
```

**Note**: For MVP, display UCI moves. Post-MVP: add `uci_to_san()` method to engine for algebraic notation.

#### Create `apps/web/src/components/GameStatus.tsx`

```typescript
export function GameStatus() {
  const { gameStatus, winner, currentFen, isEngineThinking } = useGameStore();
  const engine = useGameEngine();

  const [isCheck, setIsCheck] = useState(false);

  useEffect(() => {
    // Check if position is in check
    engine.checkGameOver(currentFen).then(({ status }) => {
      setIsCheck(status === 'check');
    });
  }, [currentFen]);

  if (isEngineThinking) return <div>🤔 Engine thinking...</div>;
  if (isCheck) return <div>⚠️ Check!</div>;
  if (gameStatus === 'checkmate') return <div>🏁 Checkmate! {winner} wins</div>;
  if (gameStatus === 'stalemate') return <div>🏁 Stalemate - Draw</div>;
  if (gameStatus === 'draw') return <div>🏁 Draw</div>;

  const turnColor = currentFen.split(' ')[1]; // 'w' or 'b'
  return <div>▶️ {turnColor === 'w' ? 'White' : 'Black'} to move</div>;
}
```

#### Create `apps/web/src/components/CapturedPieces.tsx`

- Display captured pieces with Unicode symbols
- Count material advantage

#### UI Layout:

```
┌─────────────────────────────────────┐
│  Move History    │   Chessboard     │
│  1. e4 e5        │   ♜ ♞ ♝ ♛ ♚ ...  │
│  2. Nf3 Nc6      │                   │
│  3. ...          │                   │
│                  │                   │
│  Captured: ♟♟♞   │                   │
├──────────────────┴───────────────────┤
│  Status: White to move               │
│  [New Game] [Resign] Difficulty: 8   │
└─────────────────────────────────────┘
```

---

### **Phase 6: App Integration** (1 hour)

#### Modify `apps/web/src/App.tsx`

Add mode switching:

```typescript
type AppMode = 'analysis' | 'play';
const [appMode, setAppMode] = useState<AppMode>('analysis');

return (
  <div className="container">
    <h1>♟️ Chess AI</h1>

    {/* Mode switcher */}
    <div className="mode-switch">
      <button onClick={() => setAppMode('analysis')}>Analysis</button>
      <button onClick={() => setAppMode('play')}>Play vs Engine</button>
    </div>

    {appMode === 'analysis' ? (
      <AnalysisMode /> // Existing analysis UI
    ) : (
      <PlayMode /> // New game UI
    )}
  </div>
);
```

#### Keep existing analysis functionality intact:

- Analysis mode: Current FEN input + depth selection + analyze button
- Play mode: Interactive game with engine opponent

---

### **Phase 7: Polish & Testing** (2 hours)

#### Styling (`apps/web/src/styles.css`)

```css
.game-container {
  display: grid;
  grid-template-columns: 300px 600px;
  gap: 2rem;
}

.chessboard {
  width: 600px;
  height: 600px;
}

.move-history {
  max-height: 400px;
  overflow-y: auto;
  font-family: monospace;
}

.game-status {
  font-size: 1.2rem;
  font-weight: bold;
  padding: 1rem;
  text-align: center;
}
```

#### Testing scenarios:

1. **Play as white**: e4 → engine responds → continue to checkmate
2. **Play as black**: Engine opens → respond → verify turn switching
3. **Difficulty test**: Depth 1 (weak) vs Depth 15 (strong)
4. **Game endings**:
   - Checkmate detection
   - Stalemate (e.g., king vs king)
   - Threefold repetition
5. **All engine modes**:
   - Fake mode (random moves)
   - WASM mode (if working)
   - Remote mode (if server running)
6. **Edge cases**:
   - Promotion handling (pawn to 8th rank)
   - En passant
   - Castling
7. **UX**:
   - Resign → game over message
   - New game → reset board
   - Change difficulty mid-game

#### Manual test checklist:

- [ ] Board renders correctly
- [ ] Pieces drag smoothly
- [ ] Legal moves validated
- [ ] Engine responds within reasonable time
- [ ] Checkmate detected correctly
- [ ] Move history displays in algebraic notation
- [ ] Difficulty slider affects engine strength
- [ ] Can play full game from start to finish
- [ ] Mode switching preserves analysis functionality

---

## File Structure Summary

```
apps/web/src/
├── components/
│   ├── Game.tsx              # Main game component (NEW)
│   ├── GameBoard.tsx         # Chessboard wrapper (NEW)
│   ├── GameControls.tsx      # New game, resign, settings (NEW)
│   ├── GameStatus.tsx        # Check, checkmate, turn display (NEW)
│   ├── MoveHistory.tsx       # Move list (NEW)
│   └── CapturedPieces.tsx    # Material count (NEW)
├── store/
│   └── gameStore.ts          # Game state management (NEW)
├── engine/
│   └── engineClient.ts       # Add useMoveEngine() hook (MODIFY)
├── App.tsx                   # Add play mode (MODIFY)
└── styles.css               # Game UI styling (MODIFY)
```

---

## Key Technical Decisions

### 1. **Move Validation: Our Engine** ✅

- **Winner**: Zero bundle cost, single source of truth, guaranteed consistency
- **Tradeoff**: 2-5ms WASM latency vs <1ms chess.js (imperceptible to humans)
- **Rejected**: chess.js (code duplication), server-only (doesn't work offline)

### 2. **Engine Difficulty: Search Depth**

- Range: 1 (beginner) to 20 (expert)
- Depth 1: ~0.1s, weak moves
- Depth 8: ~1-3s, decent play
- Depth 15: ~10-30s, strong play

### 3. **Board Library: react-chessboard**

- Mature, well-maintained
- Drag-and-drop built-in
- Customizable styles
- 200KB bundle (acceptable)

### 4. **State Management: Zustand**

- Already in dependencies
- Simpler than Redux
- Perfect for game state

---

## Timeline Estimate

| Phase     | Task                      | Time          |
| --------- | ------------------------- | ------------- |
| **0**     | **Engine API extensions** | **30 min**    |
| 1         | Dependencies              | 5 min         |
| 2         | Game store (engine-based) | 2 hrs         |
| 3         | Chessboard UI             | 2 hrs         |
| 4         | Engine integration        | 2 hrs         |
| 5         | Game controls & info      | 2.5 hrs       |
| 6         | App integration           | 1 hr          |
| 7         | Polish & testing          | 2 hrs         |
| **Total** |                           | **~12 hours** |

**Note**: Phase 0 is new but only adds 30 min. Other phases are slightly faster due to simpler architecture (no chess.js).

---

## Future Enhancements (Post-MVP)

- **Time controls**: Blitz (3+0), Rapid (10+0), Classical (30+0)
- **Opening book**: Display opening name
- **Position evaluation**: Live eval bar during game
- **Undo move**: Take back last move
- **PGN export**: Download game
- **Game analysis**: Review game with engine after playing
- **Multiple difficulty presets**: Beginner/Intermediate/Master
- **Hints**: Show best move button
- **Statistics**: Track wins/losses/draws

---

## Success Metrics

### MVP Complete When:

- ✅ User can start a new game as white or black
- ✅ Interactive board allows piece movement via drag-and-drop
- ✅ Engine responds with moves based on selected difficulty
- ✅ Game detects checkmate, stalemate, and draw conditions
- ✅ Move history displays in algebraic notation
- ✅ Works with all three engine modes (fake/WASM/remote)
- ✅ Users can resign and start a new game
- ✅ Existing analysis mode still works
- ✅ UI is responsive and polished

---

**Created**: 2025-10-26
**Revised**: 2025-10-26 (replaced chess.js with engine APIs)
**Status**: Ready for implementation

---

## Revision Summary

**What Changed**:

- ❌ Removed chess.js dependency
- ✅ Added Phase 0: Engine API extensions (30 min)
- ✅ Updated Phase 2: Game store uses engine APIs
- ✅ Updated Phase 4: Engine client extensions
- ✅ Updated all code examples to use FEN + engine

**Why**:

- **Single source of truth**: Engine is already doing all chess logic
- **Zero bundle cost**: Engine already loaded in WASM/remote modes
- **Guaranteed consistency**: Can't diverge from engine behavior
- **2-5ms latency**: Imperceptible to humans vs chess.js <1ms

**Trade-off Accepted**: Slightly slower move validation (2-5ms vs <1ms) for architectural cleanliness and zero bundle cost.
