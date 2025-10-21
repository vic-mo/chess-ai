import { useEffect, useRef, useState } from 'react';
import { AnalyzeRequest, SearchInfo, BestMove } from '@chess-ai/protocol';
import { useEngine, getEngineMode } from './engine/engineClient';
import './styles.css';

export default function App() {
  const engine = useEngine();
  const [fen, setFen] = useState('startpos');
  const [depth, setDepth] = useState(8);
  const [logs, setLogs] = useState<string[]>([]);
  const idRef = useRef<string>('');

  function log(line: string) {
    setLogs((prev) => [line, ...prev].slice(0, 200));
  }

  const onAnalyze = async () => {
    idRef.current = crypto.randomUUID();
    const req: AnalyzeRequest = {
      id: idRef.current,
      fen,
      limit: { kind: 'depth', depth },
    };
    engine.analyze(req, (evt) => {
      if (evt.type === 'searchInfo') {
        const i: SearchInfo = evt.payload;
        log(
          `depth ${i.depth}  score ${i.score.kind === 'cp' ? i.score.value + 'cp' : 'mate ' + i.score.plies}  pv ${i.pv.join(' ')}`,
        );
      } else if (evt.type === 'bestMove') {
        const bm: BestMove = evt.payload;
        log(`bestmove ${bm.best}${bm.ponder ? ' ponder ' + bm.ponder : ''}`);
      } else if (evt.type === 'error') {
        log(`error: ${evt.payload.message}`);
      }
    });
  };

  const onStop = () => engine.stop(idRef.current);

  useEffect(() => {
    log(`Engine mode: ${getEngineMode()}`);
  }, []);

  return (
    <div className="container">
      <h1>♟️ Chess AI Scaffold</h1>
      <div className="card">
        <div className="row">
          <label>
            FEN:&nbsp;
            <input
              className="mono"
              value={fen}
              onChange={(e) => setFen(e.target.value)}
              style={{ width: '36rem' }}
            />
          </label>
          <label>
            Depth:&nbsp;
            <input
              type="number"
              min={1}
              max={99}
              value={depth}
              onChange={(e) => setDepth(parseInt(e.target.value || '8', 10))}
            />
          </label>
          <button className="btn" onClick={onAnalyze}>
            Analyze
          </button>
          <button className="btn" onClick={onStop}>
            Stop
          </button>
        </div>
        <p className="small">
          This demo uses a fake engine stream by default. Wire the WASM/remote client when ready.
        </p>
      </div>

      <div className="card" style={{ marginTop: '1rem' }}>
        <h3>Search Log</h3>
        <pre className="mono" style={{ whiteSpace: 'pre-wrap' }}>
          {logs.join('\n')}
        </pre>
      </div>
    </div>
  );
}
