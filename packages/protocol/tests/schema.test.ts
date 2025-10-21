import { describe, it, expect } from 'vitest';
import * as Schema from '../src/schema';
import type * as Types from '../src/types';

describe('Protocol Schema Validation', () => {
  describe('Score', () => {
    it('validates centipawn score', () => {
      const valid: Types.Score = { kind: 'cp', value: 150 };
      expect(Schema.Score.safeParse(valid).success).toBe(true);
    });

    it('validates mate score', () => {
      const valid: Types.Score = { kind: 'mate', plies: 5 };
      expect(Schema.Score.safeParse(valid).success).toBe(true);
    });

    it('rejects invalid score kind', () => {
      const invalid = { kind: 'invalid', value: 100 };
      expect(Schema.Score.safeParse(invalid).success).toBe(false);
    });

    it('rejects score missing required fields', () => {
      const invalid = { kind: 'cp' };
      expect(Schema.Score.safeParse(invalid).success).toBe(false);
    });
  });

  describe('SearchLimit', () => {
    it('validates depth limit', () => {
      const valid: Types.SearchLimit = { kind: 'depth', depth: 10 };
      expect(Schema.SearchLimit.safeParse(valid).success).toBe(true);
    });

    it('validates nodes limit', () => {
      const valid: Types.SearchLimit = { kind: 'nodes', nodes: 1000000 };
      expect(Schema.SearchLimit.safeParse(valid).success).toBe(true);
    });

    it('validates time limit', () => {
      const valid: Types.SearchLimit = { kind: 'time', moveTimeMs: 5000 };
      expect(Schema.SearchLimit.safeParse(valid).success).toBe(true);
    });

    it('validates infinite limit', () => {
      const valid: Types.SearchLimit = { kind: 'infinite' };
      expect(Schema.SearchLimit.safeParse(valid).success).toBe(true);
    });

    it('rejects depth < 1', () => {
      const invalid = { kind: 'depth', depth: 0 };
      expect(Schema.SearchLimit.safeParse(invalid).success).toBe(false);
    });

    it('rejects non-integer depth', () => {
      const invalid = { kind: 'depth', depth: 5.5 };
      expect(Schema.SearchLimit.safeParse(invalid).success).toBe(false);
    });
  });

  describe('EngineOptions', () => {
    it('validates complete options', () => {
      const valid: Types.EngineOptions = {
        hashSizeMB: 128,
        threads: 4,
        contempt: 10,
        skillLevel: 15,
        multiPV: 3,
        useTablebases: true,
      };
      expect(Schema.EngineOptions.safeParse(valid).success).toBe(true);
    });

    it('validates minimal required fields', () => {
      const valid: Types.EngineOptions = {
        hashSizeMB: 64,
        threads: 1,
      };
      expect(Schema.EngineOptions.safeParse(valid).success).toBe(true);
    });

    it('rejects invalid skill level', () => {
      const invalid = {
        hashSizeMB: 64,
        threads: 1,
        skillLevel: 25, // max is 20
      };
      expect(Schema.EngineOptions.safeParse(invalid).success).toBe(false);
    });

    it('rejects negative hash size', () => {
      const invalid = {
        hashSizeMB: -1,
        threads: 1,
      };
      expect(Schema.EngineOptions.safeParse(invalid).success).toBe(false);
    });
  });

  describe('AnalyzeRequest', () => {
    it('validates complete request', () => {
      const valid: Types.AnalyzeRequest = {
        id: 'test-123',
        fen: 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1',
        moves: ['e2e4', 'e7e5'],
        limit: { kind: 'depth', depth: 10 },
        options: { hashSizeMB: 64, threads: 1 },
        context: { allowPonder: true },
      };
      expect(Schema.AnalyzeRequest.safeParse(valid).success).toBe(true);
    });

    it('validates minimal request', () => {
      const valid: Types.AnalyzeRequest = {
        id: 'test-456',
        fen: 'startpos',
        limit: { kind: 'infinite' },
      };
      expect(Schema.AnalyzeRequest.safeParse(valid).success).toBe(true);
    });

    it('rejects request missing id', () => {
      const invalid = {
        fen: 'startpos',
        limit: { kind: 'depth', depth: 5 },
      };
      expect(Schema.AnalyzeRequest.safeParse(invalid).success).toBe(false);
    });

    it('rejects request with invalid limit', () => {
      const invalid = {
        id: 'test',
        fen: 'startpos',
        limit: { kind: 'depth', depth: 0 }, // depth must be >= 1
      };
      expect(Schema.AnalyzeRequest.safeParse(invalid).success).toBe(false);
    });
  });

  describe('SearchInfo', () => {
    it('validates complete search info', () => {
      const valid: Types.SearchInfo = {
        id: 'search-1',
        depth: 10,
        seldepth: 12,
        nodes: 1000000,
        nps: 500000,
        timeMs: 2000,
        score: { kind: 'cp', value: 50 },
        pv: ['e2e4', 'e7e5', 'g1f3'],
        hashfull: 500,
        tbHits: 100,
      };
      expect(Schema.SearchInfo.safeParse(valid).success).toBe(true);
    });

    it('validates minimal search info', () => {
      const valid: Types.SearchInfo = {
        id: 'search-2',
        depth: 1,
        nodes: 20,
        nps: 1000,
        timeMs: 20,
        score: { kind: 'mate', plies: 3 },
        pv: ['f7f8q'],
      };
      expect(Schema.SearchInfo.safeParse(valid).success).toBe(true);
    });

    it('rejects negative depth', () => {
      const invalid = {
        id: 'search-3',
        depth: -1,
        nodes: 100,
        nps: 1000,
        timeMs: 100,
        score: { kind: 'cp', value: 0 },
        pv: [],
      };
      expect(Schema.SearchInfo.safeParse(invalid).success).toBe(false);
    });

    it('rejects hashfull > 1000', () => {
      const invalid = {
        id: 'search-4',
        depth: 5,
        nodes: 1000,
        nps: 1000,
        timeMs: 1000,
        score: { kind: 'cp', value: 0 },
        pv: ['e2e4'],
        hashfull: 1001, // max is 1000
      };
      expect(Schema.SearchInfo.safeParse(invalid).success).toBe(false);
    });
  });

  describe('BestMove', () => {
    it('validates best move with ponder', () => {
      const valid: Types.BestMove = {
        id: 'move-1',
        best: 'e2e4',
        ponder: 'e7e5',
      };
      expect(Schema.BestMove.safeParse(valid).success).toBe(true);
    });

    it('validates best move without ponder', () => {
      const valid: Types.BestMove = {
        id: 'move-2',
        best: 'g1f3',
      };
      expect(Schema.BestMove.safeParse(valid).success).toBe(true);
    });

    it('rejects missing best field', () => {
      const invalid = {
        id: 'move-3',
        ponder: 'e7e5',
      };
      expect(Schema.BestMove.safeParse(invalid).success).toBe(false);
    });
  });

  describe('EngineEvent', () => {
    it('validates searchInfo event', () => {
      const valid: Types.EngineEvent = {
        type: 'searchInfo',
        payload: {
          id: 'evt-1',
          depth: 5,
          nodes: 10000,
          nps: 50000,
          timeMs: 200,
          score: { kind: 'cp', value: 25 },
          pv: ['e2e4'],
        },
      };
      expect(Schema.EngineEvent.safeParse(valid).success).toBe(true);
    });

    it('validates bestMove event', () => {
      const valid: Types.EngineEvent = {
        type: 'bestMove',
        payload: {
          id: 'evt-2',
          best: 'd2d4',
          ponder: 'd7d5',
        },
      };
      expect(Schema.EngineEvent.safeParse(valid).success).toBe(true);
    });

    it('validates error event', () => {
      const valid: Types.EngineEvent = {
        type: 'error',
        payload: {
          id: 'evt-3',
          message: 'Invalid FEN string',
        },
      };
      expect(Schema.EngineEvent.safeParse(valid).success).toBe(true);
    });

    it('rejects invalid event type', () => {
      const invalid = {
        type: 'unknown',
        payload: {},
      };
      expect(Schema.EngineEvent.safeParse(invalid).success).toBe(false);
    });

    it('rejects event with wrong payload structure', () => {
      const invalid = {
        type: 'bestMove',
        payload: {
          // missing 'best' field
          id: 'evt-4',
        },
      };
      expect(Schema.EngineEvent.safeParse(invalid).success).toBe(false);
    });
  });
});
