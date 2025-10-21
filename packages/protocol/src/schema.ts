import { z } from 'zod';

export const Score = z.union([
  z.object({ kind: z.literal('cp'), value: z.number() }),
  z.object({ kind: z.literal('mate'), plies: z.number() }),
]);

export const SearchLimit = z.union([
  z.object({ kind: z.literal('depth'), depth: z.number().int().min(1) }),
  z.object({ kind: z.literal('nodes'), nodes: z.number().int().min(1) }),
  z.object({ kind: z.literal('time'), moveTimeMs: z.number().int().min(1) }),
  z.object({ kind: z.literal('infinite') }),
]);

export const EngineOptions = z.object({
  hashSizeMB: z.number().int().min(1),
  threads: z.number().int().min(1),
  contempt: z.number().optional(),
  skillLevel: z.number().int().min(0).max(20).optional(),
  multiPV: z.number().int().min(1).optional(),
  useTablebases: z.boolean().optional(),
});

export const AnalyzeRequest = z.object({
  id: z.string(),
  fen: z.string(),
  moves: z.array(z.string()).optional(),
  limit: SearchLimit,
  options: EngineOptions.partial().optional(),
  context: z.object({ allowPonder: z.boolean().optional() }).optional(),
});

export const SearchInfo = z.object({
  id: z.string(),
  depth: z.number().int().min(0),
  seldepth: z.number().int().min(0).optional(),
  nodes: z.number().int().min(0),
  nps: z.number().int().min(0),
  timeMs: z.number().int().min(0),
  score: Score,
  pv: z.array(z.string()),
  hashfull: z.number().int().min(0).max(1000).optional(),
  tbHits: z.number().int().min(0).optional(),
});

export const BestMove = z.object({
  id: z.string(),
  best: z.string(),
  ponder: z.string().optional(),
});

export const EngineEvent = z.union([
  z.object({ type: z.literal('searchInfo'), payload: SearchInfo }),
  z.object({ type: z.literal('bestMove'), payload: BestMove }),
  z.object({
    type: z.literal('error'),
    payload: z.object({ id: z.string(), message: z.string() }),
  }),
]);
