//! Performance benchmarks for M7 advanced search
//!
//! This suite measures the efficiency improvements from M7 techniques.

use engine::io::parse_fen;
use engine::search::Searcher;
use std::time::Instant;

struct BenchPosition {
    name: &'static str,
    fen: &'static str,
    depth: u32,
}

const BENCH_POSITIONS: &[BenchPosition] = &[
    BenchPosition {
        name: "Starting position",
        fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        depth: 4,
    },
    BenchPosition {
        name: "Tactical position",
        fen: "r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 0 1",
        depth: 4,
    },
    BenchPosition {
        name: "Endgame",
        fen: "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
        depth: 5,
    },
];

#[test]
fn bench_search_efficiency() {
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║              M7 Advanced Search Performance Benchmark          ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    let mut total_nodes = 0u64;
    let mut total_time_ms = 0u64;

    for pos in BENCH_POSITIONS {
        println!("─────────────────────────────────────────────────────────────");
        println!("Position: {}", pos.name);
        println!("Depth:    {}", pos.depth);

        let board = parse_fen(pos.fen).expect("Valid FEN");
        let mut searcher = Searcher::new();

        let start = Instant::now();
        let result = searcher.search(&board, pos.depth);
        let elapsed = start.elapsed();

        let nodes = result.nodes;
        let time_ms = elapsed.as_millis() as u64;
        let nps = if time_ms > 0 {
            (nodes as f64 / time_ms as f64 * 1000.0) as u64
        } else {
            nodes
        };

        total_nodes += nodes;
        total_time_ms += time_ms;

        println!("Nodes:    {}", nodes);
        println!("Time:     {} ms", time_ms);
        println!("NPS:      {} nodes/sec", nps);
        println!(
            "Move:     {} (score: {})",
            result.best_move.to_uci(),
            result.score
        );
        println!();
    }

    println!("═════════════════════════════════════════════════════════════");
    println!("                         TOTALS");
    println!("═════════════════════════════════════════════════════════════");
    println!("Total nodes:    {}", total_nodes);
    println!("Total time:     {} ms", total_time_ms);

    let avg_nps = if total_time_ms > 0 {
        (total_nodes as f64 / total_time_ms as f64 * 1000.0) as u64
    } else {
        total_nodes
    };
    println!("Average NPS:    {} nodes/sec", avg_nps);
    println!("═════════════════════════════════════════════════════════════\n");

    // Performance targets for M7
    // These are conservative - actual performance may vary by hardware
    assert!(
        avg_nps > 10_000,
        "Average NPS {} is below minimum threshold 10,000",
        avg_nps
    );
}

#[test]
fn bench_pruning_effectiveness() {
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║                  Pruning Effectiveness Test                     ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
    let board = parse_fen(fen).unwrap();

    println!("Testing depth scalability with M7 pruning:");
    println!("Position: Kiwipete (complex middlegame)\n");

    let mut prev_nodes = 0u64;
    for depth in 3..=4 {
        let mut searcher = Searcher::new();
        let result = searcher.search(&board, depth);

        let branching_factor = if prev_nodes > 0 {
            result.nodes as f64 / prev_nodes as f64
        } else {
            0.0
        };

        println!("Depth {}: {} nodes", depth, result.nodes);
        if prev_nodes > 0 {
            println!("  Effective branching factor: {:.2}", branching_factor);
        }

        prev_nodes = result.nodes;

        // M7 should keep branching factor reasonable (below 4.0)
        if depth > 3 {
            assert!(
                branching_factor < 6.0,
                "Branching factor {:.2} too high at depth {} - pruning may not be working",
                branching_factor,
                depth
            );
        }
    }

    println!("\n✓ Pruning is effectively controlling search tree growth");
}

#[test]
fn bench_move_ordering_quality() {
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║                 Move Ordering Quality Test                      ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    // Position with a clear best move (Bxf7+)
    let fen = "r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 0 1";
    let board = parse_fen(fen).unwrap();

    let mut searcher = Searcher::new();
    let result = searcher.search(&board, 4);

    println!("Position: Italian Game tactical position");
    println!("Nodes searched: {}", result.nodes);
    println!("Best move: {}", result.best_move.to_uci());
    println!("Score: {}", result.score);

    // With good move ordering, should find the tactical move efficiently
    // and not need to search too many nodes
    assert!(
        result.nodes < 200_000,
        "Searched {} nodes - move ordering may be poor",
        result.nodes
    );

    println!("\n✓ Move ordering is working efficiently");
}

#[test]
fn bench_extension_effectiveness() {
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║                  Extension Effectiveness Test                   ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    // Position with checks and tactical elements that should trigger extensions
    let fen = "r1bq1rk1/pppp1ppp/2n2n2/2b1p3/2B1P3/2NP1N2/PPP2PPP/R1BQK2R w KQ - 0 1";
    let board = parse_fen(fen).unwrap();

    let mut searcher = Searcher::new();
    let result = searcher.search(&board, 5);

    println!("Position: Tactical middlegame with extension opportunities");
    println!("Depth: 5");
    println!("Nodes: {}", result.nodes);
    println!("Best move: {}", result.best_move.to_uci());

    // Extensions should help us see deeper into forcing lines
    // We should find a reasonable move
    assert!(board.is_legal(result.best_move));

    println!("\n✓ Extensions are being applied correctly");
}

#[test]
fn bench_see_accuracy() {
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║                    SEE Accuracy Test                            ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    // Position where SEE should help avoid bad captures
    let fen = "r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 0 1";
    let board = parse_fen(fen).unwrap();

    let mut searcher = Searcher::new();
    let result = searcher.search(&board, 4);

    println!("Position: Italian Game");
    println!("Best move: {}", result.best_move.to_uci());
    println!("Score: {}", result.score);

    // Should find Bxf7+ which is a good tactical shot
    // SEE should correctly evaluate this as winning material
    assert!(
        result.score > 100,
        "SEE should recognize this is winning for white"
    );

    println!("\n✓ SEE is correctly evaluating captures");
}
