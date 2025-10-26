# Production Plan: WebSocket Server + Playable Chess UI

**Goal**: Production-ready chess engine with playable UI
**Timeline**: 5 sessions (~12-16 hours)
**Priority**: High

---

## Desired End State

### 3 Core Requirements:

1. ✅ **Production WebSocket Server**
   - Handles multiple concurrent connections
   - Bidirectional communication (analyze, stop, get best move)
   - Reliable, observable, deployable

2. ✅ **Engine Analyzes FEN Positions via Server**
   - Engine works in native mode ✅ (M9 complete)
   - **NEEDS**: WebSocket server backend
   - **NEEDS**: Connect existing analysis UI to server ("remote" mode)

3. ✅ **Playable Chess UI**
   - User plays as White/Black against engine
   - Interactive chess board
   - Engine responds to user moves
   - Game state management (legal moves, check, checkmate, draw)

---

## Current State Assessment

### ✅ What Works

- **Native engine**: Searcher with alpha-beta, pruning, extensions
- **FEN parsing**: `parse_fen()` converts FEN → Board
- **Move generation**: Legal move generation working
- **Analysis**: Engine can analyze positions and return best moves
- **Protocol types**: SearchInfo, BestMove, SearchLimit all defined
- **Web UI skeleton**: React app with engine modes (fake/remote/wasm)
- **433 tests passing**: Core engine validated

### ❌ What's Missing

- **No WebSocket server** - Only fake/WASM modes exist
- **Remote mode not implemented** - UI has "remote" option but no backend
- **No playable UI** - Current UI is analysis-only (paste FEN, analyze)
- **No game loop** - No state management for playing a full game
- **No move validation UI** - Can't make moves interactively
- **No chess board display** - Just logs, no visual board

### ⚠️ What's Broken

- **WASM mode disabled** - Mate detection bug (not needed for production plan)

---

## Architecture Overview

### Target Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      Browser (React)                        │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              Playable Chess UI                      │   │
│  │  • Interactive chessboard (react-chessboard)       │   │
│  │  • Game state management (chess.js)                │   │
│  │  • User moves → WebSocket → Engine response        │   │
│  └────────────────────────┬────────────────────────────┘   │
└───────────────────────────┼────────────────────────────────┘
                            │ WebSocket
                            │
┌───────────────────────────▼────────────────────────────────┐
│                  WebSocket Server (Rust/Tokio)             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  • Connection manager (1 engine per connection)     │   │
│  │  • Message routing (analyze, stop, get_best_move)   │   │
│  │  • Engine process spawning                          │   │
│  └────────────────────────┬────────────────────────────┘   │
└───────────────────────────┼────────────────────────────────┘
                            │ UCI Protocol
                            │
┌───────────────────────────▼────────────────────────────────┐
│              Chess Engine (Native Binary)                  │
│  • EngineImpl with Searcher                                │
│  • Alpha-beta search                                       │
│  • FEN → Board → Best Move                                 │
└────────────────────────────────────────────────────────────┘
```

---

## High-Level Plan

### Phase 1: WebSocket Server Foundation (Sessions 1-2)

**Goal**: Production-ready WebSocket server that connects browser to engine

### Phase 2: Connect Existing Analysis UI (Session 3)

**Goal**: Wire current analysis UI to WebSocket server (implement "remote" mode)

### Phase 3: Playable Chess UI (Session 4)

**Goal**: Interactive chess board where user can play against engine

### Phase 4: Polish & Deploy (Session 5)

**Goal**: Production deployment with monitoring and error handling

---

## Session Breakdown

---

### Session 1: WebSocket Server - Core Infrastructure 🔴

**Duration**: 3-4 hours

**Objective**: Build WebSocket server that can receive messages and spawn engine processes

**Implementation**:

1. **Create `apps/uci-server` package**

   ```
   apps/uci-server/
   ├── Cargo.toml
   └── src/
       ├── main.rs          # Server entry point
       ├── connection.rs    # Per-connection handler
       └── engine.rs        # Engine process manager
   ```

2. **WebSocket Server (main.rs)**
   - Use `tokio` + `tokio-tungstenite` for async WebSocket
   - Listen on `localhost:8080`
   - Accept connections and spawn connection handler
   - Handle graceful shutdown

3. **Connection Handler (connection.rs)**
   - One engine process per connection
   - Parse incoming JSON messages
   - Route to engine handler
   - Send responses back to client

4. **Engine Process Manager (engine.rs)**
   - Spawn engine binary as subprocess (or use in-process EngineImpl)
   - Send UCI commands via stdin
   - Read UCI responses from stdout
   - Convert UCI ↔ Protocol types

**Protocol Design**:

```json
// Client → Server: Analyze position
{
  "type": "analyze",
  "id": "req-uuid",
  "fen": "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
  "limit": { "kind": "depth", "depth": 10 }
}

