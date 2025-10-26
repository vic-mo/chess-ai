/**
 * Chess Engine Web Worker
 *
 * Runs the WASM chess engine in a separate thread to avoid blocking the UI.
 * Handles engine initialization, position setup, and search execution.
 */

/// <reference lib="webworker" />

import type { AnalyzeRequest, EngineEvent } from '@chess-ai/protocol';

// WASM module and engine instance
// eslint-disable-next-line @typescript-eslint/no-explicit-any
let wasmEngine: any = null;
let wasmInitialized = false;

/**
 * Worker message types
 */
type WorkerMessage =
  | { type: 'init'; wasmPath?: string }
  | { type: 'analyze'; payload: AnalyzeRequest }
  | { type: 'stop'; id: string }
  | { type: 'ping' };

/**
 * Initialize the WASM engine
 */
async function initWasm(wasmPath?: string): Promise<void> {
  if (wasmInitialized) {
    self.postMessage({ type: 'initialized' });
    return;
  }

  try {
    // Import WASM module from public directory using importScripts
    const basePath = wasmPath || '/wasm/engine_bridge_wasm.js';

    // Use importScripts to load the WASM glue code
    // This is the proper way to load external scripts in Web Workers
    importScripts(basePath);

    // The WASM module is now available in global scope
    // TypeScript doesn't know about the dynamically loaded module, so we use any
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const initWasm = (self as any).default;
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const WasmEngine = (self as any).WasmEngine;

    if (!initWasm || !WasmEngine) {
      throw new Error('WASM module did not export expected functions');
    }

    // Initialize the WASM module
    // Pass the WASM file path explicitly
    await initWasm('/wasm/engine_bridge_wasm_bg.wasm');

    // Create engine instance with default options
    wasmEngine = new WasmEngine({
      hashSizeMB: 128,
      threads: 1,
      contempt: 0,
      skillLevel: 20,
      multiPV: 1,
      useTablebases: false,
    });

    wasmInitialized = true;

    // Send success message back to main thread
    self.postMessage({ type: 'initialized' });
  } catch (error) {
    // Send error message back to main thread
    const message = error instanceof Error ? error.message : String(error);
    self.postMessage({
      type: 'error',
      payload: { id: 'init', message: `Failed to initialize WASM: ${message}` },
    });
  }
}

/**
 * Handle analyze request
 */
function handleAnalyze(req: AnalyzeRequest): void {
  if (!wasmEngine) {
    self.postMessage({
      type: 'error',
      payload: { id: req.id, message: 'Engine not initialized' },
    });
    return;
  }

  try {
    // Set position
    wasmEngine.position(req.fen, req.moves || []);

    // Convert search limit to WASM format
    let limit: unknown;
    switch (req.limit.kind) {
      case 'depth':
        limit = { depth: req.limit.depth };
        break;
      case 'nodes':
        limit = { nodes: req.limit.nodes };
        break;
      case 'time':
        limit = { moveTimeMs: req.limit.moveTimeMs };
        break;
      case 'infinite':
        limit = { infinite: true };
        break;
    }

    // Callback for search info
    const callback = (info: unknown) => {
      self.postMessage({
        type: 'searchInfo',
        payload: info,
      });
    };

    // Start analysis
    const result = wasmEngine.analyze(limit, callback);

    // Send best move
    const bestMoveEvent: EngineEvent = {
      type: 'bestMove',
      payload: result,
    };
    self.postMessage(bestMoveEvent);
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    self.postMessage({
      type: 'error',
      payload: { id: req.id, message },
    });
  }
}

/**
 * Handle stop request
 */
function handleStop(): void {
  if (wasmEngine) {
    try {
      wasmEngine.stop();
    } catch (error) {
      // Ignore stop errors
      console.warn('Error stopping engine:', error);
    }
  }
}

/**
 * Message handler
 */
self.onmessage = (ev: MessageEvent<WorkerMessage>) => {
  const msg = ev.data;

  switch (msg.type) {
    case 'init':
      void initWasm(msg.wasmPath);
      break;

    case 'analyze':
      handleAnalyze(msg.payload);
      break;

    case 'stop':
      handleStop();
      break;

    case 'ping':
      self.postMessage({ type: 'pong' });
      break;

    default:
      console.warn('Unknown message type:', msg);
  }
};

/**
 * Error handler
 */
self.onerror = (error: ErrorEvent) => {
  console.error('Worker error:', error);
  self.postMessage({
    type: 'error',
    payload: { id: 'worker', message: error.message || String(error) },
  });
};
