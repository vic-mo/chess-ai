import { useEffect, useRef, useState } from 'react';
import { AnalyzeRequest, SearchInfo, BestMove } from '@chess-ai/protocol';
import {
  useEngine,
  getEngineMode,
  setEngineMode,
  preloadWasm,
  getPerformanceReport,
  resetPerformanceMetrics,
} from './engine/engineClient';
import { getCompatibilityInfo, formatCompatibilityReport } from './utils/browserDetect';
import { PlayMode } from './components/PlayMode';
import './styles.css';

type EngineMode = 'fake' | 'remote' | 'wasm';
type AppMode = 'analysis' | 'play';

export default function App() {
  const engine = useEngine();
  const [appMode, setAppMode] = useState<AppMode>('analysis');
  const [fen, setFen] = useState('startpos');
  const [depth, setDepth] = useState(8);
  const [logs, setLogs] = useState<string[]>([]);
  const [mode, setMode] = useState<EngineMode>(getEngineMode());
  const [wasmStatus, setWasmStatus] = useState<string>('uninitialized');
  const [isAnalyzing, setIsAnalyzing] = useState(false);
  const [showPerformance, setShowPerformance] = useState(false);
  const [performanceReport, setPerformanceReport] = useState<string>('');
  const [showCompatibility, setShowCompatibility] = useState(false);
  const idRef = useRef<string>('');
  const stopHandlerRef = useRef<(() => void) | null>(null);
  const perfIntervalRef = useRef<number | null>(null);

  const compatInfo = getCompatibilityInfo();

  function log(line: string) {
    setLogs((prev) => [line, ...prev].slice(0, 200));
  }

  const onModeChange = (newMode: EngineMode) => {
    setEngineMode(newMode);
    setMode(newMode);
    log(`Switched to ${newMode} mode`);

    // Preload WASM if switching to WASM mode
    if (newMode === 'wasm') {
      setWasmStatus('initializing');
      preloadWasm()
        .then(() => {
          setWasmStatus('ready');
          log('WASM engine ready');
        })
        .catch((error) => {
          setWasmStatus('error');
          log(`WASM initialization failed: ${error.message}`);
        });
    }
  };

  const onAnalyze = async () => {
    setIsAnalyzing(true);
    idRef.current = crypto.randomUUID();
    log(`Starting analysis (${mode} mode, depth ${depth})...`);

    const req: AnalyzeRequest = {
      id: idRef.current,
      fen,
      limit: { kind: 'depth', depth },
    };

    const stopHandler = engine.analyze(req, (evt) => {
      if (evt.type === 'searchInfo') {
        const i: SearchInfo = evt.payload;
        log(
          `depth ${i.depth}  score ${i.score.kind === 'cp' ? i.score.value + 'cp' : 'mate ' + i.score.plies}  pv ${i.pv.join(' ')}`,
        );
      } else if (evt.type === 'bestMove') {
        const bm: BestMove = evt.payload;
        log(`bestmove ${bm.best}${bm.ponder ? ' ponder ' + bm.ponder : ''}`);
        setIsAnalyzing(false);
        stopHandlerRef.current = null;
      } else if (evt.type === 'error') {
        log(`error: ${evt.payload.message}`);
        setIsAnalyzing(false);
        stopHandlerRef.current = null;
      }
    });

    stopHandlerRef.current = stopHandler;
  };

  const onStop = () => {
    if (stopHandlerRef.current) {
      stopHandlerRef.current();
      stopHandlerRef.current = null;
    }
    engine.stop(idRef.current);
    setIsAnalyzing(false);
    log('Analysis stopped');
  };

  // Update performance metrics periodically
  useEffect(() => {
    if (showPerformance) {
      const updateMetrics = () => {
        setPerformanceReport(getPerformanceReport());
      };

      // Update immediately
      updateMetrics();

      // Update every second
      perfIntervalRef.current = window.setInterval(updateMetrics, 1000);

      return () => {
        if (perfIntervalRef.current) {
          clearInterval(perfIntervalRef.current);
          perfIntervalRef.current = null;
        }
      };
    }
  }, [showPerformance, isAnalyzing]);

  useEffect(() => {
    log(`Engine mode: ${getEngineMode()}`);
  }, []);

  return (
    <div className="container">
      <h1>♟️ Chess AI - WASM Engine</h1>

      {/* App Mode Switcher */}
      <div className="card">
        <div className="row">
          <button
            className="btn"
            onClick={() => setAppMode('analysis')}
            style={{
              backgroundColor: appMode === 'analysis' ? '#2563eb' : '#374151',
            }}
          >
            Analysis Mode
          </button>
          <button
            className="btn"
            onClick={() => setAppMode('play')}
            style={{
              backgroundColor: appMode === 'play' ? '#2563eb' : '#374151',
            }}
          >
            Play vs Engine
          </button>
        </div>
      </div>

      {/* Play Mode */}
      {appMode === 'play' && <PlayMode />}

      {/* Analysis Mode */}
      {appMode === 'analysis' && (
        <>
          {/* Browser Compatibility Banner */}
          {(compatInfo.errors.length > 0 || compatInfo.warnings.length > 0) && (
            <div
              className="card"
              style={{
                backgroundColor: compatInfo.errors.length > 0 ? '#fee' : '#ffc',
                borderColor: compatInfo.errors.length > 0 ? '#c00' : '#880',
              }}
            >
              <div
                className="row"
                style={{ justifyContent: 'space-between', alignItems: 'center' }}
              >
                <h3 style={{ margin: 0 }}>
                  {compatInfo.errors.length > 0 ? '⚠️ Compatibility Issues' : '⚠️ Browser Warnings'}
                </h3>
                <button className="btn" onClick={() => setShowCompatibility(!showCompatibility)}>
                  {showCompatibility ? 'Hide' : 'Show'} Details
                </button>
              </div>
              {compatInfo.errors.length > 0 && (
                <div style={{ marginTop: '0.5rem' }}>
                  {compatInfo.errors.map((err, i) => (
                    <div key={i} style={{ color: '#c00' }}>
                      ✗ {err}
                    </div>
                  ))}
                </div>
              )}
              {compatInfo.warnings.length > 0 && (
                <div style={{ marginTop: '0.5rem' }}>
                  {compatInfo.warnings.map((warn, i) => (
                    <div key={i} style={{ color: '#880' }}>
                      ⚠ {warn}
                    </div>
                  ))}
                </div>
              )}
              {showCompatibility && (
                <pre className="mono" style={{ marginTop: '1rem', fontSize: '0.85rem' }}>
                  {formatCompatibilityReport(compatInfo)}
                </pre>
              )}
            </div>
          )}

          <div className="card">
            <h3>Engine Configuration</h3>
            <div className="row">
              <label>
                Engine Mode:&nbsp;
                <select value={mode} onChange={(e) => onModeChange(e.target.value as EngineMode)}>
                  <option value="fake">Fake (Demo)</option>
                  <option value="wasm" disabled title="WASM mode disabled - mate detection bug">
                    WASM (Local) - DISABLED
                  </option>
                  <option value="remote">Remote (Server)</option>
                </select>
              </label>
              {mode === 'wasm' && (
                <span className="small" style={{ marginLeft: '1rem' }}>
                  Status: <strong>{wasmStatus}</strong>
                </span>
              )}
            </div>
          </div>

          <div className="card" style={{ marginTop: '1rem' }}>
            <h3>Analysis</h3>
            <div className="row">
              <label>
                FEN:&nbsp;
                <input
                  className="mono"
                  value={fen}
                  onChange={(e) => setFen(e.target.value)}
                  style={{ width: '36rem' }}
                  disabled={isAnalyzing}
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
                  disabled={isAnalyzing}
                />
              </label>
              <button
                className="btn"
                onClick={onAnalyze}
                disabled={isAnalyzing || (mode === 'wasm' && wasmStatus !== 'ready')}
              >
                {isAnalyzing ? 'Analyzing...' : 'Analyze'}
              </button>
              <button className="btn" onClick={onStop} disabled={!isAnalyzing}>
                Stop
              </button>
            </div>
          </div>

          <div className="card" style={{ marginTop: '1rem' }}>
            <div className="row" style={{ justifyContent: 'space-between', alignItems: 'center' }}>
              <h3>Search Log</h3>
              <div>
                <button
                  className="btn"
                  onClick={() => setShowPerformance(!showPerformance)}
                  style={{ marginRight: '0.5rem' }}
                >
                  {showPerformance ? 'Hide' : 'Show'} Performance
                </button>
                <button className="btn" onClick={() => resetPerformanceMetrics()}>
                  Reset Metrics
                </button>
              </div>
            </div>
            <pre
              className="mono"
              style={{ whiteSpace: 'pre-wrap', maxHeight: '400px', overflow: 'auto' }}
            >
              {logs.join('\n')}
            </pre>
          </div>

          {showPerformance && (
            <div className="card" style={{ marginTop: '1rem' }}>
              <h3>Performance Metrics</h3>
              <pre className="mono" style={{ whiteSpace: 'pre-wrap' }}>
                {performanceReport || 'No metrics available yet'}
              </pre>
            </div>
          )}
        </>
      )}
    </div>
  );
}
