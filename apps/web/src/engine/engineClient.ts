import type { AnalyzeRequest, EngineEvent, SearchInfo, BestMove } from '@chess-ai/protocol';
import { Schema } from '@chess-ai/protocol';

type EngineMode = 'fake' | 'remote' | 'wasm';
interface EngineGlobals {
  __ENGINE_MODE__?: EngineMode;
}

type EventHandler = (evt: EngineEvent) => void;
type StopHandler = () => void;

function fakeAnalyze(req: AnalyzeRequest, onEvent: EventHandler): StopHandler {
  let depth = 0;
  const maxDepth = req.limit.kind === 'depth' ? req.limit.depth : 6;
  const id = req.id;
  const timer = setInterval(() => {
    depth++;
    if (depth <= maxDepth) {
      const payload: SearchInfo = {
        id,
        depth,
        nodes: depth * 10_000,
        nps: 1_000_000,
        timeMs: depth * 70,
        score: { kind: 'cp', value: depth * 10 },
        pv: ['e2e4', 'e7e5'],
        seldepth: depth + 1,
        hashfull: depth * 10,
      };
      onEvent({ type: 'searchInfo', payload });
    } else {
      clearInterval(timer);
      const bestMove: BestMove = { id, best: 'e2e4', ponder: 'e7e5' };
      onEvent({ type: 'bestMove', payload: bestMove });
    }
  }, 120);

  return () => clearInterval(timer);
}

let remoteWs: WebSocket | null = null;
let remoteLastId: string | null = null;

function remoteAnalyze(req: AnalyzeRequest, onEvent: EventHandler): StopHandler {
  const base = import.meta.env.VITE_ENGINE_SERVER_URL || 'http://127.0.0.1:8080';
  remoteLastId = req.id;
  const cleanup: StopHandler = () => {
    if (remoteWs) {
      remoteWs.close();
      remoteWs = null;
    }
  };
  fetch(base + '/analyze', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ id: req.id, fen: req.fen, limit: req.limit }),
  })
    .then((res) => res.json())
    .then(({ id }: { id: string }) => {
      const ws = new WebSocket(base.replace(/^http/, 'ws') + '/streams/' + id);
      ws.onmessage = (ev) => {
        try {
          const parsed: unknown = JSON.parse(ev.data);
          const result = Schema.EngineEvent.safeParse(parsed);
          if (result.success) {
            onEvent(result.data);
          }
        } catch (e) {
          /* ignore */
        }
      };
      remoteWs = ws;
      ws.onclose = () => {
        if (remoteWs === ws) {
          remoteWs = null;
        }
      };
    });
  return cleanup;
}

// Web Worker instance for WASM engine
let wasmWorker: Worker | null = null;
let wasmInitialized = false;
const wasmEventHandlers = new Map<string, EventHandler>();

/**
 * Initialize WASM worker
 */
function initWasmWorker(): Promise<void> {
  if (wasmInitialized) {
    return Promise.resolve();
  }

  return new Promise((resolve, reject) => {
    try {
      // Create worker
      wasmWorker = new Worker(new URL('../workers/engine.worker.ts', import.meta.url), {
        type: 'module',
      });

      // Set up message handler
      wasmWorker.onmessage = (ev: MessageEvent) => {
        const msg = ev.data;

        // Handle initialization
        if (msg.type === 'initialized') {
          wasmInitialized = true;
          resolve();
          return;
        }

        // Handle engine events
        if (msg.type === 'searchInfo' || msg.type === 'bestMove' || msg.type === 'error') {
          const payload = msg.payload as { id: string };
          const handler = wasmEventHandlers.get(payload.id);
          if (handler) {
            handler(msg as EngineEvent);

            // Clean up handler on bestMove or error
            if (msg.type === 'bestMove' || msg.type === 'error') {
              wasmEventHandlers.delete(payload.id);
            }
          }
        }
      };

      // Set up error handler
      wasmWorker.onerror = (error) => {
        console.error('WASM worker error:', error);
        reject(error);
      };

      // Send init message
      wasmWorker.postMessage({ type: 'init' });
    } catch (error) {
      reject(error);
    }
  });
}

function wasmAnalyze(req: AnalyzeRequest, onEvent: EventHandler): StopHandler {
  // Register event handler
  wasmEventHandlers.set(req.id, onEvent);

  // Initialize worker if needed
  if (!wasmInitialized) {
    initWasmWorker()
      .then(() => {
        // Send analyze request
        wasmWorker?.postMessage({ type: 'analyze', payload: req });
      })
      .catch((error) => {
        // Fall back to fake on error
        console.error('Failed to initialize WASM worker, falling back to fake:', error);
        wasmEventHandlers.delete(req.id);
        return fakeAnalyze(req, onEvent);
      });
  } else {
    // Send analyze request
    wasmWorker?.postMessage({ type: 'analyze', payload: req });
  }

  // Return stop handler
  return () => {
    wasmWorker?.postMessage({ type: 'stop', id: req.id });
    wasmEventHandlers.delete(req.id);
  };
}

function stopRemote(id: string) {
  const base = import.meta.env.VITE_ENGINE_SERVER_URL || 'http://127.0.0.1:8080';
  fetch(base + '/stop', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ id: remoteLastId ?? id }),
  });
  if (remoteWs) {
    remoteWs.close();
    remoteWs = null;
  }
}

export function getEngineMode(): EngineMode {
  return (globalThis as EngineGlobals).__ENGINE_MODE__ ?? 'fake';
}

export function setEngineMode(mode: EngineMode): void {
  (globalThis as EngineGlobals).__ENGINE_MODE__ = mode;

  // Reset WASM worker when switching modes
  if (mode !== 'wasm' && wasmWorker) {
    wasmWorker.terminate();
    wasmWorker = null;
    wasmInitialized = false;
    wasmEventHandlers.clear();
  }

  // Reset remote WebSocket when switching modes
  if (mode !== 'remote' && remoteWs) {
    remoteWs.close();
    remoteWs = null;
  }
}

/**
 * Get WASM engine status
 */
export function getWasmStatus(): 'uninitialized' | 'initializing' | 'ready' | 'error' {
  if (wasmWorker === null) return 'uninitialized';
  if (wasmInitialized) return 'ready';
  return 'initializing';
}

/**
 * Preload WASM engine (optional, for better UX)
 */
export function preloadWasm(): Promise<void> {
  if (getEngineMode() !== 'wasm') {
    return Promise.reject(new Error('Engine mode must be "wasm" to preload'));
  }
  return initWasmWorker();
}

export function useEngine() {
  const mode = getEngineMode();
  return {
    analyze(req: AnalyzeRequest, onEvent: EventHandler) {
      if (mode === 'remote') return remoteAnalyze(req, onEvent);
      if (mode === 'wasm') return wasmAnalyze(req, onEvent);
      return fakeAnalyze(req, onEvent);
    },
    stop(id: string) {
      if (mode === 'remote') stopRemote(id);
      if (mode === 'wasm') {
        wasmWorker?.postMessage({ type: 'stop', id });
      }
      // fake mode: no-op
    },
  };
}
