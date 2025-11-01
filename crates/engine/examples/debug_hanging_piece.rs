use engine::io::parse_fen;
use engine::search::Searcher;

fn main() {
    println!("=== DEBUGGING HANGING KNIGHT POSITION ===\n");

    // The problematic position where engine missed Nxd4
    let fen = "rnbqkb1r/pppppppp/5n2/8/3N4/8/PPPPPPPP/RNBQKB1R b KQkq - 0 1";
    println!("FEN: {}", fen);
    println!("White has knight on d4 that is COMPLETELY undefended");
    println!("Black should play f6xd4 winning the knight (~+300cp advantage)\n");

    let board = parse_fen(fen).unwrap();

    // Test at different depths to see when/if it finds Nxd4
    for depth in [4, 6, 8, 10, 12] {
        println!("╔════════════════════════════════════════════════════════════╗");
        println!("║ DEPTH: {}", depth);
        println!("╚════════════════════════════════════════════════════════════╝");

        let mut searcher = Searcher::new();
        let result = searcher.search(&board, depth);

        println!("Best move: {}", result.best_move.to_uci());
        println!("Score: {} cp", result.score);
        println!("PV: {}", result.pv.iter().map(|m| m.to_uci()).collect::<Vec<_>>().join(" "));
        println!("Nodes: {}", result.nodes);

        if result.best_move.to_uci() == "f6d4" {
            println!("✅ Found Nxd4!\n");
        } else {
            println!("❌ Missed Nxd4 - played {} instead\n", result.best_move.to_uci());
        }
    }

    // Now test what score the engine gives to Nxd4 specifically
    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║ EVALUATING POSITION AFTER Nxd4");
    println!("╚════════════════════════════════════════════════════════════╝");

    let fen_after = "rnbqkb1r/pppppppp/8/8/3n4/8/PPPPPPPP/RNBQKB1R w KQkq - 0 1";
    let board_after = parse_fen(fen_after).unwrap();

    let mut searcher_after = Searcher::new();
    let result_after = searcher_after.search(&board_after, 6);

    println!("Position after Nxd4:");
    println!("Score for Black: {} cp", result_after.score);
    println!("Expected: ~+300cp for Black (winning a knight)");

    if result_after.score > 250 {
        println!("✅ Evaluation looks correct (+300cp range)");
    } else {
        println!("❌ Evaluation is WRONG - should be ~+300cp but got {} cp", result_after.score);
    }

    // Check all legal moves in the position
    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║ ALL LEGAL MOVES FOR BLACK");
    println!("╚════════════════════════════════════════════════════════════╝");

    let moves = board.generate_legal_moves();
    println!("Total legal moves: {}", moves.len());

    // Check if Nxd4 is even in the list
    let has_nxd4 = moves.iter().any(|m| m.to_uci() == "f6d4");
    println!("Is f6xd4 in legal moves: {}", has_nxd4);

    if !has_nxd4 {
        println!("❌ CRITICAL BUG: Nxd4 is not even in the legal move list!");
    }

    // Generate all moves and show their from/to
    println!("\nAll moves from f6:");
    for m in moves.iter() {
        if m.from().to_string() == "f6" {
            println!("  - {}", m.to_uci());
        }
    }
}
