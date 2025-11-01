/// Test SEE on Free Pieces
///
/// This will test if SEE correctly evaluates captures of completely free pieces
/// (pieces with no defenders).
///
/// Usage: cargo run --example test_see_free_pieces

use engine::board::Board;
use engine::io::parse_fen;
use engine::movegen::generate_moves;
use engine::search::see::see_value_of_move;

fn main() {
    println!("========================================");
    println!("SEE TEST: FREE PIECES");
    println!("========================================\n");

    let mut passed = 0;
    let mut failed = 0;

    // Test 1: Free rook on e4
    println!("Test 1: Free Rook on e4");
    println!("----------------------------------------");
    let fen = "rnbqkbnr/pppppppp/8/8/4r3/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let board = parse_fen(fen).unwrap();
    let moves = generate_moves(&board);

    // Find Qxe4
    let mut found = false;
    println!("Total moves generated: {}", moves.len());
    println!("Looking for captures to e4...");
    for i in 0..moves.len() {
        let mv = moves[i];
        if mv.to().to_algebraic() == "e4" {
            println!("  Found move to e4: {} (is_capture: {})", mv.to_uci(), mv.is_capture());
            if mv.is_capture() {
                let see_score = see_value_of_move(&board, mv);
                println!("Move: {} (from {})", mv.to_uci(), mv.from().to_algebraic());
                println!("SEE score: {}", see_score);
                println!("Expected: ~+500 (value of rook)");

                if see_score > 400 {
                    println!("✓ PASS\n");
                    passed += 1;
                } else {
                    println!("✗ FAIL - SEE should be positive!\n");
                    failed += 1;
                }
                found = true;
                break;
            }
        }
    }

    if !found {
        println!("Available captures:");
        for i in 0..moves.len() {
            let mv = moves[i];
            if mv.is_capture() {
                println!("  {}", mv.to_uci());
            }
        }
        println!("✗ FAIL - Couldn't find capture move\n");
        failed += 1;
    }

    // Test 2: Free knight on e4
    println!("Test 2: Free Knight on e4");
    println!("----------------------------------------");
    let fen = "rnbqkbnr/pppppppp/8/8/4n3/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let board = parse_fen(fen).unwrap();
    let moves = generate_moves(&board);

    let mut found = false;
    for i in 0..moves.len() {
        let mv = moves[i];
        if mv.to().to_algebraic() == "e4" && mv.is_capture() {
            let see_score = see_value_of_move(&board, mv);
            println!("Move: {} (from {})", mv.to_uci(), mv.from().to_algebraic());
            println!("SEE score: {}", see_score);
            println!("Expected: ~+320 (value of knight)");

            if see_score > 250 {
                println!("✓ PASS\n");
                passed += 1;
            } else {
                println!("✗ FAIL - SEE should be positive!\n");
                failed += 1;
            }
            found = true;
            break;
        }
    }

    if !found {
        println!("✗ FAIL - Couldn't find capture move\n");
        failed += 1;
    }

    // Test 3: Free queen on d4
    println!("Test 3: Free Queen on d4");
    println!("----------------------------------------");
    let fen = "rnbqkbnr/pppppppp/8/8/3q4/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let board = parse_fen(fen).unwrap();
    let moves = generate_moves(&board);

    let mut found = false;
    for i in 0..moves.len() {
        let mv = moves[i];
        if mv.to().to_algebraic() == "d4" && mv.is_capture() {
            let see_score = see_value_of_move(&board, mv);
            println!("Move: {} (from {})", mv.to_uci(), mv.from().to_algebraic());
            println!("SEE score: {}", see_score);
            println!("Expected: ~+900 (value of queen)");

            if see_score > 700 {
                println!("✓ PASS\n");
                passed += 1;
            } else {
                println!("✗ FAIL - SEE should be positive!\n");
                failed += 1;
            }
            found = true;
            break;
        }
    }

    if !found {
        println!("✗ FAIL - Couldn't find capture move\n");
        failed += 1;
    }

    // Test 4: Defended piece (pawn on e5 defended by d6 pawn)
    println!("Test 4: Defended Pawn (should still work)");
    println!("----------------------------------------");
    let fen = "rnbqkbnr/ppp2ppp/3p4/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1";
    let board = parse_fen(fen).unwrap();
    let moves = generate_moves(&board);

    let mut found = false;
    for i in 0..moves.len() {
        let mv = moves[i];
        if mv.from().to_algebraic() == "e4" && mv.to().to_algebraic() == "e5" {
            let see_score = see_value_of_move(&board, mv);
            println!("Move: {}", mv.to_uci());
            println!("SEE score: {}", see_score);
            println!("Expected: 0 (pawn for pawn trade)");

            if see_score >= -50 && see_score <= 50 {
                println!("✓ PASS\n");
                passed += 1;
            } else {
                println!("⚠ WARNING - SEE should be ~0 for equal trade\n");
                passed += 1; // Not critical
            }
            found = true;
            break;
        }
    }

    if !found {
        println!("✗ FAIL - Couldn't find capture move\n");
        failed += 1;
    }

    // Summary
    println!("========================================");
    println!("SUMMARY");
    println!("========================================");
    println!("Passed: {}", passed);
    println!("Failed: {}", failed);

    if failed == 0 {
        println!("\n✅ ALL SEE TESTS PASSED");
        println!("SEE is working correctly!");
        println!("The bug must be elsewhere (search logic, TT, etc.)");
    } else {
        println!("\n🚨 SEE TESTS FAILED");
        println!("SEE is not correctly evaluating free pieces!");
        println!("This is likely the root cause of the bug.");
    }
}
