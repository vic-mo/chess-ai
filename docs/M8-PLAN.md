# M8: WASM Integration - Implementation Plan

## Executive Summary

**Goal**: Integrate the Rust chess engine with the React frontend via WebAssembly, enabling high-performance browser-based chess analysis.

**Duration**: 1-2 weeks (8-12 sessions)

**Current Status**:

- ✅ WASM bridge code exists (`crates/engine-bridge-wasm`)
- ✅ Web app scaffold exists (`apps/web`)
- ✅ Engine client abstraction exists
- ✅ Protocol types defined
- ⚠️ No WASM build configuration
- ⚠️ No worker implementation
- ⚠️ No integration testing

**Success Criteria**:

- ✅ WASM binary builds and loads in browser
- ✅ Binary size <2.5MB (optimized)
- ✅ Performance within 1.5× of native
- ✅ Works in Chrome, Firefox, Safari
- ✅ No UI thread blocking
- ✅ Protocol parity with native engine
- ✅ Comprehensive integration tests

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────┐
│                   React App (UI Thread)              │
│  - User interactions                                 │
│  - State management                                  │
│  - Rendering                                         │
└──────────────────┬──────────────────────────────────┘
                   │ postMessage
                   │ (AnalyzeRequest, Stop)
                   ↓
┌─────────────────────────────────────────────────────┐
│              Web Worker (Worker Thread)              │
│  - WASM engine instance                              │
│  - Message handling                                  │
│  - Event streaming                                   │
└──────────────────┬──────────────────────────────────┘
                   │ WASM calls
                   │ (position, analyze, stop)
                   ↓
┌─────────────────────────────────────────────────────┐
│         WASM Module (engine-bridge-wasm)             │
│  - WasmEngine wrapper                                │
│  - serde_wasm_bindgen serialization                  │
│  - Callback handling                                 │
└──────────────────┬──────────────────────────────────┘
                   │ Rust calls
                   │ (EngineImpl methods)
                   ↓
┌─────────────────────────────────────────────────────┐
│           Rust Engine Core (engine crate)            │
│  - Board representation                              │
│  - Move generation                                   │
│  - Search (M3-M7 techniques)                         │
│  - Evaluation                                        │
└─────────────────────────────────────────────────────┘
```

---

## Session Breakdown

### **Session 1-2: WASM Build Configuration** (2-3 days)

**Goal**: Set up WASM build pipeline with optimizations

**Tasks**:

1. Add WASM build scripts to `package.json`
2. Configure `wasm-pack` for optimal builds
3. Set up `wasm-opt` for size optimization
4. Configure Cargo.toml for WASM target
5. Add build profiles (dev/release)
6. Test basic WASM build

**Deliverables**:

- `scripts/build-wasm.sh` - Build automation script
- `crates/engine-bridge-wasm/Cargo.toml` - WASM-optimized config
- `.github/workflows/wasm-build.yml` - CI integration
- `apps/web/public/` - WASM artifacts directory

**Files to Modify/Create**:

```
scripts/build-wasm.sh                    (new)
crates/engine-bridge-wasm/Cargo.toml     (modify)
package.json                             (modify)
.github/workflows/wasm-build.yml         (new)
```

**Acceptance Criteria**:

- ✅ `pnpm build:wasm` produces optimized WASM binary
- ✅ Binary size <2.5MB (gzipped <1MB ideal)
- ✅ Build completes in <2 minutes
- ✅ CI can build WASM artifacts

---

### **Session 3-4: Web Worker Implementation** (2-3 days)

**Goal**: Implement Web Worker to run WASM engine off main thread

**Tasks**:

1. Create `engine.worker.ts` with WASM loading
2. Implement message handling (analyze, stop, position)
3. Set up event streaming from engine to UI
4. Handle WASM initialization and error cases
5. Add worker lifecycle management
6. Test worker communication

**Deliverables**:

- `apps/web/src/workers/engine.worker.ts` - Main worker implementation
- `apps/web/src/workers/wasm-loader.ts` - WASM initialization helper
- Worker communication tests

**Files to Create/Modify**:

```
apps/web/src/workers/engine.worker.ts    (modify/complete)
apps/web/src/workers/wasm-loader.ts      (new)
apps/web/src/workers/types.ts            (new)
```

**Code Structure**:

```typescript
// engine.worker.ts
import init, { WasmEngine } from '@/wasm/engine-bridge-wasm';
import type { AnalyzeRequest, EngineEvent } from '@chess-ai/protocol';