// Server → Client: Search info (streaming, multiple messages)
{
  "type": "searchInfo",
  "id": "req-uuid",
  "payload": {
    "depth": 5,
    "score": { "kind": "cp", "value": 25 },
    "pv": ["e2e4", "e7e5", "g1f3"],
    "nodes": 12453,
    "nps": 8234,
    "time_ms": 125
  }
}

// Server → Client: Best move (final message)
{
  "type": "bestMove",
  "id": "req-uuid",
  "payload": {
    "best": "e2e4",
    "ponder": "e7e5"
  }
}

// Client → Server: Stop search
{
  "type": "stop",
  "id": "req-uuid"
}
```

**Success Criteria**:

- ✅ Server starts and listens on port 8080
- ✅ Accepts WebSocket connections
- ✅ Spawns engine process per connection
- ✅ Can parse and route messages
- ✅ Sends dummy response back to client

**Files to Create**:

- `apps/uci-server/Cargo.toml`
- `apps/uci-server/src/main.rs`
- `apps/uci-server/src/connection.rs`
- `apps/uci-server/src/engine.rs`

**Dependencies**:

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
tokio-tungstenite = "0.21"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
engine = { path = "../../crates/engine" }
protocol = { path = "../../crates/protocol" }
anyhow = "1"
tracing = "0.1"
tracing-subscriber = "0.3"
```

---

### Session 2: WebSocket Server - Engine Integration 🔴

**Duration**: 2-3 hours

**Objective**: Wire WebSocket messages to actual engine analysis

**Implementation**:

1. **Engine Integration (engine.rs)**
   - Use `EngineImpl` directly (in-process, not subprocess)
   - Handle `position` command (set FEN)
   - Handle `analyze` command (with callback)
   - Handle `stop` command
   - Stream SearchInfo back through WebSocket

2. **Message Routing (connection.rs)**

   ```rust
   async fn handle_message(&mut self, msg: ClientMessage) -> Result<()> {
       match msg.msg_type.as_str() {
           "analyze" => {
               let id = msg.id;
               let fen = msg.fen;
               let limit = msg.limit;

               // Set position
               self.engine.position(&fen, &[])?;

               // Analyze with callback for SearchInfo streaming
               let tx = self.ws_sender.clone();
               let best = self.engine.analyze(limit, move |info| {
                   // Send SearchInfo to WebSocket
                   let msg = ServerMessage {
                       msg_type: "searchInfo".to_string(),
                       id: id.clone(),
                       payload: serde_json::to_value(&info).unwrap(),
                   };
                   let _ = tx.send(msg);
               });

               // Send BestMove to WebSocket
               self.send_best_move(&id, best).await;
           }
           "stop" => {
               self.engine.stop();
           }
           _ => {
               eprintln!("Unknown message type: {}", msg.msg_type);
           }
       }
   }
   ```

3. **Async Callback Handling**
   - Use `tokio::sync::mpsc` channel for SearchInfo streaming
   - Spawn task to send SearchInfo messages
   - Ensure thread-safe communication

**Success Criteria**:

- ✅ Client sends `getBestMove` → Server returns real move
- ✅ Server streams SearchInfo updates during analysis
- ✅ Client sends `stop` → Search terminates
- ✅ Multiple connections work independently
- ✅ No crashes or deadlocks

**Testing**:

- Write integration test with WebSocket client
- Test with `wscat` CLI tool
- Verify analysis on startpos, tactical positions, endgames

---

### Session 3: Connect Existing Analysis UI to WebSocket Server 🟡

**Duration**: 2-3 hours

**Objective**: Wire existing analysis UI (App.tsx) to WebSocket server - implement "remote" mode

**Implementation**:

1. **Understand Current UI State**
   - `apps/web/src/App.tsx` already has analysis UI:
     - FEN input
     - Depth selector
     - Analyze/Stop buttons
     - Engine mode selector (fake/wasm/remote)
   - "Remote" mode exists but not implemented
   - Uses `useEngine()` hook from `engineClient.ts`

