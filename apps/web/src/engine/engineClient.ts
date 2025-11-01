import type { AnalyzeRequest, EngineEvent, SearchInfo, BestMove } from '@chess-ai/protocol';
import { Schema } from '@chess-ai/protocol';
import { performanceMonitor } from '../utils/performance';
import { logger } from '../utils/logger';

type EngineMode = 'remote' | 'wasm';
interface EngineGlobals {
  __ENGINE_MODE__?: EngineMode;
}

type EventHandler = (evt: EngineEvent) => void;
type StopHandler = () => void;

function fakeAnalyze(req: AnalyzeRequest, onEvent: EventHandler): StopHandler {
  // Start search performance monitoring
  performanceMonitor.startSearch();

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
      // End search performance monitoring
      performanceMonitor.endSearch();
    }
  }, 120);

  return () => {
    clearInterval(timer);
    performanceMonitor.endSearch();
  };
}

let remoteWs: WebSocket | null = null;
let remoteLastId: string | null = null;

function remoteAnalyze(req: AnalyzeRequest, onEvent: EventHandler): StopHandler {
  const base = import.meta.env.VITE_ENGINE_SERVER_URL || 'ws://127.0.0.1:8080';
  const wsUrl = base.replace(/^http/, 'ws');
  remoteLastId = req.id;

  const ws = new WebSocket(wsUrl);
  let requestSent = false;
  let bestMoveSent = false;

  ws.onopen = () => {
    // Send analyze request over WebSocket
    ws.send(
      JSON.stringify({
        type: 'analyze',
        id: req.id,
        fen: req.fen,
        limit: req.limit,
      }),
    );
    requestSent = true;
  };

  ws.onmessage = (ev) => {
    try {
      logger.log('[RemoteAnalyze] Raw message:', ev.data);
      const parsed: unknown = JSON.parse(ev.data);
      logger.log('[RemoteAnalyze] Parsed message:', parsed);
      const result = Schema.EngineEvent.safeParse(parsed);
      logger.log('[RemoteAnalyze] Schema validation:', result);
      if (result.success) {
        logger.log('[RemoteAnalyze] Calling onEvent with:', result.data);
        if (result.data.type === 'bestMove') {
          bestMoveSent = true;
        }
        onEvent(result.data);
      } else {
        logger.error('[RemoteAnalyze] Schema validation failed:', result.error);
      }
    } catch (e) {
      logger.error('Failed to parse WebSocket message:', e);
    }
  };

  ws.onerror = (error) => {
    logger.error('WebSocket error:', error);
    if (!bestMoveSent) {
      onEvent({
        type: 'error',
        payload: { id: req.id, message: 'WebSocket connection failed' },
      });
    }
  };

  ws.onclose = (event) => {
    // If connection closes before sending request or receiving bestMove, report error
    if (!requestSent || !bestMoveSent) {
      logger.error('WebSocket closed prematurely:', event.code, event.reason);
      if (!bestMoveSent) {
        onEvent({
          type: 'error',
          payload: {
            id: req.id,
            message: `WebSocket closed before completion: ${event.reason || 'Unknown reason'}`,
          },
        });
      }
    }
    if (remoteWs === ws) {
      remoteWs = null;
    }
  };

  remoteWs = ws;

  const cleanup: StopHandler = () => {
    if (remoteWs === ws) {
      ws.close();
      remoteWs = null;
    }
  };

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
      // Start performance monitoring
      performanceMonitor.startWorkerCreate();
      performanceMonitor.startWasmLoad();

      // Create worker
      wasmWorker = new Worker(new URL('../workers/engine.worker.ts', import.meta.url), {
        type: 'module',
      });

      performanceMonitor.endWorkerCreate();

      // Set up message handler
      wasmWorker.onmessage = (ev: MessageEvent) => {
        const msg = ev.data;
        logger.debug('[EngineClient] Received message from worker:', msg);

        // Handle initialization
        if (msg.type === 'initialized') {
          wasmInitialized = true;
          performanceMonitor.endWasmLoad();
          logger.debug('[Performance] WASM initialized:', performanceMonitor.getMetrics());
          resolve();
          return;
        }

        // Handle engine events
        if (msg.type === 'searchInfo' || msg.type === 'bestMove' || msg.type === 'error') {
          const payload = msg.payload as { id: string };
          logger.debug('[EngineClient] Event type:', msg.type, 'ID:', payload?.id);

          const handler = wasmEventHandlers.get(payload.id);
          logger.debug(
            '[EngineClient] Handler found:',
            !!handler,
            'Active handlers:',
            Array.from(wasmEventHandlers.keys()),
          );

          if (handler) {
            logger.debug('[EngineClient] Calling handler with event');
            handler(msg as EngineEvent);

            // Clean up handler on bestMove or error
            if (msg.type === 'bestMove' || msg.type === 'error') {
              wasmEventHandlers.delete(payload.id);
              performanceMonitor.endSearch();
              logger.debug('[EngineClient] Handler cleaned up');
            }
          } else {
            logger.warn('[EngineClient] No handler for ID:', payload?.id);
          }
        }
      };

      // Set up error handler
      wasmWorker.onerror = (error) => {
        logger.error('WASM worker error:', error);

        // If not initialized yet, reject initialization
        if (!wasmInitialized) {
          reject(error);
        } else {
          // Worker crashed after initialization, notify all pending handlers
          wasmEventHandlers.forEach((handler, id) => {
            handler({
              type: 'error',
              payload: { id, message: 'WASM worker crashed' },
            });
          });
          wasmEventHandlers.clear();
        }
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

  // Start search performance monitoring
  performanceMonitor.startSearch();

  let fallbackStopHandler: StopHandler | null = null;
  let initFailed = false;

  // Initialize worker if needed
  if (!wasmInitialized) {
    initWasmWorker()
      .then(() => {
        // Only send if init didn't fail in the meantime
        if (!initFailed) {
          wasmWorker?.postMessage({ type: 'analyze', payload: req });
        }
      })
      .catch((error) => {
        // Fall back to fake on error
        logger.error('Failed to initialize WASM worker, falling back to fake:', error);
        initFailed = true;
        wasmEventHandlers.delete(req.id);
        performanceMonitor.endSearch();

        // Start fake analyze and store its stop handler
        // Note: fakeAnalyze will call onEvent with searchInfo and bestMove
        fallbackStopHandler = fakeAnalyze(req, onEvent);
      });
  } else {
    // Send analyze request
    wasmWorker?.postMessage({ type: 'analyze', payload: req });
  }

  // Return stop handler
  return () => {
    if (fallbackStopHandler) {
      // Use fallback stop handler if we fell back
      fallbackStopHandler();
    } else {
      // Normal WASM stop
      wasmWorker?.postMessage({ type: 'stop', id: req.id });
      wasmEventHandlers.delete(req.id);
      performanceMonitor.endSearch();
    }
  };
}

function stopRemote(id: string) {
  if (remoteWs && remoteWs.readyState === WebSocket.OPEN) {
    // Send stop message over WebSocket
    remoteWs.send(
      JSON.stringify({
        type: 'stop',
        id: remoteLastId ?? id,
      }),
    );
  }
  if (remoteWs) {
    remoteWs.close();
    remoteWs = null;
  }
}

export function getEngineMode(): EngineMode {
  const stored = (globalThis as EngineGlobals).__ENGINE_MODE__;
  logger.log('[EngineClient] getEngineMode - stored:', stored);
  // Validate stored mode (in case 'fake' was stored before removal)
  if (stored === 'remote' || stored === 'wasm') {
    logger.log('[EngineClient] getEngineMode - returning valid stored mode:', stored);
    return stored;
  }
  // Reset to default if invalid
  logger.log('[EngineClient] getEngineMode - invalid mode, resetting to remote');
  (globalThis as EngineGlobals).__ENGINE_MODE__ = 'remote';
  return 'remote';
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
      // Default to remote if somehow invalid
      return remoteAnalyze(req, onEvent);
    },
    stop(id: string) {
      if (mode === 'remote') stopRemote(id);
      if (mode === 'wasm') {
        wasmWorker?.postMessage({ type: 'stop', id });
      }
    },
  };
}

