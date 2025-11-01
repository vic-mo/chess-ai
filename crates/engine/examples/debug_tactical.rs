use engine::io::parse_fen;
use engine::search::Searcher;

fn main() {
    println!("=== TACTICAL DEBUGGING ===\n");

    // Test 1: Simple hanging piece - don't move into attack
    println!("Test 1: Don't move knight into pawn attack");
    println!("----------------------------------------");
    let fen1 = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let board1 = parse_fen(fen1).unwrap();

    let mut searcher1 = Searcher::new();
    let result1 = searcher1.search(&board1, 8);

    println!("FEN: {}", fen1);
    println!("Best move: {}", result1.best_move.to_uci());
    println!("Score: {} cp", result1.score);
    println!("PV: {}", result1.pv.iter().map(|m| m.to_uci()).collect::<Vec<_>>().join(" "));

    // Check if engine hangs knight on c3 attacked by d5 pawn
    let bad_moves = vec!["b1a3", "g1h3"]; // Knights to rim
    if bad_moves.contains(&result1.best_move.to_uci().as_str()) {
        println!("⚠️  WARNING: Engine played knight to rim\n");
    } else {
        println!("✓ Played reasonable opening move\n");
    }

    // Test 2: Position where engine can hang queen or play normal move
    println!("\nTest 2: Don't hang the queen");
    println!("----------------------------------------");
    // Position: White queen on e2, black has control of d3, e3, f3
    // Engine should NOT play Qe3 (hangs to pawn)
    let fen2 = "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPPQPPP/RNB1KBNR w KQkq - 0 1";
    let board2 = parse_fen(fen2).unwrap();

    let mut searcher2 = Searcher::new();
    let result2 = searcher2.search(&board2, 8);

    println!("FEN: {}", fen2);
    println!("Best move: {}", result2.best_move.to_uci());
    println!("Score: {} cp", result2.score);
    println!("PV: {}", result2.pv.iter().map(|m| m.to_uci()).collect::<Vec<_>>().join(" "));

    // Queen on e2 can go to e3 (hangs to d4 pawn if there was one), or many safe squares
    if result2.best_move.to_uci().starts_with("e2") {
        println!("⚠️  Engine wants to move queen from e2 (risky)\n");
    } else {
        println!("✓ Engine developed other pieces\n");
    }

    // Test 3: Actual hanging piece position
    println!("\nTest 3: Position with knight that can be captured for free");
    println!("----------------------------------------");
    // Black has knight on d4, white can take it
    let fen3 = "rnbqkb1r/pppppppp/5n2/8/3N4/8/PPPPPPPP/RNBQKB1R b KQkq - 0 1";
    let board3 = parse_fen(fen3).unwrap();

    let mut searcher3 = Searcher::new();
    let result3 = searcher3.search(&board3, 8);

    println!("FEN: {}", fen3);
    println!("Best move: {}", result3.best_move.to_uci());
    println!("Score: {} cp (positive = good for black)", result3.score);
    println!("PV: {}", result3.pv.iter().map(|m| m.to_uci()).collect::<Vec<_>>().join(" "));

    // Black should NOT move the knight to a square where it hangs
    // It should either defend the knight or attack something
    if result3.best_move.to().to_string() != "d4" && !result3.best_move.to_uci().starts_with("f6") {
        println!("⚠️  Engine didn't address the hanging knight on d4\n");
    } else {
        println!("✓ Engine dealt with the threat\n");
    }

    // Test 4: Scholar's mate trap - don't take f7
    println!("\nTest 4: Scholar's mate trap - don't take f7");
    println!("----------------------------------------");
    // After 1.e4 e5 2.Bc4 Nc6 3.Qh5 Nf6?? 4.Qxf7# is mate
    // Engine playing Black should NOT allow Qxf7
    let fen4 = "r1bqkb1r/pppp1ppp/2n2n2/4p2Q/2B1P3/8/PPPP1PPP/RNB1K1NR w KQkq - 0 1";
    let board4 = parse_fen(fen4).unwrap();

    let mut searcher4 = Searcher::new();
    let result4 = searcher4.search(&board4, 8);

    println!("FEN: {}", fen4);
    println!("Best move: {}", result4.best_move.to_uci());
    println!("Score: {} cp", result4.score);
    println!("PV: {}", result4.pv.iter().map(|m| m.to_uci()).collect::<Vec<_>>().join(" "));

    if result4.best_move.to_uci() == "h5f7" {
        println!("✓ Engine sees the mate! (Qxf7#)\n");
        if result4.score > 25000 {
            println!("✓ Engine recognizes it as mate (score > 25000)\n");
        } else {
            println!("⚠️  Engine plays mate but doesn't score it as mate: {} cp\n", result4.score);
        }
    } else {
        println!("✗ Engine missed the mate in 1\n");
    }

    // Test 5: Don't sacrifice piece for no compensation
    println!("\nTest 5: Don't sacrifice knight for 1 pawn");
    println!("----------------------------------------");
    // White to move - has Nxe5 winning a pawn, or Nxf7 winning 1 pawn but losing knight
    let fen5 = "rnbqkb1r/pppp1ppp/5n2/4p3/4P3/3P1N2/PPP2PPP/RNBQKB1R w KQkq - 0 1";
    let board5 = parse_fen(fen5).unwrap();

    let mut searcher5 = Searcher::new();
    let result5 = searcher5.search(&board5, 8);

    println!("FEN: {}", fen5);
    println!("Best move: {}", result5.best_move.to_uci());
    println!("Score: {} cp", result5.score);
    println!("PV: {}", result5.pv.iter().map(|m| m.to_uci()).collect::<Vec<_>>().join(" "));

    if result5.best_move.to_uci() == "f3e5" {
        println!("✓ Engine takes the free pawn (Nxe5)\n");
    } else if result5.best_move.to_uci() == "f3f7" {
        println!("✗ Engine sacrificed knight for 1 pawn (Nxf7 - bad!)\n");
    } else {
        println!("  Engine played different move\n");
    }

    println!("\n=== SUMMARY ===");
    println!("Check the results above to see if engine is making tactical errors");
}