2. **Implement Remote Engine Client**

   ```typescript
   // apps/web/src/engine/remoteEngine.ts
   export class RemoteEngine implements EngineClient {
     private ws: WebSocket;
     private callbacks: Map<string, (evt: EngineEvent) => void>;

     constructor(serverUrl: string = 'ws://localhost:8080') {
       this.ws = new WebSocket(serverUrl);
       this.callbacks = new Map();

       this.ws.onmessage = (evt) => {
         const msg = JSON.parse(evt.data);
         const callback = this.callbacks.get(msg.id);

         if (msg.type === 'searchInfo') {
           callback?.({ type: 'searchInfo', payload: msg.payload });
         } else if (msg.type === 'bestMove') {
           callback?.({ type: 'bestMove', payload: msg.payload });
           this.callbacks.delete(msg.id);
         }
       };
     }

     analyze(req: AnalyzeRequest, callback: (evt: EngineEvent) => void): () => void {
       this.callbacks.set(req.id, callback);

       this.ws.send(
         JSON.stringify({
           type: 'analyze',
           id: req.id,
           fen: req.fen,
           limit: req.limit,
         }),
       );

       return () => this.stop(req.id);
     }

     stop(id: string): void {
       this.ws.send(JSON.stringify({ type: 'stop', id }));
       this.callbacks.delete(id);
     }
   }
   ```

3. **Wire Remote Engine to `useEngine` Hook**

   ```typescript
   // apps/web/src/engine/engineClient.ts
   let remoteEngine: RemoteEngine | null = null;

   export function useEngine(): EngineClient {
     const mode = getEngineMode();

     if (mode === 'remote') {
       if (!remoteEngine) {
         remoteEngine = new RemoteEngine('ws://localhost:8080');
       }
       return remoteEngine;
     }
     // ... existing fake/wasm logic
   }
   ```

4. **Update Server URL Configuration**

   ```typescript
   // apps/web/src/config.ts
   export const REMOTE_ENGINE_URL = import.meta.env.VITE_ENGINE_URL || 'ws://localhost:8080';
   ```

5. **Test Remote Mode in Existing UI**
   - Start WebSocket server: `cargo run -p uci-server`
   - Open web UI, select "Remote (Server)" mode
   - Paste FEN, set depth, click "Analyze"
   - Should see SearchInfo streaming in logs
   - Should see bestMove result

**Success Criteria**:

- ✅ "Remote" mode connects to WebSocket server
- ✅ Analyze button sends request to server
- ✅ SearchInfo logs appear in real-time
- ✅ BestMove displayed when search completes
- ✅ Stop button terminates search
- ✅ Can analyze multiple positions without reconnecting

**Files to Create**:

- `apps/web/src/engine/remoteEngine.ts`
- `apps/web/src/config.ts`

**Files to Modify**:

- `apps/web/src/engine/engineClient.ts`
- `apps/web/src/App.tsx` (remove disabled attribute from remote option)

---

### Session 4: Playable Chess UI - Interactive Board + Game Loop 🟡

**Duration**: 3-4 hours

**Objective**: Build interactive chess board where user can play full games against engine

**Implementation**:

1. **Install Chess UI Libraries**

   ```bash
   pnpm add chess.js react-chessboard
   ```

   - `chess.js`: Game state, move validation, check/checkmate detection
   - `react-chessboard`: Visual chess board component

2. **Create `GameBoard` Component**

   ```typescript
   // apps/web/src/components/GameBoard.tsx
   export function GameBoard() {
     const [game, setGame] = useState(new Chess());
     const [position, setPosition] = useState(game.fen());
     const [userColor, setUserColor] = useState<'white' | 'black'>('white');
     const [engineThinking, setEngineThinking] = useState(false);
     const [engineDepth, setEngineDepth] = useState(10);

     function onDrop(sourceSquare: string, targetSquare: string) {
       const move = game.move({
         from: sourceSquare,
         to: targetSquare,
         promotion: 'q', // Always promote to queen for simplicity
       });

       if (move === null) return false; // Illegal move

       setPosition(game.fen());

       // If game not over, request engine move
       if (!game.isGameOver()) {
         getEngineMove(game.fen());
       }

       return true;
     }

     async function getEngineMove(fen: string) {
       setEngineThinking(true);

       try {
         // Use RemoteEngine.analyze() to get best move
         // (same as analysis UI, just ignores SearchInfo updates)
         const bestMove = await new Promise<string>((resolve) => {
           const engine = new RemoteEngine();
           engine.analyze(
             { id: crypto.randomUUID(), fen, limit: { kind: 'depth', depth: engineDepth } },
             (evt) => {
               if (evt.type === 'bestMove') {
                 resolve(evt.payload.best);
               }
             }
           );
         });

         // Apply engine move to board
         const move = game.move({
           from: bestMove.substring(0, 2),
           to: bestMove.substring(2, 4),
           promotion: bestMove.length > 4 ? bestMove[4] : undefined,
         });

         setPosition(game.fen());
         setEngineThinking(false);

         // Check game over
         if (game.isGameOver()) {
           showGameOverDialog();
         }
       } catch (error) {
         console.error('Engine move failed:', error);
         setEngineThinking(false);
       }
     }

     return (
       <div>
         <Chessboard
           position={position}
           onPieceDrop={onDrop}
           boardOrientation={userColor}
         />
         <div>Status: {getGameStatus()}</div>
         {engineThinking && <div>Engine thinking...</div>}
       </div>
     );
   }
   ```

