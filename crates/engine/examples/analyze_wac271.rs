/// Deep analysis of WAC.271 to understand why it fails
use engine::io::parse_fen;
use engine::movegen::generate_moves;
use engine::search::Searcher;
use engine::square::Square;

fn main() {
    let fen = "2kr4/ppp3Pp/4RP1B/2r5/5P2/1P6/P2p4/3K4 w - - 0 1";

    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║              WAC.271 FAILURE ANALYSIS                      ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    println!("FEN: {}", fen);
    println!("Expected: Rd6");
    println!("\nNote: White is already losing badly in this position.");
    println!("This is a defensive/resourceful move test.\n");

    let board = parse_fen(fen).unwrap();

    // Find all legal moves and show top candidates
    let moves = generate_moves(&board);

    println!("═══════════════════════════════════════════════════════════");
    println!("ANALYZING ALL CANDIDATE MOVES");
    println!("═══════════════════════════════════════════════════════════\n");

    let mut move_scores = Vec::new();

    for mv in moves.iter() {
        let mut test_board = board.clone();
        test_board.make_move(*mv);

        let mut test_searcher = Searcher::new();
        let test_result = test_searcher.search(&test_board, 7);
        let score = -test_result.score;

        move_scores.push((mv.to_uci(), score, mv.from().to_algebraic(), mv.to().to_algebraic()));
    }

    // Sort by score (best first)
    move_scores.sort_by(|a, b| b.1.cmp(&a.1));

    println!("Top 10 moves by evaluation:\n");
    for (i, (uci, score, from, to)) in move_scores.iter().take(10).enumerate() {
        let from_sq = Square::from_algebraic(from).unwrap();
        let piece = board.piece_at(from_sq).unwrap();
        let piece_name = format!("{:?}", piece.piece_type);

        println!("{}. {} ({} {}→{}) : {} cp",
            i + 1,
            uci,
            piece_name,
            from,
            to,
            score
        );

        // Highlight specific moves
        if to == "d6" {
            println!("   ← Move to d6 (expected square)");
        }
        if uci == "e6d6" {
            println!("   ← ENGINE'S CHOICE");
        }
    }

    println!("\n═══════════════════════════════════════════════════════════");
    println!("DETAILED COMPARISON");
    println!("═══════════════════════════════════════════════════════════\n");

    // Engine's choice
    println!("Engine's choice: e6d6");
    let engine_score = move_scores.iter().find(|(uci, _, _, _)| uci == "e6d6").map(|(_, s, _, _)| s);
    if let Some(score) = engine_score {
        println!("  Score: {} cp", score);
        println!("  Piece: Rook (e6 → d6)");
    }

    // Find best move(s)
    let best_score = move_scores[0].1;
    println!("\nBest move(s) according to search:");
    for (uci, score, from, to) in move_scores.iter().take(5) {
        if *score == best_score {
            let from_sq = Square::from_algebraic(&from).unwrap();
            let piece = board.piece_at(from_sq).unwrap();
            println!("  {} ({:?} {}→{}) : {} cp", uci, piece.piece_type, from, to, score);
        }
    }

    // Check if there's another piece that can go to d6
    println!("\nAll moves to d6:");
    let d6_moves: Vec<_> = move_scores.iter()
        .filter(|(_, _, _, to)| to == "d6")
        .collect();

    for (uci, score, from, to) in d6_moves.iter() {
        let from_sq = Square::from_algebraic(&from).unwrap();
        let piece = board.piece_at(from_sq).unwrap();
        println!("  {} ({:?} {}→{}) : {} cp", uci, piece.piece_type, from, to, score);

        if uci == "e6d6" {
            println!("    ← Engine chose this one");
        }
    }

    println!("\n═══════════════════════════════════════════════════════════");
    println!("ANALYSIS");
    println!("═══════════════════════════════════════════════════════════\n");

    if let Some(best) = move_scores.first() {
        if let Some(engine) = engine_score {
            let diff = best.1 - engine;

            println!("Score difference: {} cp", diff);
            println!("Best move: {}", best.0);
            println!("Engine move: e6d6");

            if diff > 300 {
                println!("\nConclusion: SIGNIFICANT ERROR");
                println!("The engine is missing a key tactical idea.");
                println!("\nPossible causes:");
                println!("  1. Search depth insufficient (only depth 8)");
                println!("  2. Horizon effect - best move requires deeper calculation");
                println!("  3. Evaluation misunderstanding the position");
                println!("  4. Move ordering - best move searched late/pruned");
            } else if diff > 100 {
                println!("\nConclusion: MODERATE ERROR");
                println!("The engine chose a reasonable but suboptimal move.");
            } else {
                println!("\nConclusion: MINOR ERROR");
                println!("The moves are nearly equivalent.");
            }
        }
    }

    println!("\n═══════════════════════════════════════════════════════════");
    println!("DEEPER SEARCH TEST");
    println!("═══════════════════════════════════════════════════════════\n");

    println!("Testing at depth 10 to see if deeper search finds better move:\n");

    let mut deep_searcher = Searcher::new();
    let deep_result = deep_searcher.search(&board, 10);

    println!("Depth 10 result:");
    println!("  Best move: {}", deep_result.best_move.to_uci());
    println!("  Score: {} cp", deep_result.score);
    println!("  Nodes: {}", deep_result.nodes);

    if deep_result.best_move.to_uci() != "e6d6" {
        println!("\n✓ Deeper search finds different move!");
        println!("  This confirms it's a search depth issue.");
    } else {
        println!("\n✗ Even at depth 10, engine still chooses e6d6");
        println!("  This suggests an evaluation or move ordering issue.");
    }

    println!("\n═══════════════════════════════════════════════════════════");
}
