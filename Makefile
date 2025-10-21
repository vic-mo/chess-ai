.PHONY: setup lint lint-rust fmt fmt-check test build ci

setup:
	pnpm install
	cargo fetch --locked

lint:
	pnpm lint

lint-rust:
	pnpm lint:rust

fmt:
	pnpm fmt

fmt-check:
	pnpm fmt:check

build:
	cargo build --workspace --all-targets
	pnpm build

ci:
	pnpm install --frozen-lockfile
	pnpm fmt:check
	pnpm lint
	pnpm lint:rust
	pnpm test
	cargo test --workspace --all-features