let engine: WasmEngine | null = null;

self.onmessage = async (e: MessageEvent) => {
  const { type, payload } = e.data;

  switch (type) {
    case 'init':
      await initWasm();
      break;
    case 'analyze':
      handleAnalyze(payload as AnalyzeRequest);
      break;
    case 'stop':
      handleStop(payload.id);
      break;
  }
};

async function initWasm() {
  await init();
  engine = new WasmEngine({
    /* options */
  });
  self.postMessage({ type: 'ready' });
}

function handleAnalyze(req: AnalyzeRequest) {
  if (!engine) return;

  engine.position(req.fen, req.moves || []);

  const callback = (info: SearchInfo) => {
    self.postMessage({
      type: 'searchInfo',
      payload: info,
    });
  };

  const result = engine.analyze(req.limit, callback);

  self.postMessage({
    type: 'bestMove',
    payload: result,
  });
}
```

**Acceptance Criteria**:

- ✅ Worker loads WASM successfully
- ✅ Worker processes analyze requests
- ✅ Search info streams to UI without blocking
- ✅ Stop command terminates search
- ✅ Proper error handling

---

### **Session 5-6: Engine Client Integration** (2 days)

**Goal**: Update engine client to support WASM mode

**Tasks**:

1. Implement WASM engine client class
2. Add mode detection (fake/wasm/remote)
3. Create worker pool for concurrent analysis
4. Implement request/response correlation
5. Add timeout and error handling
6. Update `useEngine` hook

**Deliverables**:

- `apps/web/src/engine/wasmClient.ts` - WASM client implementation
- `apps/web/src/engine/engineClient.ts` - Updated mode switching
- Integration with React hooks

**Files to Modify/Create**:

```
apps/web/src/engine/wasmClient.ts       (new)
apps/web/src/engine/engineClient.ts     (modify)
apps/web/src/engine/types.ts            (new)
```

**Code Structure**:

```typescript
// wasmClient.ts
export class WasmEngineClient {
  private worker: Worker;
  private callbacks: Map<string, EventCallback>;

  constructor() {
    this.worker = new Worker(new URL('../workers/engine.worker.ts', import.meta.url), {
      type: 'module',
    });
    this.worker.onmessage = this.handleMessage.bind(this);
  }

  async init(): Promise<void> {
    return new Promise((resolve) => {
      this.worker.postMessage({ type: 'init' });
      // Wait for 'ready' message
    });
  }

  analyze(req: AnalyzeRequest, callback: EventCallback): void {
    this.callbacks.set(req.id, callback);
    this.worker.postMessage({
      type: 'analyze',
      payload: req,
    });
  }

  stop(id: string): void {
    this.worker.postMessage({
      type: 'stop',
      payload: { id },
    });
  }

  private handleMessage(e: MessageEvent): void {
    const { type, payload } = e.data;
    // Route to appropriate callback
  }
}

// engineClient.ts - Mode switching
export function getEngineMode(): 'fake' | 'wasm' | 'remote' {
  return (import.meta.env.VITE_ENGINE_MODE || 'fake') as any;
}

export function useEngine() {
  const mode = getEngineMode();

  switch (mode) {
    case 'wasm':
      return useWasmEngine();
    case 'remote':
      return useRemoteEngine();
    default:
      return useFakeEngine();
  }
}
```

**Acceptance Criteria**:

- ✅ Mode switching works via environment variable
- ✅ WASM client maintains connection to worker
- ✅ Multiple concurrent requests handled correctly
- ✅ Callbacks invoked with proper events
- ✅ Cleanup on unmount

---

### **Session 7-8: Performance Optimization** (2 days)

**Goal**: Optimize WASM binary size and runtime performance

**Tasks**:

1. Enable LTO (Link-Time Optimization)
2. Strip debug symbols and unnecessary code
3. Use `wasm-opt` for aggressive optimization
4. Profile WASM performance vs native
5. Optimize serialization overhead
6. Add performance benchmarks
7. Memory profiling and leak detection

**Deliverables**:

- Optimized build configuration
- Performance benchmark suite
- Size optimization report

**Optimization Checklist**:

```toml
# Cargo.toml
[profile.release]
opt-level = 'z'        # Optimize for size
lto = true             # Link-time optimization
codegen-units = 1      # Better optimization
panic = 'abort'        # Smaller binary
strip = true           # Remove symbols

