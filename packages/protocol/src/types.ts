export const PROTOCOL_VERSION = '1.0.0';

export type UciMove = string;

export type SearchLimit =
  | { kind: 'depth'; depth: number }
  | { kind: 'nodes'; nodes: number }
  | { kind: 'time'; moveTimeMs: number }
  | { kind: 'infinite' };

export type Score = { kind: 'cp'; value: number } | { kind: 'mate'; plies: number };

export interface EngineOptions {
  hashSizeMB: number;
  threads: number;
  contempt?: number;
  skillLevel?: number;
  multiPV?: number;
  useTablebases?: boolean;
}

export interface AnalyzeRequest {
  id: string;
  fen: string;
  moves?: UciMove[];
  limit: SearchLimit;
  options?: Partial<EngineOptions>;
  context?: { allowPonder?: boolean };
}

export interface SearchInfo {
  id: string;
  depth: number;
  seldepth?: number;
  nodes: number;
  nps: number;
  timeMs: number;
  score: Score;
  pv: UciMove[];
  hashfull?: number;
  tbHits?: number;
}

export interface BestMove {
  id: string;
  best: UciMove;
  ponder?: UciMove;
}

export type EngineEvent =
  | { type: 'searchInfo'; payload: SearchInfo }
  | { type: 'bestMove'; payload: BestMove }
  | { type: 'error'; payload: { id: string; message: string } };