/**
 * Get performance metrics
 */
export function getPerformanceMetrics() {
  return performanceMonitor.getMetrics();
}

/**
 * Get performance report
 */
export function getPerformanceReport(): string {
  return performanceMonitor.getReport();
}

/**
 * Reset performance metrics
 */
export function resetPerformanceMetrics(): void {
  performanceMonitor.reset();
}

// ========== Game Engine Client ==========

export interface GameEngineClient {
  validateMove(fen: string, uciMove: string): Promise<boolean>;
  makeMove(fen: string, uciMove: string): Promise<string>;
  legalMoves(fen: string): Promise<string[]>;
  checkGameOver(fen: string): Promise<{ isOver: boolean; status?: string }>;
}

// Fake mode: Simple stub implementation
const fakeGameClient: GameEngineClient = {
  async validateMove(_fen: string, _uciMove: string): Promise<boolean> {
    // Always return true for fake mode
    return true;
  },

  async makeMove(fen: string, uciMove: string): Promise<string> {
    // Flip the turn in the FEN (crude but works for fake mode)
    const parts = fen.split(' ');
    parts[1] = parts[1] === 'w' ? 'b' : 'w';
    return parts.join(' ');
  },

  async legalMoves(_fen: string): Promise<string[]> {
    // Return some fake moves
    return ['e2e4', 'e2e3', 'd2d4', 'd2d3', 'g1f3'];
  },

  async checkGameOver(_fen: string): Promise<{ isOver: boolean; status?: string }> {
    return { isOver: false };
  },
};

// WASM mode: Use WASM bindings
let wasmGameEngine: any = null;

async function initWasmGame() {
  if (wasmGameEngine) return wasmGameEngine;

  // Wait for worker to be initialized
  if (!wasmInitialized) {
    await initWasmWorker();
  }

  // Import WASM module directly (not via worker for game methods)
  try {
    // Note: WASM package must be built first with ./scripts/build-wasm.sh
    // Using eval to bypass Vite's static analysis
    const wasmModule = await eval('import("@chess-ai/engine-wasm")');
    wasmGameEngine = new wasmModule.WasmEngine({ hashSizeMb: 16, threads: 1 });
    return wasmGameEngine;
  } catch (e) {
    logger.error('Failed to load WASM game engine (package not built yet):', e);
    throw new Error('WASM package not available. Run ./scripts/build-wasm.sh first.');
  }
}

