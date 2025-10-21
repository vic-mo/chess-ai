# @chess-ai/protocol

**Version:** 1.0.0

Shared protocol contract between the TypeScript frontend and Rust chess engine. This package provides type-safe message definitions and validation schemas for engine communication.

## Overview

The protocol defines a versioned, JSON-based wire format that allows:

- TypeScript/React frontend to communicate with the Rust engine
- Support for multiple transport layers (WebSocket, WASM, HTTP)
- Runtime validation with Zod schemas
- Compile-time type safety

## Installation

```bash
pnpm add @chess-ai/protocol
```

## Usage

### TypeScript

```typescript
import { Schema, type AnalyzeRequest, type EngineEvent } from '@chess-ai/protocol';

// Create a request
const request: AnalyzeRequest = {
  id: 'search-123',
  fen: 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1',
  limit: { kind: 'depth', depth: 15 },
};

// Validate incoming data
const result = Schema.EngineEvent.safeParse(data);
if (result.success) {
  const event: EngineEvent = result.data;
  // Handle event
}
```

### Rust

```rust
use engine::types::{AnalyzeRequest, EngineEvent, SearchInfo, Score};

// Deserialize request
let request: AnalyzeRequest = serde_json::from_str(&json)?;

// Serialize response
let event = EngineEvent::SearchInfo {
    payload: SearchInfo {
        id: request.id.clone(),
        depth: 10,
        nodes: 1000000,
        nps: 500000,
        time_ms: 2000,
        score: Score::Cp { value: 50 },
        pv: vec!["e2e4".to_string(), "e7e5".to_string()],
        hashfull: Some(500),
        tb_hits: None,
        seldepth: None,
    },
};
let json = serde_json::to_string(&event)?;
```

## Wire Format

All messages are serialized as JSON with **camelCase** field names.

### AnalyzeRequest

Request to analyze a position:

```json
{
  "id": "req-1",
  "fen": "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
  "moves": ["e2e4", "c7c5"],
  "limit": { "kind": "depth", "depth": 20 },
  "options": {
    "hashSizeMB": 128,
    "threads": 4,
    "multiPV": 2
  },
  "context": {
    "allowPonder": true
  }
}
```

### EngineEvent: SearchInfo

Progress update during search:

```json
{
  "type": "searchInfo",
  "payload": {
    "id": "req-1",
    "depth": 15,
    "seldepth": 20,
    "nodes": 5000000,
    "nps": 1250000,
    "timeMs": 4000,
    "score": { "kind": "cp", "value": 87 },
    "pv": ["e2e4", "c7c5", "g1f3", "d7d6"],
    "hashfull": 650,
    "tbHits": 25
  }
}
```

### EngineEvent: BestMove

Final result of analysis:

```json
{
  "type": "bestMove",
  "payload": {
    "id": "req-1",
    "best": "e2e4",
    "ponder": "c7c5"
  }
}
```

### EngineEvent: Error

Error during analysis:

```json
{
  "type": "error",
  "payload": {
    "id": "req-1",
    "message": "Invalid FEN string"
  }
}
```

## Types

### SearchLimit

Discriminated union controlling search depth:

- `{ kind: 'depth', depth: number }` - Search to fixed depth
- `{ kind: 'nodes', nodes: number }` - Search fixed number of nodes
- `{ kind: 'time', moveTimeMs: number }` - Search for fixed time
- `{ kind: 'infinite' }` - Search indefinitely (must call stop)

### Score

Discriminated union for position evaluation:

- `{ kind: 'cp', value: number }` - Centipawn score (100 = 1 pawn advantage)
- `{ kind: 'mate', plies: number }` - Mate in N plies (positive = we win, negative = we lose)

### EngineOptions

```typescript
interface EngineOptions {
  hashSizeMB: number; // Hash table size (MB)
  threads: number; // Number of search threads
  contempt?: number; // Contempt factor
  skillLevel?: number; // Skill level (0-20)
  multiPV?: number; // Number of principal variations
  useTablebases?: boolean; // Enable tablebase probing
}
```

## Versioning

**Current version:** `1.0.0`

### Semantic Versioning

- **Major (1.x.x)**: Breaking changes to wire format or message structure
- **Minor (x.1.x)**: New optional fields or message types (backward compatible)
- **Patch (x.x.1)**: Documentation, bug fixes (no protocol changes)

### Compatibility Policy

**1.0.0 is FROZEN** - No breaking changes will be made to the protocol contract until 2.0.0.

#### Allowed changes (minor/patch):

- Adding new optional fields
- Adding new enum variants (if clients handle unknown variants)
- Documentation updates
- Bug fixes in validation

#### Breaking changes (major):

- Removing fields
- Changing field types
- Renaming fields
- Making optional fields required
- Changing discriminator values

### Extension Strategy

To add new features while maintaining backward compatibility:

1. **Add optional fields** to existing messages
2. **Version negotiation**: Client sends `protocolVersion` in connection handshake
3. **Feature flags**: Use `context` field for experimental features

Example:

```typescript
// 1.1.0: Add new optional field
interface SearchInfo {
  // ... existing fields
  currMove?: string; // NEW: currently searched move
}
```

## Testing

The protocol includes comprehensive test coverage:

```bash
# Run all tests
pnpm test

# TypeScript tests
cd packages/protocol && pnpm test

# Rust tests
cargo test --test protocol_roundtrip
```

### Test Coverage

- ✅ Schema validation (all types)
- ✅ JSON roundtrip (TS → JSON → TS)
- ✅ Cross-language compatibility (TS JSON → Rust)
- ✅ Shared JSON fixtures

## Field Naming Conventions

| TypeScript Field | Rust Field     | Wire Format  |
| ---------------- | -------------- | ------------ |
| `hashSizeMB`     | `hash_size_mb` | `hashSizeMB` |
| `moveTimeMs`     | `move_time_ms` | `moveTimeMs` |
| `multiPV`        | `multi_pv`     | `multiPV`    |
| `tbHits`         | `tb_hits`      | `tbHits`     |
| `timeMs`         | `time_ms`      | `timeMs`     |

**Rule**: Wire format is always **camelCase**. Rust uses `snake_case` internally with `#[serde(rename)]`.

## Migration Guide (Future)

When breaking changes are necessary:

1. Announce deprecation in documentation
2. Provide migration period (2+ minor versions)
3. Publish upgrade guide
4. Release 2.0.0 with breaking changes

## License

MIT

## Related Documentation

- [M1 — Shared Protocol Contract](../../docs/M1.md) - Full milestone specification
- [Chess AI Architecture](../../docs/README.md) - System overview
