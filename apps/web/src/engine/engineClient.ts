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

function wasmAnalyze(req: AnalyzeRequest, onEvent: EventHandler): StopHandler {
  // Placeholder: you will import your wasm pkg and use it here.
  // For now, route to fake until wasm is wired.
  return fakeAnalyze(req, onEvent);
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
      // fake/wasm: no-op for scaffold
    },
  };
}