const wasmGameClient: GameEngineClient = {
  async validateMove(fen: string, uciMove: string): Promise<boolean> {
    const engine = await initWasmGame();
    return engine.isMoveLegal(fen, uciMove);
  },

  async makeMove(fen: string, uciMove: string): Promise<string> {
    const engine = await initWasmGame();
    return engine.makeMove(fen, uciMove);
  },

  async legalMoves(fen: string): Promise<string[]> {
    const engine = await initWasmGame();
    return engine.legalMoves(fen);
  },

  async checkGameOver(fen: string): Promise<{ isOver: boolean; status?: string }> {
    const engine = await initWasmGame();
    const result = engine.isGameOver(fen);
    return { isOver: result.isOver, status: result.status };
  },
};

// Remote mode: Use WebSocket server
let remoteGameWs: WebSocket | null = null;
let remotePendingRequests = new Map<string, (value: any) => void>();

function getRemoteGameWs(): Promise<WebSocket> {
  if (remoteGameWs?.readyState === WebSocket.OPEN) {
    return Promise.resolve(remoteGameWs);
  }

  return new Promise((resolve, reject) => {
    const base = import.meta.env.VITE_ENGINE_SERVER_URL || 'ws://127.0.0.1:8080';
    const ws = new WebSocket(base.replace(/^http/, 'ws'));

    ws.onopen = () => {
      remoteGameWs = ws;
      resolve(ws);
    };

    ws.onerror = (error) => {
      reject(error);
    };

    ws.onmessage = (ev) => {
      try {
        const msg = JSON.parse(ev.data);
        const resolver = remotePendingRequests.get(msg.id);
        if (resolver) {
          resolver(msg.payload);
          remotePendingRequests.delete(msg.id);
        }
      } catch (e) {
        logger.error('Failed to parse remote game message:', e);
      }
    };
  });
}

function sendRemoteGameRequest(type: string, payload: any): Promise<any> {
  const id = Math.random().toString(36).substring(7);
  return new Promise(async (resolve, reject) => {
    try {
      const ws = await getRemoteGameWs();
      remotePendingRequests.set(id, resolve);

      ws.send(
        JSON.stringify({
          type,
          id,
          ...payload,
        }),
      );

      // Timeout after 5 seconds
      setTimeout(() => {
        if (remotePendingRequests.has(id)) {
          remotePendingRequests.delete(id);
          reject(new Error('Remote game request timeout'));
        }
      }, 5000);
    } catch (e) {
      reject(e);
    }
  });
}

const remoteGameClient: GameEngineClient = {
  async validateMove(fen: string, uciMove: string): Promise<boolean> {
    logger.log('[RemoteGameClient] validateMove:', { fen, uciMove });
    try {
      const result = await sendRemoteGameRequest('validateMove', { fen, uci_move: uciMove });
      logger.log('[RemoteGameClient] validateMove result:', result);
      return result.valid;
    } catch (e) {
      logger.error('[RemoteGameClient] validateMove error:', e);
      return false;
    }
  },

  async makeMove(fen: string, uciMove: string): Promise<string> {
    logger.log('[RemoteGameClient] makeMove:', { fen, uciMove });
    try {
      const result = await sendRemoteGameRequest('makeMove', { fen, uci_move: uciMove });
      logger.log('[RemoteGameClient] makeMove result:', result);
      if (result.error) {
        throw new Error(result.error);
      }
      return result.fen;
    } catch (e) {
      logger.error('[RemoteGameClient] makeMove error:', e);
      throw e;
    }
  },

  async legalMoves(fen: string): Promise<string[]> {
    logger.log('[RemoteGameClient] legalMoves:', fen);
    try {
      const result = await sendRemoteGameRequest('legalMoves', { fen });
      logger.log('[RemoteGameClient] legalMoves result:', result);
      return result.moves;
    } catch (e) {
      logger.error('[RemoteGameClient] legalMoves error:', e);
      return [];
    }
  },

  async checkGameOver(fen: string): Promise<{ isOver: boolean; status?: string }> {
    logger.log('[RemoteGameClient] checkGameOver:', fen);
    try {
      const result = await sendRemoteGameRequest('gameStatus', { fen });
      logger.log('[RemoteGameClient] checkGameOver result:', result);
      return { isOver: result.isOver, status: result.status };
    } catch (e) {
      logger.error('[RemoteGameClient] checkGameOver error:', e);
      return { isOver: false };
    }
  },
};

/**
 * Hook for game-specific engine methods (move validation, application, etc.)
 */
export function useGameEngine(): GameEngineClient {
  const mode = getEngineMode();
  if (mode === 'remote') return remoteGameClient;
  if (mode === 'wasm') return wasmGameClient;
  // Default to remote if somehow invalid
  return remoteGameClient;
}
