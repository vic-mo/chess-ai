# Engine Client

## Overview

The engine client provides a unified interface for interacting with the chess engine in different modes. It supports three backend modes:

1. **Fake**: Simulated engine for testing and demos
2. **WASM**: Local browser-based engine (runs in Web Worker)
3. **Remote**: Server-based engine via WebSocket

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                   React Application                 │
│                                                     │
│  ┌─────────────────────────────────────────────┐  │
│  │           engineClient.ts                    │  │
│  │                                              │  │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  │  │
│  │  │   Fake   │  │   WASM   │  │  Remote  │  │  │
│  │  │  Engine  │  │  Engine  │  │  Engine  │  │  │
│  │  └──────────┘  └──────────┘  └──────────┘  │  │
│  │                     │                        │  │
│  └─────────────────────┼────────────────────────┘  │
│                        │                           │
└────────────────────────┼───────────────────────────┘
                         │
                         ▼
                  ┌──────────────┐
                  │  Web Worker  │
                  │              │
                  │  WASM Engine │
                  └──────────────┘
```

## Usage

### Basic Usage

```typescript
import { useEngine, setEngineMode } from './engine/engineClient';

function MyComponent() {
  const engine = useEngine();

  // Set engine mode
  setEngineMode('wasm'); // 'fake' | 'wasm' | 'remote'

  // Analyze a position
  const stopHandler = engine.analyze(
    {
      id: 'game-123',
      fen: 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1',
      limit: { kind: 'depth', depth: 10 },
    },
    (event) => {
      if (event.type === 'searchInfo') {
        console.log('Depth:', event.payload.depth);
        console.log('Score:', event.payload.score);
        console.log('PV:', event.payload.pv);
      } else if (event.type === 'bestMove') {
        console.log('Best move:', event.payload.best);
      } else if (event.type === 'error') {
        console.error('Error:', event.payload.message);
      }
    },
  );

  // Stop analysis
  stopHandler(); // or engine.stop('game-123')
}
```

### Mode Management

```typescript
import { getEngineMode, setEngineMode, getWasmStatus } from './engine/engineClient';

// Get current mode
const mode = getEngineMode(); // 'fake' | 'wasm' | 'remote'

// Switch modes
setEngineMode('wasm');

// Check WASM status
const status = getWasmStatus(); // 'uninitialized' | 'initializing' | 'ready' | 'error'
```

### Preloading WASM

For better UX, you can preload the WASM engine before the user starts analysis:

```typescript
import { setEngineMode, preloadWasm } from './engine/engineClient';

// Switch to WASM mode
setEngineMode('wasm');

// Preload the engine
preloadWasm()
  .then(() => console.log('WASM engine ready'))
  .catch((error) => console.error('Failed to load WASM:', error));