[package.metadata.wasm-pack.profile.release]
wasm-opt = ['-Oz']     # Aggressive size optimization
```

**Build Script**:

```bash
#!/bin/bash
# scripts/build-wasm.sh

set -e

echo "Building WASM with optimizations..."

# Build with wasm-pack
wasm-pack build \
  --target web \
  --out-dir ../../apps/web/src/wasm \
  --release \
  crates/engine-bridge-wasm

# Further optimize with wasm-opt
wasm-opt \
  -Oz \
  --strip-debug \
  --strip-producers \
  -o apps/web/src/wasm/engine_bg.wasm \
  apps/web/src/wasm/engine_bg.wasm

# Report size
echo "Final size:"
ls -lh apps/web/src/wasm/engine_bg.wasm
gzip -c apps/web/src/wasm/engine_bg.wasm | wc -c | \
  awk '{print "Gzipped: " $1/1024/1024 "MB"}'
```

**Performance Benchmarks**:

```typescript
// apps/web/src/benchmarks/wasm-perf.test.ts
describe('WASM Performance', () => {
  it('should search within 1.5x of native speed', async () => {
    const wasmEngine = new WasmEngineClient();
    await wasmEngine.init();

    const start = performance.now();
    await wasmEngine.analyze(
      {
        id: 'bench',
        fen: 'startpos',
        limit: { kind: 'depth', depth: 6 },
      },
      () => {},
    );
    const elapsed = performance.now() - start;

    // Compare with native benchmark
    expect(elapsed).toBeLessThan(nativeTime * 1.5);
  });

  it('should have binary size < 2.5MB', async () => {
    const response = await fetch('/wasm/engine_bg.wasm');
    const blob = await response.blob();
    expect(blob.size).toBeLessThan(2.5 * 1024 * 1024);
  });
});
```

**Acceptance Criteria**:

- ✅ Binary size <2.5MB (ideally <2MB)
- ✅ Gzipped size <1MB
- ✅ NPS within 1.5× of native
- ✅ No memory leaks after 100+ searches
- ✅ Startup time <500ms

---

### **Session 9-10: Cross-Browser Testing** (1-2 days)

**Goal**: Ensure WASM works across all major browsers

**Tasks**:

1. Test in Chrome/Chromium
2. Test in Firefox
3. Test in Safari
4. Test on mobile browsers (iOS Safari, Chrome Mobile)
5. Handle browser-specific issues
6. Add browser compatibility detection
7. Provide fallback for unsupported browsers

**Deliverables**:

- Browser compatibility matrix
- Feature detection code
- Fallback UI for unsupported browsers

**Browser Detection**:

```typescript
// apps/web/src/utils/browser-support.ts
export function checkWasmSupport(): {
  supported: boolean;
  reason?: string;
} {
  // Check WebAssembly support
  if (typeof WebAssembly !== 'object') {
    return {
      supported: false,
      reason: 'WebAssembly not supported',
    };
  }

  // Check Worker support
  if (typeof Worker === 'undefined') {
    return {
      supported: false,
      reason: 'Web Workers not supported',
    };
  }

  // Check SharedArrayBuffer (optional, for threading)
  if (typeof SharedArrayBuffer === 'undefined') {
    console.warn('SharedArrayBuffer not available - multi-threading disabled');
  }

  return { supported: true };
}

