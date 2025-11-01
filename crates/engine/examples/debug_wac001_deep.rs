/// Deep analysis of WAC.001
use engine::board::Board;
use engine::io::parse_fen;
use engine::movegen::generate_moves;
use engine::search::Searcher;

fn main() {
    println!("========================================");
    println!("WAC.001 DEEP ANALYSIS");
    println!("========================================\n");

    let fen = "2rr3k/pp3pp1/1nnqbN1p/3pN3/2pP4/2P3Q1/PPB4P/R4RK1 w - - 0 1";
    println!("FEN: {}", fen);
    println!("Expected: Qg6 (threatens mate)\n");

    let board = parse_fen(fen).unwrap();

    // Generate moves to see what's available
    let moves = generate_moves(&board);
    println!("Legal moves: {}", moves.len());

    // Look for g3 moves (queen is on g3)
    println!("\nQueen moves from g3:");
    for i in 0..moves.len() {
        let mv = moves[i];
        if mv.from().to_algebraic() == "g3" {
            println!("  {} (capture: {})", mv.to_uci(), mv.is_capture());
        }
    }

    // Search and see what engine finds
    println!("\nEngine search at depth 8:");
    let mut searcher = Searcher::new();
    let result = searcher.search(&board, 8);

    println!("Best move: {}", result.best_move.to_uci());
    println!("Score: {} cp", result.score);
    println!("Nodes: {}", result.nodes);

    // Check if this is a mate score
    if result.score > 30000 {
        println!("\n✓ ENGINE FOUND MATE!");
        println!("Mate score: M{}", (32767 - result.score + 1) / 2);
    } else if result.score < -30000 {
        println!("\n✗ Engine is getting mated");
    }

    // Now test Qg6 specifically
    println!("\n----------------------------------------");
    println!("Testing Qg6 (g3g6):");

    let qg6 = moves.iter().find(|m| m.to_uci() == "g3g6");
    if let Some(mv) = qg6 {
        let mut test_board = board.clone();
        test_board.make_move(*mv);

        let mut test_searcher = Searcher::new();
        let test_result = test_searcher.search(&test_board, 7);
        let qg6_score = -test_result.score;

        println!("Qg6 would score: {} cp", qg6_score);

        if qg6_score > 30000 {
            println!("Qg6 leads to mate: M{}", (32767 - qg6_score + 1) / 2);
        }
    } else {
        println!("Qg6 (g3g6) not found in legal moves!");
    }

    // Test g3g7
    println!("\n----------------------------------------");
    println!("Testing g3g7 (engine's choice):");

    let g3g7 = moves.iter().find(|m| m.to_uci() == "g3g7");
    if let Some(mv) = g3g7 {
        println!("Move found: {}", mv.to_uci());
        println!("Is capture: {}", mv.is_capture());

        let mut test_board = board.clone();
        test_board.make_move(*mv);

        println!("After g3g7:");
        println!("  Position: {}", engine::io::ToFen::to_fen(&test_board));
        println!("  In check: {}", test_board.is_in_check());

        // Check if it's checkmate
        let opp_moves = generate_moves(&test_board);
        println!("  Opponent moves: {}", opp_moves.len());

        if opp_moves.len() == 0 {
            println!("\n✓✓✓ IT'S CHECKMATE! ✓✓✓");
            println!("g3g7 is better than Qg6 - it's immediate mate!");
        }
    } else {
        println!("g3g7 not found in legal moves!");
    }
}
