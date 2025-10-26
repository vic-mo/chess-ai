//! Perft (performance test) runner example.
//!
//! Demonstrates how to use the perft functions to validate move generation
//! and measure performance.

use engine::board::Board;
use engine::io::parse_fen;
use engine::perft::{perft, perft_divide};
use std::time::Instant;

fn main() {
    println!("=== Perft Runner ===\n");

    // Test starting position
    println!("Testing starting position...");
    let board = Board::startpos();
    run_perft(&board, 5);
    println!();

    // Test Kiwipete position (complex midgame)
    println!("Testing Kiwipete position...");
    let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
    let board = parse_fen(fen).unwrap();
    run_perft(&board, 4);
    println!();

    // Show move breakdown for depth 2
    println!("=== Move breakdown (startpos, depth 2) ===");
    let board = Board::startpos();
    let results = perft_divide(&board, 2);

    let mut total = 0u64;
    for (mv, count) in results.iter().take(10) {
        total += count;
        println!("{}: {}", mv, count);
    }
    println!("... ({} more moves)", results.len() - 10);
    println!("Total: {}", total);
}

fn run_perft(board: &Board, max_depth: u32) {
    for depth in 1..=max_depth {
        let start = Instant::now();
        let nodes = perft(board, depth);
        let elapsed = start.elapsed();

        let nps = (nodes as f64 / elapsed.as_secs_f64()) as u64;

        println!(
            "  Depth {}: {:>12} nodes in {:>8.2?} ({:>8} nps)",
            depth,
            nodes,
            elapsed,
            format_nps(nps)
        );
    }
}

fn format_nps(nps: u64) -> String {
    if nps >= 1_000_000 {
        format!("{:.1}M", nps as f64 / 1_000_000.0)
    } else if nps >= 1_000 {
        format!("{:.1}K", nps as f64 / 1_000.0)
    } else {
        format!("{}", nps)
    }
}
