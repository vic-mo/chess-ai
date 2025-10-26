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
    console.log('[Worker] Starting WASM initialization...');

    // Load WASM module as ES module using dynamic import with base URL
    const basePath = wasmPath || '/wasm/engine_bridge_wasm.js';
    const baseUrl = self.location.origin;
    const fullUrl = new URL(basePath, baseUrl).href;

    console.log('[Worker] Loading WASM from:', fullUrl);

    // Dynamic import of the ES module
    const wasmModule = await import(/* @vite-ignore */ fullUrl);

    console.log('[Worker] WASM module loaded, initializing...');

    // Initialize with explicit WASM binary path
    const wasmBinaryPath = new URL('/wasm/engine_bridge_wasm_bg.wasm', baseUrl).href;
    await wasmModule.default(wasmBinaryPath);

    console.log('[Worker] WASM initialized, creating engine instance...');

    // Create engine instance
    wasmEngine = new wasmModule.WasmEngine({
      hashSizeMB: 128,
      threads: 1,
      contempt: 0,
      skillLevel: 20,
      multiPV: 1,
      useTablebases: false,
    });

    wasmInitialized = true;

    console.log('[Worker] Engine instance created successfully');

    // Send success message back to main thread
    self.postMessage({ type: 'initialized' });
  } catch (error) {
    // Send error message back to main thread
    const message = error instanceof Error ? error.message : String(error);
    console.error('[Worker] WASM initialization failed:', error);
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

    // Search limit already has the correct format with 'kind' field
    // Just pass it through as-is
    const limit = req.limit;

    // Callback for search info
    const callback = (info: unknown) => {
      self.postMessage({
        type: 'searchInfo',
        payload: info,
      });
    };

    console.log('[Worker] Starting search with limit:', limit);

    // Start analysis
    const result = wasmEngine.analyze(limit, callback);

    console.log('[Worker] Search completed, result:', result);

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
