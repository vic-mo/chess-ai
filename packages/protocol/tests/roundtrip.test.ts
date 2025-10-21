import { describe, it, expect } from 'vitest';
import * as Schema from '../src/schema';
import type * as Types from '../src/types';

describe('Protocol JSON Roundtrip', () => {
  describe('Score', () => {
    it('roundtrips centipawn score', () => {
      const original: Types.Score = { kind: 'cp', value: 123 };
      const json = JSON.stringify(original);
      const parsed = JSON.parse(json);
      const result = Schema.Score.safeParse(parsed);

      expect(result.success).toBe(true);
      if (result.success) {
        expect(result.data).toEqual(original);
      }
    });

    it('roundtrips mate score', () => {
      const original: Types.Score = { kind: 'mate', plies: 7 };
      const json = JSON.stringify(original);
      const parsed = JSON.parse(json);
      const result = Schema.Score.safeParse(parsed);

      expect(result.success).toBe(true);
      if (result.success) {
        expect(result.data).toEqual(original);
      }
    });
  });

  describe('SearchLimit', () => {
    it('roundtrips depth limit', () => {
      const original: Types.SearchLimit = { kind: 'depth', depth: 15 };
      const json = JSON.stringify(original);
      const parsed = JSON.parse(json);
      const result = Schema.SearchLimit.safeParse(parsed);

      expect(result.success).toBe(true);
      if (result.success) {
        expect(result.data).toEqual(original);
      }
    });

    it('roundtrips nodes limit', () => {
      const original: Types.SearchLimit = { kind: 'nodes', nodes: 5000000 };
      const json = JSON.stringify(original);
      const parsed = JSON.parse(json);
      const result = Schema.SearchLimit.safeParse(parsed);

      expect(result.success).toBe(true);
      if (result.success) {
        expect(result.data).toEqual(original);
      }
    });

    it('roundtrips time limit', () => {
      const original: Types.SearchLimit = { kind: 'time', moveTimeMs: 3000 };
      const json = JSON.stringify(original);
      const parsed = JSON.parse(json);
      const result = Schema.SearchLimit.safeParse(parsed);

      expect(result.success).toBe(true);
      if (result.success) {
        expect(result.data).toEqual(original);
      }
    });

    it('roundtrips infinite limit', () => {
      const original: Types.SearchLimit = { kind: 'infinite' };
      const json = JSON.stringify(original);
      const parsed = JSON.parse(json);
      const result = Schema.SearchLimit.safeParse(parsed);

      expect(result.success).toBe(true);
      if (result.success) {
        expect(result.data).toEqual(original);
      }
    });
  });

  describe('EngineOptions', () => {
    it('roundtrips complete options', () => {
      const original: Types.EngineOptions = {
        hashSizeMB: 256,
        threads: 8,
        contempt: 15,
        skillLevel: 18,
        multiPV: 5,
        useTablebases: true,
      };
      const json = JSON.stringify(original);
      const parsed = JSON.parse(json);
      const result = Schema.EngineOptions.safeParse(parsed);

      expect(result.success).toBe(true);
      if (result.success) {
        expect(result.data).toEqual(original);
      }
    });

    it('roundtrips minimal options', () => {
      const original: Types.EngineOptions = {
        hashSizeMB: 64,
        threads: 1,
      };
      const json = JSON.stringify(original);
      const parsed = JSON.parse(json);
      const result = Schema.EngineOptions.safeParse(parsed);

      expect(result.success).toBe(true);
      if (result.success) {
        expect(result.data).toEqual(original);
      }
    });
  });

  describe('AnalyzeRequest', () => {
    it('roundtrips complete request', () => {
      const original: Types.AnalyzeRequest = {
        id: 'req-abc-123',
        fen: 'rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1',
        moves: ['e2e4', 'c7c5', 'g1f3'],
        limit: { kind: 'depth', depth: 20 },
        options: {
          hashSizeMB: 128,
          threads: 4,
          multiPV: 2,
        },
        context: { allowPonder: true },
      };
      const json = JSON.stringify(original);
      const parsed = JSON.parse(json);
      const result = Schema.AnalyzeRequest.safeParse(parsed);

      expect(result.success).toBe(true);
      if (result.success) {
        expect(result.data).toEqual(original);
      }
    });

    it('roundtrips minimal request', () => {
      const original: Types.AnalyzeRequest = {
        id: 'req-minimal',
        fen: 'startpos',
        limit: { kind: 'infinite' },
      };
      const json = JSON.stringify(original);
      const parsed = JSON.parse(json);
      const result = Schema.AnalyzeRequest.safeParse(parsed);

      expect(result.success).toBe(true);
      if (result.success) {
        expect(result.data).toEqual(original);
      }
    });
  });

  describe('SearchInfo', () => {
    it('roundtrips complete search info', () => {
      const original: Types.SearchInfo = {
        id: 'info-xyz-789',
        depth: 18,
        seldepth: 24,
        nodes: 8500000,
        nps: 2100000,
        timeMs: 4048,
        score: { kind: 'cp', value: 87 },
        pv: ['e2e4', 'c7c5', 'g1f3', 'n8c6', 'f1b5'],
        hashfull: 753,
        tbHits: 42,
      };
      const json = JSON.stringify(original);
      const parsed = JSON.parse(json);
      const result = Schema.SearchInfo.safeParse(parsed);

      expect(result.success).toBe(true);
      if (result.success) {
        expect(result.data).toEqual(original);
      }
    });

    it('roundtrips mate score search info', () => {
      const original: Types.SearchInfo = {
        id: 'info-mate',
        depth: 12,
        nodes: 125000,
        nps: 625000,
        timeMs: 200,
        score: { kind: 'mate', plies: 5 },
        pv: ['d8h4', 'g2g3', 'h4f2'],
      };
      const json = JSON.stringify(original);
      const parsed = JSON.parse(json);
      const result = Schema.SearchInfo.safeParse(parsed);

      expect(result.success).toBe(true);
      if (result.success) {
        expect(result.data).toEqual(original);
      }
    });
  });

  describe('BestMove', () => {
    it('roundtrips best move with ponder', () => {
      const original: Types.BestMove = {
        id: 'best-123',
        best: 'e2e4',
        ponder: 'e7e5',
      };
      const json = JSON.stringify(original);
      const parsed = JSON.parse(json);
      const result = Schema.BestMove.safeParse(parsed);

      expect(result.success).toBe(true);
      if (result.success) {
        expect(result.data).toEqual(original);
      }
    });

    it('roundtrips best move without ponder', () => {
      const original: Types.BestMove = {
        id: 'best-456',
        best: 'g1f3',
      };
      const json = JSON.stringify(original);
      const parsed = JSON.parse(json);
      const result = Schema.BestMove.safeParse(parsed);

      expect(result.success).toBe(true);
      if (result.success) {
        expect(result.data).toEqual(original);
      }
    });
  });

  describe('EngineEvent', () => {
    it('roundtrips searchInfo event', () => {
      const original: Types.EngineEvent = {
        type: 'searchInfo',
        payload: {
          id: 'evt-search-1',
          depth: 10,
          nodes: 500000,
          nps: 1000000,
          timeMs: 500,
          score: { kind: 'cp', value: 32 },
          pv: ['d2d4', 'd7d5'],
        },
      };
      const json = JSON.stringify(original);
      const parsed = JSON.parse(json);
      const result = Schema.EngineEvent.safeParse(parsed);

      expect(result.success).toBe(true);
      if (result.success) {
        expect(result.data).toEqual(original);
      }
    });

    it('roundtrips bestMove event', () => {
      const original: Types.EngineEvent = {
        type: 'bestMove',
        payload: {
          id: 'evt-best-1',
          best: 'c2c4',
          ponder: 'e7e5',
        },
      };
      const json = JSON.stringify(original);
      const parsed = JSON.parse(json);
      const result = Schema.EngineEvent.safeParse(parsed);

      expect(result.success).toBe(true);
      if (result.success) {
        expect(result.data).toEqual(original);
      }
    });

    it('roundtrips error event', () => {
      const original: Types.EngineEvent = {
        type: 'error',
        payload: {
          id: 'evt-err-1',
          message: 'Search was interrupted',
        },
      };
      const json = JSON.stringify(original);
      const parsed = JSON.parse(json);
      const result = Schema.EngineEvent.safeParse(parsed);

      expect(result.success).toBe(true);
      if (result.success) {
        expect(result.data).toEqual(original);
      }
    });
  });

  describe('Edge cases', () => {
    it('handles empty PV array', () => {
      const original: Types.SearchInfo = {
        id: 'edge-empty-pv',
        depth: 1,
        nodes: 10,
        nps: 100,
        timeMs: 100,
        score: { kind: 'cp', value: 0 },
        pv: [],
      };
      const json = JSON.stringify(original);
      const parsed = JSON.parse(json);
      const result = Schema.SearchInfo.safeParse(parsed);

      expect(result.success).toBe(true);
      if (result.success) {
        expect(result.data).toEqual(original);
      }
    });

    it('handles zero hashfull', () => {
      const original: Types.SearchInfo = {
        id: 'edge-zero-hash',
        depth: 5,
        nodes: 1000,
        nps: 5000,
        timeMs: 200,
        score: { kind: 'cp', value: 10 },
        pv: ['e2e4'],
        hashfull: 0,
      };
      const json = JSON.stringify(original);
      const parsed = JSON.parse(json);
      const result = Schema.SearchInfo.safeParse(parsed);

      expect(result.success).toBe(true);
      if (result.success) {
        expect(result.data).toEqual(original);
      }
    });

    it('handles max hashfull (1000)', () => {
      const original: Types.SearchInfo = {
        id: 'edge-max-hash',
        depth: 20,
        nodes: 10000000,
        nps: 2000000,
        timeMs: 5000,
        score: { kind: 'mate', plies: -3 },
        pv: ['a2a3'],
        hashfull: 1000,
      };
      const json = JSON.stringify(original);
      const parsed = JSON.parse(json);
      const result = Schema.SearchInfo.safeParse(parsed);

      expect(result.success).toBe(true);
      if (result.success) {
        expect(result.data).toEqual(original);
      }
    });
  });
});
