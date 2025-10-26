# Build Scripts

## WASM Build Script

`build-wasm.sh` - Builds the chess engine as WebAssembly for browser use.

### Prerequisites

- **wasm-pack**: `cargo install wasm-pack`
- **binaryen** (optional, for wasm-opt): `brew install binaryen` (macOS) or `apt install binaryen` (Linux)

### Usage

```bash
# Build release version (optimized for production)
pnpm build:wasm
# or
./scripts/build-wasm.sh --release

# Build development version (faster compilation, larger binary)
pnpm build:wasm:dev
# or
./scripts/build-wasm.sh --dev
```

### Output

WASM files are generated in `apps/web/public/wasm/`:

- `engine_bridge_wasm_bg.wasm` - The compiled WebAssembly binary
- `engine_bridge_wasm.js` - JavaScript glue code
- `engine_bridge_wasm.d.ts` - TypeScript type definitions
- `package.json` - Package metadata

### Build Statistics

**Development Build:**

- WASM binary: ~540 KB
- Gzipped: ~163 KB
- Build time: ~7s (first build), <1s (cached)

**Release Build:**

- WASM binary: ~95 KB
- Gzipped: ~40 KB
- Build time: ~6s (first build), <1s (cached)

### Optimization

The release build applies aggressive size optimization:

- `opt-level = 'z'` - Optimize for smallest binary size
- `lto = true` - Link-time optimization
- `codegen-units = 1` - Maximum optimization (slower compile)
- `panic = 'abort'` - No unwinding (smaller binary)
- `strip = true` - Remove debug symbols
- `wasm-opt -Oz` - Additional WASM-specific optimization

### Troubleshooting

**wasm-pack not found:**

```bash
cargo install wasm-pack
```

**wasm-opt errors:**

If you see validation errors from wasm-opt, ensure you have the latest version:

```bash
brew upgrade binaryen  # macOS
```

The build will still succeed without wasm-opt, but the binary will be slightly larger.

**Build failures:**

Clean the build cache and try again:

```bash
cargo clean
./scripts/build-wasm.sh --release
```