3. **Game State Management**
   - Track current position (FEN)
   - Detect check, checkmate, stalemate, draw
   - Handle promotion (default to queen, or show dialog)
   - Show game status (White to move, Black in check, etc.)
   - Move history display

4. **Game Flow**
   - User makes move → Update board → Send FEN to engine
   - Engine returns move → Apply to board → Update UI
   - Repeat until game over (checkmate/stalemate/draw)

5. **Game Controls**
   - New Game button
   - Choose color (play as White/Black)
   - Choose engine strength (depth: 1-20)
   - Resign button
   - Flip board button
   - Show move history

6. **Add Game Mode to App**

   ```typescript
   // apps/web/src/App.tsx
   const [viewMode, setViewMode] = useState<'analysis' | 'game'>('analysis');

   return (
     <div>
       <button onClick={() => setViewMode('analysis')}>Analysis</button>
       <button onClick={() => setViewMode('game')}>Play</button>

       {viewMode === 'analysis' ? <AnalysisView /> : <GameBoard />}
     </div>
   );
   ```

**Success Criteria**:

- ✅ Chess board renders correctly
- ✅ User can drag pieces to make legal moves
- ✅ Illegal moves are rejected
- ✅ Engine responds automatically after user move
- ✅ Full game playable from start to checkmate
- ✅ Engine strength configurable
- ✅ Game over detected (checkmate, stalemate, draw)
- ✅ Can start new game and play again

**Files to Create**:

- `apps/web/src/components/GameBoard.tsx`
- `apps/web/src/components/MoveHistory.tsx`

**Files to Modify**:

- `apps/web/src/App.tsx` (add game mode toggle)

---

### Session 5: Polish & Production Deploy 🟢

**Duration**: 2-3 hours

**Objective**: Production-ready deployment with monitoring and error handling

**Implementation**:

1. **Server Production Config**
   - Environment variables for configuration
   - CORS handling for web clients
   - Rate limiting (prevent abuse)
   - Connection timeouts
   - Graceful shutdown

2. **Error Handling**
   - Invalid FEN → Return error message
   - Engine crash → Restart process
   - WebSocket disconnect → Clean up resources
   - Timeout on long searches

3. **Logging & Monitoring**
   - Structured logging with `tracing`
   - Log all connections, requests, errors
   - Performance metrics (search time, nodes/sec)
   - Health check endpoint

4. **UI Polish**
   - Responsive design (mobile-friendly)
   - Loading states and error messages
   - Sound effects (move, capture, check)
   - Piece animations
   - Dark mode

5. **Deployment**
   - Docker container for server
   - Environment-specific configs (dev/prod)
   - Deploy server to cloud (fly.io, Railway, etc.)
   - Deploy web UI to Vercel/Netlify

6. **Documentation**
   - README with setup instructions
   - API documentation (WebSocket protocol)
   - User guide (how to play)

**Success Criteria**:

- ✅ Server runs in production environment
- ✅ Graceful error handling (no crashes)
- ✅ Logging and monitoring in place
- ✅ UI works on desktop and mobile
- ✅ Documentation complete
- ✅ Can play full games reliably

**Files to Create**:

- `apps/uci-server/Dockerfile`
- `apps/uci-server/.env.example`
- `README.md` (updated)
- `docs/API.md` (WebSocket protocol docs)

---

## Dependencies & Prerequisites

### Rust Dependencies

```toml
# apps/uci-server/Cargo.toml
[dependencies]
tokio = { version = "1", features = ["full"] }
tokio-tungstenite = "0.21"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
engine = { path = "../../crates/engine" }
protocol = { path = "../../crates/protocol" }
anyhow = "1"
tracing = "0.1"
tracing-subscriber = "0.3"
futures = "0.3"
```