```

## Engine Modes

### Fake Mode

- **Purpose**: Testing and demos
- **Characteristics**:
  - No actual chess calculation
  - Returns simulated search info
  - Instant responses
  - Always suggests e2e4
- **Use Cases**: Development, UI testing, demos

### WASM Mode

- **Purpose**: Production use in browser
- **Characteristics**:
  - Full chess engine in browser
  - Runs in Web Worker (non-blocking)
  - ~95 KB WASM binary (40 KB gzipped)
  - Local computation (privacy, offline)
- **Use Cases**: Production, offline play, privacy-sensitive scenarios

### Remote Mode

- **Purpose**: Server-side computation
- **Characteristics**:
  - Engine runs on server
  - Communicates via WebSocket
  - Lower client resource usage
  - Requires network connection
- **Use Cases**: Mobile devices, resource-constrained clients, cloud deployment

## API Reference

### `useEngine()`

Returns an engine interface with the following methods:

#### `analyze(request, onEvent)`

Start analyzing a position.

**Parameters:**

- `request: AnalyzeRequest` - Analysis request
  ```typescript
  {
    id: string;           // Unique request ID
    fen: string;          // Position FEN
    moves?: string[];     // Move list (UCI format)
    limit: SearchLimit;   // Search limit
  }
  ```
- `onEvent: (event: EngineEvent) => void` - Event callback

**Returns:** `StopHandler` - Function to stop analysis

**Events:**

- `searchInfo` - Search progress update
- `bestMove` - Final result
- `error` - Error notification

#### `stop(id)`

Stop an ongoing analysis.

**Parameters:**

- `id: string` - Request ID to stop

### `getEngineMode()`

Get the current engine mode.

**Returns:** `'fake' | 'wasm' | 'remote'`

### `setEngineMode(mode)`

Switch engine mode.

**Parameters:**

- `mode: 'fake' | 'wasm' | 'remote'` - New mode

**Side Effects:**

- Terminates WASM worker when switching away from WASM
- Closes remote WebSocket when switching away from remote
- Clears pending event handlers

### `getWasmStatus()`

Get WASM engine initialization status.

**Returns:** `'uninitialized' | 'initializing' | 'ready' | 'error'`

### `preloadWasm()`

Preload WASM engine (must be in WASM mode).

**Returns:** `Promise<void>`

**Throws:** Error if not in WASM mode

## Error Handling

The engine client includes comprehensive error handling:

### WASM Errors

- **Initialization Failure**: Falls back to fake engine
- **Worker Crash**: Sends error event to callback
- **Import Errors**: Logged to console, fallback to fake

### Remote Errors

- **Connection Failure**: No fallback (user should switch modes)
- **Server Errors**: Propagated via error events
- **WebSocket Closure**: Cleanup handlers called

### Error Event Format

```typescript
{
  type: 'error',
  payload: {
    id: string,      // Request ID
    message: string  // Error description
  }
}
```

## Performance Considerations

### WASM Mode

- **First Load**: ~40 KB download + initialization (~50-100ms)
- **Subsequent Loads**: Cached by browser
- **Memory**: ~10-50 MB depending on hash table size
- **CPU**: Full engine search (NPS ~10k on M1 Mac)

### Remote Mode

- **Latency**: Depends on network + server load
- **Bandwidth**: Minimal (only search info updates)
- **Memory**: Very low (just UI state)

### Fake Mode

- **Overhead**: Negligible (just timers)
- **Memory**: Minimal

## Testing

Unit tests are located in `engineClient.test.ts`:

```bash
pnpm test
```

**Test Coverage:**

- Mode switching
- WASM status tracking
- Cleanup on mode change
- Error handling

## Browser Compatibility

| Feature     | Chrome | Firefox | Safari | Edge   |
| ----------- | ------ | ------- | ------ | ------ |
| Fake Mode   | ✅     | ✅      | ✅     | ✅     |
| Remote Mode | ✅     | ✅      | ✅     | ✅     |
| WASM Mode   | ✅ 80+ | ✅ 114+ | ✅ 15+ | ✅ 80+ |

**Notes:**

- WASM mode requires module worker support
- Older browsers may need polyfills
- See `src/workers/README.md` for worker compatibility details

## Troubleshooting

### WASM not loading

1. Check browser console for errors
2. Verify WASM files exist in `/wasm/` directory
3. Run `pnpm build:wasm` to rebuild
4. Check browser compatibility

### Worker errors

1. Check that worker is being loaded as module
2. Verify `engine.worker.ts` has no syntax errors
3. Check WASM path is correct (`/wasm/engine_bridge_wasm.js`)

### Mode switching issues

1. Ensure `setEngineMode()` is called before `analyze()`
2. Check that mode is persisted in global state
3. Verify cleanup handlers are running

## Future Improvements

- Persistent engine state across mode switches
- Multiple concurrent analyses
- Engine options configuration (hash size, threads, etc.)
- SharedArrayBuffer support for better WASM performance
- Engine benchmarking and diagnostics
