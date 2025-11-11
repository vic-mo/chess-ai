import type { AnalyzeRequest, EngineEvent, SearchInfo, BestMove } from '@chess-ai/protocol';
import { Schema } from '@chess-ai/protocol';
import { performanceMonitor } from '../utils/performance';
import { logger } from '../utils/logger';
import { WebSocketConnectionManager } from './connectionManager';
import { retry } from '../utils/retryHelper';

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

// Remote connection manager
let remoteConnectionManager: WebSocketConnectionManager | null = null;
const remoteEventHandlers = new Map<string, EventHandler>();

function getRemoteConnectionManager(): WebSocketConnectionManager {
  if (!remoteConnectionManager) {
    const base = import.meta.env.VITE_ENGINE_SERVER_URL || 'ws://127.0.0.1:8080';
    const wsUrl = base.replace(/^http/, 'ws');
    remoteConnectionManager = new WebSocketConnectionManager(wsUrl);

    // Set up global message handler for all analyze requests
    remoteConnectionManager.onMessage((data) => {
      try {
        logger.log('[RemoteAnalyze] Raw message:', data);
        const result = Schema.EngineEvent.safeParse(data);
        logger.log('[RemoteAnalyze] Schema validation:', result);

        if (result.success) {
          const event = result.data;
          logger.log('[RemoteAnalyze] Valid event:', event);

          // Find handler by ID
          const payload = event.payload as { id: string };
          const handler = remoteEventHandlers.get(payload.id);

          if (handler) {
            logger.log('[RemoteAnalyze] Calling handler for ID:', payload.id);
            handler(event);

            // Clean up handler on bestMove or error
            if (event.type === 'bestMove' || event.type === 'error') {
              remoteEventHandlers.delete(payload.id);
            }
          } else {
            logger.warn('[RemoteAnalyze] No handler for ID:', payload.id);
          }
        } else {
          logger.error('[RemoteAnalyze] Schema validation failed:', result.error);
        }
      } catch (e) {
        logger.error('[RemoteAnalyze] Failed to handle message:', e);
      }
    });

    // Auto-connect
    remoteConnectionManager.connect().catch((error) => {
      logger.error('[RemoteAnalyze] Initial connection failed:', error);
    });
  }

  return remoteConnectionManager;
}

function remoteAnalyze(req: AnalyzeRequest, onEvent: EventHandler): StopHandler {
  const connectionManager = getRemoteConnectionManager();

  // Register event handler
  remoteEventHandlers.set(req.id, onEvent);

  // Set up timeout for this request
  const timeoutId = setTimeout(() => {
    if (remoteEventHandlers.has(req.id)) {
      logger.error('[RemoteAnalyze] Request timeout for ID:', req.id);
      remoteEventHandlers.delete(req.id);
      onEvent({
        type: 'error',
        payload: { id: req.id, message: 'Request timeout' },
      });
    }
  }, 30000); // 30s timeout

  // Send analyze request
  connectionManager.send({
    type: 'analyze',
    id: req.id,
    fen: req.fen,
    limit: req.limit,
  });

  // Return cleanup function
  const cleanup: StopHandler = () => {
    clearTimeout(timeoutId);
    remoteEventHandlers.delete(req.id);

    // Send stop message if still connected
    if (connectionManager.isConnected()) {
      connectionManager.send({
        type: 'stop',
        id: req.id,
      });
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
  const connectionManager = remoteConnectionManager;
  if (connectionManager?.isConnected()) {
    connectionManager.send({
      type: 'stop',
      id,
    });
  }
  remoteEventHandlers.delete(id);
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

  // Reset remote connection manager when switching modes
  if (mode !== 'remote' && remoteConnectionManager) {
    remoteConnectionManager.disconnect();
    remoteConnectionManager = null;
    remoteEventHandlers.clear();
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

// Remote mode: Use WebSocket server via connection manager
let remotePendingGameRequests = new Map<
  string,
  { resolve: (value: any) => void; reject: (error: any) => void; timeoutId: NodeJS.Timeout }
>();
let gameMessageHandlerRegistered = false;

function setupGameMessageHandler(): void {
  if (gameMessageHandlerRegistered) {
    return;
  }

  const connectionManager = getRemoteConnectionManager();

  // Register handler for game-specific messages (not engine events)
  const unsubscribe = connectionManager.onMessage((msg) => {
    // Skip if this is an engine event (handled elsewhere)
    if (msg.type === 'searchInfo' || msg.type === 'bestMove' || msg.type === 'error') {
      return;
    }

    // Handle game method responses
    if (msg.id && remotePendingGameRequests.has(msg.id)) {
      const request = remotePendingGameRequests.get(msg.id);
      if (request) {
        clearTimeout(request.timeoutId);
        remotePendingGameRequests.delete(msg.id);
        request.resolve(msg.payload);
      }
    }
  });

  gameMessageHandlerRegistered = true;
}

function sendRemoteGameRequest(type: string, payload: any): Promise<any> {
  setupGameMessageHandler();

  const connectionManager = getRemoteConnectionManager();
  const id = Math.random().toString(36).substring(7);

  return new Promise((resolve, reject) => {
    // Set up timeout
    const timeoutId = setTimeout(() => {
      if (remotePendingGameRequests.has(id)) {
        remotePendingGameRequests.delete(id);
        reject(new Error('Remote game request timeout'));
      }
    }, 5000);

    // Register pending request
    remotePendingGameRequests.set(id, { resolve, reject, timeoutId });

    // Send request
    connectionManager.send({
      type,
      id,
      ...payload,
    });
  });
}

const remoteGameClient: GameEngineClient = {
  async validateMove(fen: string, uciMove: string): Promise<boolean> {
    logger.log('[RemoteGameClient] validateMove:', { fen, uciMove });
    try {
      const result = await retry(
        () => sendRemoteGameRequest('validateMove', { fen, uci_move: uciMove }),
        { maxAttempts: 3, initialDelay: 500, maxDelay: 2000 },
      );
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
      const result = await retry(
        () => sendRemoteGameRequest('makeMove', { fen, uci_move: uciMove }),
        { maxAttempts: 3, initialDelay: 500, maxDelay: 2000 },
      );
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
      const result = await retry(() => sendRemoteGameRequest('legalMoves', { fen }), {
        maxAttempts: 3,
        initialDelay: 500,
        maxDelay: 2000,
      });
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
      const result = await retry(() => sendRemoteGameRequest('gameStatus', { fen }), {
        maxAttempts: 3,
        initialDelay: 500,
        maxDelay: 2000,
      });
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

/**
 * Get remote connection manager (for status monitoring)
 */
export function getConnectionManager(): WebSocketConnectionManager | null {
  return remoteConnectionManager;
}
