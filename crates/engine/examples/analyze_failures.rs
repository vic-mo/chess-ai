/// Analyze WAC failures in detail
///
/// For each failed position, this will:
/// 1. Show the FEN and expected move
/// 2. Show what the engine chose and its score
/// 3. Search for the expected move and show its score
/// 4. Compare why the wrong move scored higher

use engine::board::Board;
use engine::io::{load_epd_file, parse_fen};
use engine::movegen::generate_moves;
use engine::r#move::Move;
use engine::search::Searcher;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let file_path = if args.len() > 1 {
        &args[1]
    } else {
        "crates/engine/positions/wacnew.epd"
    };

    let depth = if args.len() > 2 {
        args[2].parse::<u32>().unwrap_or(8)
    } else {
        8
    };

    let limit = if args.len() > 3 {
        args[3].parse::<usize>().unwrap_or(10)
    } else {
        10
    };

    println!("========================================");
    println!("WAC FAILURE ANALYSIS");
    println!("========================================");
    println!("File: {}", file_path);
    println!("Depth: {}", depth);
    println!("Analyzing first {} positions\n", limit);

    let positions = match load_epd_file(file_path) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error loading EPD file: {}", e);
            return;
        }
    };

    let positions = &positions[..limit.min(positions.len())];

    let mut failures = Vec::new();

    for (idx, epd_pos) in positions.iter().enumerate() {
        let board = &epd_pos.board;
        let mut searcher = Searcher::new();

        let result = searcher.search(board, depth);
        let engine_move = result.best_move.to_uci();

        // Check if engine found the right move
        let found_correct = epd_pos.best_moves.iter().any(|expected| {
            let expected_clean = expected.trim_end_matches(|c: char| c == '+' || c == '#');
            engine_move == expected_clean || engine_move == *expected
        });

        if !found_correct {
            failures.push((idx + 1, epd_pos, result));
        }
    }

    println!("Found {} failures out of {} positions\n", failures.len(), positions.len());
    println!("========================================");
    println!("DETAILED FAILURE ANALYSIS");
    println!("========================================\n");

    for (pos_num, epd_pos, result) in failures.iter() {
        println!("Position #{}: {}", pos_num, epd_pos.id);
        println!("----------------------------------------");
        println!("FEN: {}", engine::io::ToFen::to_fen(&epd_pos.board));
        println!("Expected: {}", epd_pos.best_moves.join(" or "));
        println!("Engine chose: {}", result.best_move.to_uci());
        println!("Engine score: {} cp", result.score);
        println!("Nodes: {}", result.nodes);
        println!("Depth: {}", result.depth);

        // Now search specifically for the expected move to see its score
        println!("\nAnalyzing expected move:");
        let moves = generate_moves(&epd_pos.board);

        for expected_uci in &epd_pos.best_moves {
            let expected_clean = expected_uci.trim_end_matches(|c: char| c == '+' || c == '#');

            // Find the expected move in the move list
            let expected_move = moves.iter().find(|m| {
                m.to_uci() == expected_clean || m.to_uci() == *expected_uci
            });

            if let Some(exp_mv) = expected_move {
                // Make the expected move and evaluate
                let mut test_board = epd_pos.board.clone();
                test_board.make_move(*exp_mv);

                // Search from opponent's perspective
                let mut test_searcher = Searcher::new();
                let exp_result = test_searcher.search(&test_board, depth - 1);
                let expected_score = -exp_result.score; // Negate because it's from opponent's view

                println!("  Move: {}", expected_uci);
                println!("  Would score: ~{} cp (after opponent's response)", expected_score);
                println!("  Difference: {} cp worse than engine's choice", result.score - expected_score);

                // Categorize the failure
                if result.score - expected_score > 200 {
                    println!("  Category: EVALUATION - Engine thinks its move is much better");
                } else if result.score - expected_score > 50 {
                    println!("  Category: PRUNING/ORDERING - Expected move not searched deeply enough");
                } else {
                    println!("  Category: MARGINAL - Scores very close, might be acceptable");
                }
            } else {
                println!("  Move: {} - NOT FOUND IN LEGAL MOVES!", expected_uci);
            }
        }

        println!("\n");
    }

    // Summary
    println!("========================================");
    println!("FAILURE SUMMARY");
    println!("========================================");
    println!("Total failures: {}/{}", failures.len(), positions.len());
    println!("\nTo analyze more positions, run:");
    println!("  cargo run --release --example analyze_failures {} {} <limit>", file_path, depth);
}
