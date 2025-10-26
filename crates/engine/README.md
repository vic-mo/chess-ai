# Chess Engine Core

A high-performance chess engine core library written in Rust, featuring bitboard representation, complete move generation, and world-class performance.

## Features

- **Bitboard Representation** - Efficient 64-bit board state representation
- **Complete Move Generation** - All piece types including special moves (castling, en passant, promotion)
- **Legal Move Validation** - Full check detection and legality verification
- **Zobrist Hashing** - Incremental position hashing for transposition tables
- **FEN Support** - Full FEN parsing and serialization
- **High Performance** - 26M+ nodes/second in perft testing (8.7x target)
- **Fully Tested** - 154+ tests with perft validation to depth 6

## Quick Start

```rust
use engine::board::Board;
use engine::r#move::{Move, MoveFlags};
use engine::square::Square;

// Create a board from starting position
let mut board = Board::startpos();

// Generate legal moves
let legal_moves = board.generate_legal_moves();
println!("Legal moves from start: {}", legal_moves.len()); // 20

// Make a move (e2-e4)
let m = Move::new(Square::E2, Square::E4, MoveFlags::DOUBLE_PAWN_PUSH);
let undo = board.make_move(m);

// Unmake the move
board.unmake_move(m, undo);
```

## FEN Parsing

```rust
use engine::io::{parse_fen, board_to_fen};

// Parse a FEN string
let fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
let board = parse_fen(fen).unwrap();

// Convert back to FEN
let fen_out = board_to_fen(&board);
assert_eq!(fen, fen_out);
```

## Performance

Run benchmarks with:

```bash
cargo bench --bench perft_bench
```

**Results (Release mode):**

| Position  | Depth | Nodes     | Time      | Nodes/sec |
| --------- | ----- | --------- | --------- | --------- |
| Startpos  | 5     | 4,865,609 | 186.15 ms | **26.1M** |
| Kiwipete  | 4     | 4,085,603 | 218.03 ms | **18.7M** |
| Position3 | 4     | 43,238    | 1.46 ms   | **29.6M** |

## Testing

```bash
# Run all tests
cargo test

# Run with ignored (slow) tests
cargo test -- --ignored

# Run specific test
cargo test perft
```

## Architecture

### Core Types

- **Board** - Main board state with bitboards for each piece type
- **Bitboard** - 64-bit integer representing square occupancy
- **Square** - Type-safe square representation (0-63)
- **Move** - Packed 16-bit move encoding
- **Piece** - Piece type and color

### Key Modules

- `board` - Board representation and core operations
- `bitboard` - Efficient square set operations
- `movegen` - Move generation for all pieces
- `attacks` - Precomputed attack tables
- `zobrist` - Position hashing
- `io` - FEN parsing/serialization
- `perft` - Performance testing

## Examples

See the [`examples/`](examples/) directory for complete examples:

- `basic_usage.rs` - Basic board operations
- `move_generation.rs` - Generating and displaying moves
- `fen_parsing.rs` - Working with FEN strings
- `perft_runner.rs` - Performance testing utility

## Documentation

Build and view documentation:

```bash
cargo doc --no-deps --open
```

## License

Part of the chess-ai-monorepo project.