// Usage in App.tsx
useEffect(() => {
  const support = checkWasmSupport();
  if (!support.supported) {
    setError(`WASM not supported: ${support.reason}`);
    setEngineMode('fake'); // Fallback to fake engine
  }
}, []);
```

**Test Matrix**:
| Browser | Version | WASM | Workers | Status |
|---------|---------|------|---------|--------|
| Chrome | 90+ | ✅ | ✅ | ✅ |
| Firefox | 88+ | ✅ | ✅ | ✅ |
| Safari | 14+ | ✅ | ✅ | ⚠️ (test) |
| Edge | 90+ | ✅ | ✅ | ✅ |
| iOS Safari | 14+ | ✅ | ✅ | ⚠️ (test) |
| Chrome Mobile | 90+ | ✅ | ✅ | ⚠️ (test) |

**Acceptance Criteria**:

- ✅ Works in Chrome, Firefox, Edge
- ✅ Works in Safari (desktop)
- ✅ Works on iOS Safari
- ✅ Graceful degradation for unsupported browsers
- ✅ Clear error messages

---

### **Session 11-12: Integration Testing & Documentation** (2 days)

**Goal**: Comprehensive testing and documentation

**Tasks**:

1. Write E2E tests for WASM mode
2. Test concurrent analysis requests
3. Test position switching
4. Test stop/restart scenarios
5. Test error recovery
6. Write integration documentation
7. Create usage examples
8. Update README

**Deliverables**:

- Complete test suite
- Integration documentation
- Usage guide
- Performance report

**E2E Tests**:

```typescript
// apps/web/src/tests/wasm-integration.test.ts
describe('WASM Integration', () => {
  let client: WasmEngineClient;

  beforeAll(async () => {
    client = new WasmEngineClient();
    await client.init();
  });

  it('should analyze startpos to depth 6', async () => {
    const events: EngineEvent[] = [];

    await new Promise((resolve) => {
      client.analyze(
        {
          id: 'test-1',
          fen: 'startpos',
          limit: { kind: 'depth', depth: 6 },
        },
        (evt) => {
          events.push(evt);
          if (evt.type === 'bestMove') resolve(null);
        },
      );
    });

    // Should receive multiple searchInfo events
    const searchInfos = events.filter((e) => e.type === 'searchInfo');
    expect(searchInfos.length).toBeGreaterThan(0);

    // Should end with bestMove
    const lastEvent = events[events.length - 1];
    expect(lastEvent.type).toBe('bestMove');
    expect(lastEvent.payload.best).toBeTruthy();
  });

  it('should stop search on demand', async () => {
    const events: EngineEvent[] = [];
    const id = 'test-stop';

    client.analyze(
      {
        id,
        fen: 'startpos',
        limit: { kind: 'depth', depth: 20 }, // Long search
      },
      (evt) => {
        events.push(evt);
      },
    );

    // Stop after 100ms
    await new Promise((r) => setTimeout(r, 100));
    client.stop(id);

    await new Promise((r) => setTimeout(r, 100));

    // Should have stopped before depth 20
    const depths = events.filter((e) => e.type === 'searchInfo').map((e) => e.payload.depth);

    expect(Math.max(...depths)).toBeLessThan(20);
  });

  it('should handle position changes', async () => {
    // First analysis
    await analyzePosition('startpos', 5);

    // Different position
    const events = await analyzePosition(
      'r1bqkbnr/pppp1ppp/2n5/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 2 3',
      5,
    );

    // Should analyze new position
    expect(events.length).toBeGreaterThan(0);
  });

  it('should handle concurrent requests', async () => {
    const results = await Promise.all([
      analyzePosition('startpos', 4),
      analyzePosition('rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1', 4),
      analyzePosition('r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4', 4),
    ]);

    // All should complete
    results.forEach((events) => {
      expect(events.length).toBeGreaterThan(0);
      expect(events[events.length - 1].type).toBe('bestMove');
    });
  });
});
```

**Documentation**:

````markdown
# M8: WASM Integration - User Guide

## Building WASM

```bash
# Install wasm-pack
cargo install wasm-pack

# Build WASM
pnpm build:wasm

# Build optimized
pnpm build:wasm:release
```
````

## Using WASM Engine

1. Set environment variable:

```bash
# .env.local
VITE_ENGINE_MODE=wasm
```

2. Start dev server:

```bash
pnpm --filter web dev
```

3. The app will automatically use WASM engine

## Performance

- Binary size: ~2.2MB (uncompressed), ~850KB (gzipped)
- Startup time: ~200ms
- Search performance: 85-95% of native speed
- Supports depths up to 20 comfortably

## Browser Support

- Chrome 90+: ✅ Full support
- Firefox 88+: ✅ Full support
- Safari 14+: ✅ Full support
- Edge 90+: ✅ Full support
- Mobile: ✅ iOS Safari 14+, Chrome Mobile 90+

## Troubleshooting

**WASM fails to load**

- Check browser console for errors
- Ensure WASM files are served with correct MIME type
- Try clearing browser cache

**Slow performance**

- Check if running in development mode
- Use production build: `pnpm build`
- Disable browser extensions that may interfere

**Worker errors**

- Check browser supports Workers
- Ensure not running from `file://` protocol (use http server)

