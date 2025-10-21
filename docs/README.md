# Chess AI Documentation

Welcome to the technical docs for the Chess AI monorepo. Follow the milestone specs to keep the Rust engine and TypeScript client aligned.

## Milestones

- [M0 — Repository Setup and Tooling](./M0.md)
- [M1 — Shared Protocol Contract](./M1.md)
- [M2 — ???](./M2.txt)
- [M3 — ???](./M3.txt)
- [M4 — ???](./M4.txt)
- [M5 — ???](./M5.txt)
- [M6 — ???](./M6.txt)
- [M7 — ???](./M7.txt)

> Specs beyond M1 are placeholders and will be iterated as implementation progresses.

## Getting Started

1. Install the toolchains pinned in `.nvmrc` (Node 20.14.0) and `rust-toolchain.toml` (Rust stable + `rustfmt`, `clippy`).
2. Run `make setup` to install pnpm dependencies and pre-fetch Rust crates.
3. Start the frontend via `pnpm --filter web dev`.
4. Build the Rust workspace with `cargo build --workspace` or run the server stub using `cargo run -p engine-server`.

## Quality Gates

- JavaScript/TypeScript formatting: `pnpm fmt` / `pnpm fmt:check` (Prettier).
- Linting: `pnpm lint` for TS/React, `pnpm lint:rust` for Rust (`cargo fmt --check` + `cargo clippy`).
- Tests: `pnpm test` (recursive Vitest) and `cargo test --workspace --all-features`.

The `pre-commit` hook runs the formatting and linting checks automatically. It is installed when you run `pnpm install` thanks to the `prepare` script.

## CI Cheatsheet

Use `make ci` locally to mirror the GitHub Actions workflow.
