# Chess AI (Rust engine + TypeScript/React UI)

This is a ready-to-run scaffold for a chess AI project:

- **Rust** engine in `crates/engine` (+ WASM bridge in `crates/engine-bridge-wasm`)
- **React + TypeScript** frontend in `apps/web` (with a Worker and a fake engine for demo)
- **Optional server** in `services/engine-server` (Axum + WebSocket streaming)
- **Shared protocol** in `packages/protocol`
- **Docs & specs** in `docs/`

## Quick Start

> Requires: Rust (stable), Node 20.14.0 (see `.nvmrc`), pnpm

### 1) Install deps

```bash
make setup
# or pnpm install
```

### 2) Run the web app (demo uses a **fake engine** stream by default)

```bash
pnpm --filter web dev
```

Open http://localhost:5173

### 3) Build Rust workspace

```bash
cargo build --workspace
```

### 4) (Optional) Run the server

```bash
cargo run -p engine-server
```

Server listens on http://127.0.0.1:8080

> To switch the web app to the remote server client, set `VITE_ENGINE_MODE=remote` in `apps/web/.env.local`.

## Structure

- `apps/web`: React app with a Worker. Fake engine stream by default; wire to WASM when ready.
- `packages/protocol`: shared TypeScript types + Zod schemas.
- `crates/engine`: core rules/search scaffolding (compiles; logic to be implemented).
- `crates/engine-bridge-wasm`: `wasm-bindgen` shim (compiles for wasm32).
- `services/engine-server`: Axum HTTP + WebSocket stub that streams mock search info.
- `docs`: roadmap and in-depth specs.

See `docs/README.md` to follow the milestone plan.