### TypeScript Dependencies

```json
// apps/web/package.json
{
  "dependencies": {
    "chess.js": "^1.0.0-beta.6",
    "react-chessboard": "^4.3.0"
  }
}
```

---

## Testing Strategy

### Unit Tests

- ✅ Message parsing (JSON ↔ Protocol types)
- ✅ Engine integration (getBestMove, analyze, stop)
- ✅ Game state management (chess.js integration)

### Integration Tests

- ✅ WebSocket end-to-end (send getBestMove, receive response)
- ✅ Multiple concurrent connections
- ✅ Game flow (full game from start to checkmate)

### Manual Testing

- ✅ Play 5+ full games against engine
- ✅ Test on Chrome, Firefox, Safari
- ✅ Test on mobile (iOS, Android)
- ✅ Test with slow network (throttling)
- ✅ Test with engine at depth 1, 10, 20

---

## Success Metrics

### Production Plan Complete When:

1. ✅ **WebSocket server running (Requirement #1)**
   - Handles 10+ concurrent connections
   - <100ms latency for bestMove requests
   - No memory leaks or crashes
   - Deployed and accessible

2. ✅ **Engine analyzes FEN via server (Requirement #2)**
   - Engine works (M9 complete) ✅
   - WebSocket server exposes analyze API ✅
   - **Existing analysis UI wired to server** ← Session 3
   - "Remote" mode functional in current UI
   - Can analyze positions via WebSocket

3. ✅ **Playable Chess UI (Requirement #3)**
   - Interactive chess board
   - User can play full games
   - Engine responds to user moves
   - Game over detection works
   - Works on desktop and mobile

---

## Timeline Estimate

| Session   | Focus                               | Duration        |
| --------- | ----------------------------------- | --------------- |
| Session 1 | WebSocket server core               | 3-4 hours       |
| Session 2 | Engine integration (backend)        | 2-3 hours       |
| Session 3 | Wire existing analysis UI to server | 2-3 hours       |
| Session 4 | Playable chess UI                   | 3-4 hours       |
| Session 5 | Polish & deploy                     | 2-3 hours       |
| **Total** |                                     | **12-17 hours** |

---

## Risk Mitigation

### High Risk

1. **WebSocket stability** - May have connection drops
   - _Mitigation_: Auto-reconnect logic, connection heartbeat

2. **Engine performance** - May be too slow on server
   - _Mitigation_: Configurable depth, time limits

### Medium Risk

3. **Concurrent load** - Server may not handle many users
   - _Mitigation_: Connection limit, rate limiting

4. **Move parsing edge cases** - UCI notation tricky
   - _Mitigation_: Use chess.js for all move validation

### Low Risk

5. **UI responsiveness** - Board may lag on slow devices
   - _Mitigation_: Test on mobile, optimize rendering

---

## Out of Scope (Future Work)

- ❌ WASM mode (mate detection bug - defer to later)
- ❌ Opening book integration
- ❌ Endgame tablebases
- ❌ Analysis mode (multi-PV, infinite analysis)
- ❌ PGN import/export
- ❌ User accounts / saved games
- ❌ Multiplayer (human vs human)
- ❌ Engine tournaments

---

## Alternative Approaches Considered

### For Server:

1. **HTTP polling** - Simpler but higher latency
2. **gRPC** - More efficient but browser support limited
3. **Server-Sent Events** - Unidirectional only

**Decision**: WebSockets for bidirectional, low-latency communication

### For UI:

1. **Custom canvas rendering** - Full control but complex
2. **Use chessboard.js** - Older, jQuery-based
3. **Use react-chessboard** - Modern, React-native

**Decision**: react-chessboard for simplicity and React integration

---

## Current State → End State

### Before (Current)

```
[Current State]
- ✅ Native engine works (M9 complete)
- ✅ Analysis UI exists (paste FEN, see results)
- ❌ No WebSocket server
- ❌ "Remote" mode in UI not implemented
- ❌ No playable game UI
- ⚠️  WASM broken (disabled)
```

### After (Production Ready)

```
[End State]
1. ✅ WebSocket server deployed and running
2. ✅ Existing analysis UI wired to server (remote mode works)
3. ✅ Playable chess UI (user vs engine games)
4. ✅ Production-grade error handling
5. ✅ Documentation complete
```

---

**Created**: 2025-10-26
**Status**: Ready to Execute
**Next Step**: Session 1 - WebSocket Server Core
