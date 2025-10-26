# Web Workers

## Overview

This directory contains Web Worker implementations for running the chess engine off the main thread. This prevents the engine's computationally intensive search from blocking the UI.

## Architecture

```
Main Thread (UI)                 Worker Thread (Engine)
┌─────────────────┐             ┌─────────────────────┐
│                 │             │                     │
│  engineClient   │────────────▶│  engine.worker     │
│                 │  messages   │                     │
│                 │◀────────────│  WASM Engine       │
│                 │  events     │                     │
└─────────────────┘             └─────────────────────┘
```

## engine.worker.ts

The main Web Worker that:

1. **Initializes WASM**: Loads and initializes the chess engine WASM module
2. **Manages Engine**: Creates and manages the `WasmEngine` instance
3. **Handles Requests**: Processes analyze and stop requests
4. **Streams Results**: Sends search info and best move events back to main thread

### Message Protocol

**From Main Thread → Worker:**

```typescript
{ type: 'init', wasmPath?: string }
{ type: 'analyze', payload: AnalyzeRequest }
{ type: 'stop', id: string }
{ type: 'ping' }
```

**From Worker → Main Thread:**

```typescript
{ type: 'initialized' }
{ type: 'searchInfo', payload: SearchInfo }
{ type: 'bestMove', payload: BestMove }
{ type: 'error', payload: { id: string, message: string } }
{ type: 'pong' }
```

## Usage Example

```typescript
// Create worker
const worker = new Worker(new URL('./engine.worker.ts', import.meta.url), {
  type: 'module',
});

// Initialize
worker.postMessage({ type: 'init' });

// Wait for initialization
worker.onmessage = (ev) => {
  if (ev.data.type === 'initialized') {
    // Start analysis
    worker.postMessage({
      type: 'analyze',
      payload: {
        id: 'game-123',
        fen: 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1',
        limit: { kind: 'depth', depth: 10 },
      },
    });
  }

  // Handle search info
  if (ev.data.type === 'searchInfo') {
    console.log('Depth:', ev.data.payload.depth);
    console.log('Score:', ev.data.payload.score);
    console.log('PV:', ev.data.payload.pv);
  }

  // Handle best move
  if (ev.data.type === 'bestMove') {
    console.log('Best move:', ev.data.payload.best);
  }
};
```

## Error Handling

The worker includes comprehensive error handling:

- **Initialization Errors**: Sent if WASM module fails to load
- **Analysis Errors**: Sent if engine crashes during search
- **Worker Errors**: Global error handler catches unhandled exceptions

All errors are sent as `EngineEvent` with type `'error'`.

## Performance

Running the engine in a Web Worker:

- **Non-blocking UI**: Engine search doesn't freeze the browser
- **Multi-threading**: Worker runs on separate thread (subject to browser thread pool)
- **Memory Isolation**: Worker has separate memory space
- **Transfer Overhead**: Minimal - only small messages passed between threads

## Development

### Testing Worker

```typescript
// Ping test
worker.postMessage({ type: 'ping' });
worker.onmessage = (ev) => {
  if (ev.data.type === 'pong') {
    console.log('✓ Worker is responsive');
  }
};
```

### Debugging

Workers can be debugged in browser DevTools:

1. Open DevTools → Sources
2. Find worker under "Threads"
3. Set breakpoints in worker code
4. Use `console.log()` in worker (appears in main console)

## Browser Compatibility

Web Workers are supported in all modern browsers:

- Chrome/Edge: ✅
- Firefox: ✅
- Safari: ✅ (iOS 10+)
- Opera: ✅

### Module Workers

The worker uses `type: 'module'` for ESM support:

- Chrome/Edge: ✅ (v80+)
- Firefox: ✅ (v114+)
- Safari: ✅ (v15+)

For older browsers, consider using a bundler like Vite/Webpack to transform the worker.
