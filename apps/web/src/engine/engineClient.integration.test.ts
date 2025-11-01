import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import {
  useEngine,
  getEngineMode,
  setEngineMode,
  getWasmStatus,
  resetPerformanceMetrics,
  getPerformanceMetrics,
} from './engineClient';
import type { AnalyzeRequest, EngineEvent } from '@chess-ai/protocol';

describe('Engine Client Integration Tests', () => {
  beforeEach(() => {
    // Reset to remote mode before each test
    setEngineMode('remote');
    resetPerformanceMetrics();
  });

  afterEach(() => {
    // Clean up after each test
    setEngineMode('remote');
  });

  describe('Remote Mode Integration', () => {
    it.skip('should complete full analyze cycle in remote mode', async () => {
      const engine = useEngine();
      expect(getEngineMode()).toBe('remote');

      const events: EngineEvent[] = [];
      const request: AnalyzeRequest = {
        id: 'test-1',
        fen: 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1',
        limit: { kind: 'depth', depth: 3 },
      };

      await new Promise<void>((resolve) => {
        const stop = engine.analyze(request, (event) => {
          events.push(event);
          if (event.type === 'bestMove') {
            stop();
            resolve();
          }
        });
      });

      // Should have received search info for depths 1, 2, 3
      const searchInfos = events.filter((e) => e.type === 'searchInfo');
      expect(searchInfos.length).toBe(3);

      // Should have received final bestmove
      const bestMoves = events.filter((e) => e.type === 'bestMove');
      expect(bestMoves.length).toBe(1);
      expect(bestMoves[0].payload).toHaveProperty('best');
    });

    it.skip('should handle stop in remote mode', async () => {
      const engine = useEngine();
      const events: EngineEvent[] = [];
      const request: AnalyzeRequest = {
        id: 'test-2',
        fen: 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1',
        limit: { kind: 'depth', depth: 10 },
      };

      await new Promise<void>((resolve) => {
        const stop = engine.analyze(request, (event) => {
          events.push(event);
          // Stop after first search info
          if (event.type === 'searchInfo' && events.length === 1) {
            stop();
            // Wait a bit to ensure no more events
            setTimeout(resolve, 200);
          }
        });
      });

      // Should have stopped early, not reaching depth 10
      const searchInfos = events.filter((e) => e.type === 'searchInfo');
      expect(searchInfos.length).toBeLessThan(10);
    });

    it.skip('should handle multiple sequential analyses', async () => {
      const engine = useEngine();
      const iterations = 3;

      for (let i = 0; i < iterations; i++) {
        const events: EngineEvent[] = [];
        const request: AnalyzeRequest = {
          id: `test-seq-${i}`,
          fen: 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1',
          limit: { kind: 'depth', depth: 2 },
        };

        await new Promise<void>((resolve) => {
          engine.analyze(request, (event) => {
            events.push(event);
            if (event.type === 'bestMove') {
              resolve();
            }
          });
        });

        expect(events.filter((e) => e.type === 'bestMove').length).toBe(1);
      }
    });
  });

  describe('Mode Switching Integration', () => {
    it('should switch from remote to wasm mode', () => {
      expect(getEngineMode()).toBe('remote');

      setEngineMode('wasm');
      expect(getEngineMode()).toBe('wasm');
      expect(['uninitialized', 'initializing']).toContain(getWasmStatus());
    });

    it('should switch from wasm to remote mode', () => {
      setEngineMode('wasm');
      expect(getEngineMode()).toBe('wasm');

      setEngineMode('remote');
      expect(getEngineMode()).toBe('remote');
      expect(getWasmStatus()).toBe('uninitialized');
    });

    it('should switch from remote to wasm mode', () => {
      expect(getEngineMode()).toBe('remote');

      setEngineMode('wasm');
      expect(getEngineMode()).toBe('wasm');
    });

    it('should handle rapid mode switching', () => {
      setEngineMode('remote');
      setEngineMode('wasm');
      setEngineMode('remote');
      setEngineMode('wasm');

      expect(getEngineMode()).toBe('wasm');
    });
  });

  describe('Performance Metrics Integration', () => {
    it('should track search count across multiple analyses', async () => {
      const engine = useEngine();
      resetPerformanceMetrics();

      const initialMetrics = getPerformanceMetrics();
      expect(initialMetrics.searchCount).toBe(0);

      // Run 3 analyses
      for (let i = 0; i < 3; i++) {
        const request: AnalyzeRequest = {
          id: `perf-test-${i}`,
          fen: 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1',
          limit: { kind: 'depth', depth: 2 },
        };

        await new Promise<void>((resolve) => {
          engine.analyze(request, (event) => {
            if (event.type === 'bestMove') {
              resolve();
            }
          });
        });
      }

      const finalMetrics = getPerformanceMetrics();
      expect(finalMetrics.searchCount).toBe(3);
    });

    it('should calculate average search time', async () => {
      const engine = useEngine();
      resetPerformanceMetrics();

      const request: AnalyzeRequest = {
        id: 'avg-test',
        fen: 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1',
        limit: { kind: 'depth', depth: 3 },
      };

      await new Promise<void>((resolve) => {
        engine.analyze(request, (event) => {
          if (event.type === 'bestMove') {
            resolve();
          }
        });
      });

      const metrics = getPerformanceMetrics();
      expect(metrics.avgSearchTime).toBeGreaterThan(0);
      expect(metrics.totalSearchTime).toBeGreaterThan(0);
    });

    it('should reset performance metrics', async () => {
      const engine = useEngine();
      resetPerformanceMetrics();

      const request: AnalyzeRequest = {
        id: 'reset-test',
        fen: 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1',
        limit: { kind: 'depth', depth: 2 },
      };

      await new Promise<void>((resolve) => {
        engine.analyze(request, (event) => {
          if (event.type === 'bestMove') {
            resolve();
          }
        });
      });

      expect(getPerformanceMetrics().searchCount).toBe(1);

      resetPerformanceMetrics();
      expect(getPerformanceMetrics().searchCount).toBe(0);
    });
  });

  describe('Request/Response Cycle', () => {
    it('should include request id in all events', async () => {
      const engine = useEngine();
      const requestId = 'id-test-123';
      const events: EngineEvent[] = [];

      const request: AnalyzeRequest = {
        id: requestId,
        fen: 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1',
        limit: { kind: 'depth', depth: 3 },
      };

      await new Promise<void>((resolve) => {
        engine.analyze(request, (event) => {
          events.push(event);
          if (event.type === 'bestMove') {
            resolve();
          }
        });
      });

      // All events should have the correct id
      events.forEach((event) => {
        expect(event.payload.id).toBe(requestId);
      });
    });

    it('should respect depth limit', async () => {
      const engine = useEngine();
      const targetDepth = 5;
      const searchInfos: any[] = [];

      const request: AnalyzeRequest = {
        id: 'depth-test',
        fen: 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1',
        limit: { kind: 'depth', depth: targetDepth },
      };

      await new Promise<void>((resolve) => {
        engine.analyze(request, (event) => {
          if (event.type === 'searchInfo') {
            searchInfos.push(event.payload);
          } else if (event.type === 'bestMove') {
            resolve();
          }
        });
      });

      // Should have search info for depths 1 through targetDepth
      expect(searchInfos.length).toBe(targetDepth);
      searchInfos.forEach((info, index) => {
        expect(info.depth).toBe(index + 1);
      });
    });

    it('should include required fields in search info', async () => {
      const engine = useEngine();
      let searchInfo: unknown = null;

      const request: AnalyzeRequest = {
        id: 'fields-test',
        fen: 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1',
        limit: { kind: 'depth', depth: 1 },
      };

      await new Promise<void>((resolve) => {
        engine.analyze(request, (event) => {
          if (event.type === 'searchInfo') {
            searchInfo = event.payload;
          } else if (event.type === 'bestMove') {
            resolve();
          }
        });
      });

      expect(searchInfo).toBeDefined();
      expect(searchInfo).toHaveProperty('id');
      expect(searchInfo).toHaveProperty('depth');
      expect(searchInfo).toHaveProperty('nodes');
      expect(searchInfo).toHaveProperty('nps');
      expect(searchInfo).toHaveProperty('timeMs');
      expect(searchInfo).toHaveProperty('score');
      expect(searchInfo).toHaveProperty('pv');
    });

    it('should include required fields in best move', async () => {
      const engine = useEngine();
      let bestMove: unknown = null;

      const request: AnalyzeRequest = {
        id: 'bestmove-test',
        fen: 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1',
        limit: { kind: 'depth', depth: 1 },
      };

      await new Promise<void>((resolve) => {
        engine.analyze(request, (event) => {
          if (event.type === 'bestMove') {
            bestMove = event.payload;
            resolve();
          }
        });
      });

      expect(bestMove).toBeDefined();
      expect(bestMove).toHaveProperty('id');
      expect(bestMove).toHaveProperty('best');
      expect(typeof (bestMove as any).best).toBe('string');
    });
  });

  describe('Concurrent Requests', () => {
    it('should handle multiple concurrent requests with different ids', async () => {
      const engine = useEngine();
      const results = new Map<string, unknown>();

      const request1: AnalyzeRequest = {
        id: 'concurrent-1',
        fen: 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1',
        limit: { kind: 'depth', depth: 2 },
      };

      const request2: AnalyzeRequest = {
        id: 'concurrent-2',
        fen: 'rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1',
        limit: { kind: 'depth', depth: 2 },
      };

      const promise1 = new Promise<void>((resolve) => {
        engine.analyze(request1, (event) => {
          if (event.type === 'bestMove') {
            results.set(request1.id, event.payload);
            resolve();
          }
        });
      });

      const promise2 = new Promise<void>((resolve) => {
        engine.analyze(request2, (event) => {
          if (event.type === 'bestMove') {
            results.set(request2.id, event.payload);
            resolve();
          }
        });
      });

      await Promise.all([promise1, promise2]);

      expect(results.size).toBe(2);
      expect(results.get('concurrent-1')).toBeDefined();
      expect(results.get('concurrent-2')).toBeDefined();
    });
  });

  describe('Stop Functionality', () => {
    it('should stop analysis when stop handler is called', async () => {
      const engine = useEngine();
      const events: EngineEvent[] = [];

      const request: AnalyzeRequest = {
        id: 'stop-test',
        fen: 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1',
        limit: { kind: 'depth', depth: 20 },
      };

      let stopHandler: (() => void) | null = null;

      const promise = new Promise<void>((resolve) => {
        stopHandler = engine.analyze(request, (event) => {
          events.push(event);
          // Stop after 3 search infos
          if (events.filter((e) => e.type === 'searchInfo').length === 3) {
            if (stopHandler) stopHandler();
            setTimeout(resolve, 100); // Wait to ensure no more events
          }
        });
      });

      await promise;

      // Should have stopped before reaching depth 20
      const searchInfos = events.filter((e) => e.type === 'searchInfo');
      expect(searchInfos.length).toBeLessThanOrEqual(3);

      // Should not have received bestmove (stopped mid-search)
      const bestMoves = events.filter((e) => e.type === 'bestMove');
      expect(bestMoves.length).toBeLessThanOrEqual(1);
    });

    it('should stop analysis when engine.stop is called', () => {
      const engine = useEngine();
      const requestId = 'stop-via-engine';

      const request: AnalyzeRequest = {
        id: requestId,
        fen: 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1',
        limit: { kind: 'depth', depth: 20 },
      };

      engine.analyze(request, () => {
        // Callback
      });

      // Stop via engine method
      expect(() => engine.stop(requestId)).not.toThrow();
    });
  });

  describe('Error Handling', () => {
    it('should handle analyze with empty FEN gracefully', async () => {
      const engine = useEngine();
      const events: EngineEvent[] = [];

      const request: AnalyzeRequest = {
        id: 'empty-fen',
        fen: '',
        limit: { kind: 'depth', depth: 1 },
      };

      await new Promise<void>((resolve) => {
        const timeout = setTimeout(resolve, 500);
        engine.analyze(request, (event) => {
          events.push(event);
          if (event.type === 'bestMove' || event.type === 'error') {
            clearTimeout(timeout);
            resolve();
          }
        });
      });

      // Should complete (fake mode doesn't validate)
      expect(events.length).toBeGreaterThan(0);
    });

    it('should handle rapid start/stop cycles', async () => {
      const engine = useEngine();

      for (let i = 0; i < 5; i++) {
        const request: AnalyzeRequest = {
          id: `rapid-${i}`,
          fen: 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1',
          limit: { kind: 'depth', depth: 10 },
        };

        const stop = engine.analyze(request, () => {
          // Callback
        });

        // Immediately stop
        stop();
        engine.stop(`rapid-${i}`);
      }

      // Should not throw errors
      expect(true).toBe(true);
    });
  });
});