````

**Acceptance Criteria**:
- ✅ All integration tests passing
- ✅ Documentation complete
- ✅ Usage examples provided
- ✅ Performance report available
- ✅ Known issues documented

---

## Build Configuration

### Package.json Scripts

```json
{
  "scripts": {
    "build:wasm": "./scripts/build-wasm.sh",
    "build:wasm:dev": "./scripts/build-wasm.sh --dev",
    "build:wasm:release": "./scripts/build-wasm.sh --release",
    "test:wasm": "vitest run src/tests/wasm-integration.test.ts",
    "bench:wasm": "vitest bench src/benchmarks/wasm-perf.test.ts"
  }
}
````

### Cargo Configuration

```toml
# crates/engine-bridge-wasm/Cargo.toml
[package]
name = "engine-bridge-wasm"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
engine = { path = "../engine" }
wasm-bindgen = "0.2"
serde = { version = "1.0", features = ["derive"] }
serde-wasm-bindgen = "0.6"
js-sys = "0.3"

[profile.release]
opt-level = 'z'
lto = true
codegen-units = 1
panic = 'abort'
strip = true

[package.metadata.wasm-pack.profile.release]
wasm-opt = ['-Oz', '--enable-mutable-globals']
```

---

## Risk Mitigation

### Risk 1: Large Binary Size

**Mitigation**:

- Aggressive optimization flags
- Strip debug symbols
- Use `wasm-opt` post-processing
- Consider code splitting if needed
- Monitor size in CI

### Risk 2: Performance Degradation

**Mitigation**:

- Profile with Chrome DevTools
- Optimize hot paths
- Use SIMD where available
- Benchmark against native regularly
- Consider threading if needed

### Risk 3: Browser Compatibility

**Mitigation**:

- Feature detection before loading
- Graceful fallback to fake engine
- Clear error messages
- Test on all target browsers
- Use polyfills if needed

### Risk 4: Worker Communication Overhead

**Mitigation**:

- Batch updates where possible
- Use efficient serialization
- Minimize message frequency
- Profile communication bottlenecks
- Consider SharedArrayBuffer for high-frequency data

### Risk 5: Memory Leaks

**Mitigation**:

- Proper cleanup on component unmount
- Monitor memory usage in DevTools
- Regular garbage collection checks
- Profile long-running sessions
- Add memory usage tests

---

## Success Metrics

### Functional Metrics

- ✅ WASM loads successfully: 100%
- ✅ All protocol methods work: 100%
- ✅ Cross-browser compatibility: Chrome, Firefox, Safari
- ✅ No crashes after 1000+ searches

### Performance Metrics

- ✅ Binary size: <2.5MB (target <2MB)
- ✅ Gzipped size: <1MB (target <850KB)
- ✅ Startup time: <500ms (target <200ms)
- ✅ NPS: >50% of native (target >70%)
- ✅ Memory stable: <100MB after 100 searches

### Quality Metrics

- ✅ Test coverage: >80%
- ✅ E2E tests: 100% passing
- ✅ Documentation: Complete
- ✅ CI/CD: Automated builds

---

## Timeline

**Total Duration**: 1-2 weeks (12 sessions max)

```
Week 1:
  Day 1-2: Sessions 1-2 (Build Configuration)
  Day 3-4: Sessions 3-4 (Worker Implementation)
  Day 5:   Sessions 5-6 (Client Integration)

Week 2:
  Day 1-2: Sessions 7-8 (Optimization)
  Day 3:   Sessions 9-10 (Browser Testing)
  Day 4-5: Sessions 11-12 (Testing & Docs)
```

**Critical Path**:

1. Build Configuration → Worker Implementation → Client Integration
2. Optimization can partially overlap with integration
3. Testing happens throughout, final validation at end

---

## Next Steps After M8

Once M8 is complete:

1. **M9**: Enhance frontend with interactive chessboard
2. **M10**: Implement server mode for cloud deployment
3. **M11**: Advanced features (opening book, tablebase)
4. **M12**: Production polish and v1.0 release

**M8 is a critical milestone** - it enables browser-based chess analysis and unblocks frontend development.

---

**Status**: 📋 Ready to implement
**Dependencies**: M7 complete ✅
**Blocks**: M9 (Frontend enhancements)
