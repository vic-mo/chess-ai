# Engine Performance Benchmarks

## Target: ≥3M nodes/second

**Status: ✅ EXCEEDED (8.7x target)**

## Benchmark Results (Release Mode)

### Primary Benchmarks

| Position  | Depth | Nodes     | Time      | Nodes/sec |
| --------- | ----- | --------- | --------- | --------- |
| Startpos  | 4     | 197,281   | 7.80 ms   | **25.3M** |
| Startpos  | 5     | 4,865,609 | 186.15 ms | **26.1M** |
| Kiwipete  | 4     | 4,085,603 | 218.03 ms | **18.7M** |
| Position3 | 4     | 43,238    | 1.46 ms   | **29.6M** |

### Performance Summary

- **Peak Performance**: 29.6M nodes/sec (Position3 depth 4)
- **Average Performance**: ~25M nodes/sec
- **vs Target**: 8.7x faster than 3M nps goal

## Optimizations Applied

### Session 18 Optimizations

1. **Added Criterion benchmarks** - Professional benchmarking framework
2. **Added inline directives** - Critical functions marked with `#[inline]`
   - `Board::piece_at()`
   - Already present in: `Bitboard`, `Square`, `Move`, `Piece`, `Color`, attack lookups

### Architecture Benefits

The engine achieves exceptional performance due to:

1. **Bitboard representation** - Efficient 64-bit operations
2. **Move packing** - Moves fit in 16 bits
3. **Precomputed attack tables** - Constant-time lookups
4. **Incremental updates** - Zobrist hashing, make/unmake
5. **Zero-cost abstractions** - Rust's inline optimization

## Test Coverage

- **154 tests passing**
- **Perft validation**: Depths 1-5 match canonical values
- **Correctness verified** across 6 standard test positions

## Running Benchmarks

```bash
# Run all benchmarks
cargo bench --bench perft_bench

# Run specific depth
cargo bench --bench perft_bench -- "depth 4"

# Quick performance check
cargo test --lib perft::tests::test_perft_startpos_depth_5 --release -- --nocapture
```

## System Info

- Platform: macOS (darwin 24.5.0)
- Compiler: rustc with --release optimizations
- Date: 2025-10-26
