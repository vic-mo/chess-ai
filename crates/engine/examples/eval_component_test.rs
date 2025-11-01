/// Evaluation Component Test
///
/// Tests each component of the evaluation function in isolation
/// to identify where bugs might be hiding.
///
/// This will help diagnose the "missing free pieces" bug by testing:
/// 1. Material counting
/// 2. Position evaluation with free pieces
/// 3. Evaluation symmetry
/// 4. Evaluation after captures
///
/// Usage: cargo run --example eval_component_test

use engine::board::Board;
use engine::eval::Evaluator;
use engine::io::{parse_fen, ToFen};
use engine::piece::Color;
use engine::eval::{evaluate_material, piece_value};
use engine::piece::PieceType;

fn main() {
    println!("========================================");
    println!("EVALUATION COMPONENT TESTS");
    println!("========================================\n");

    let mut total_tests = 0;
    let mut passed_tests = 0;

    // Test 1: Starting position should be roughly equal
    println!("Test 1: Starting Position");
    println!("----------------------------------------");
    total_tests += 1;
    let board = Board::startpos();
    let mut eval = Evaluator::new();
    let score = eval.evaluate(&board);
    println!("FEN: {}", board.to_fen());
    println!("Score: {} cp", score);
    if score.abs() < 100 {
        println!("✓ PASS - Score is roughly equal\n");
        passed_tests += 1;
    } else {
        println!("✗ FAIL - Expected score near 0, got {}\n", score);
    }

    // Test 2: White up a queen
    println!("Test 2: White Up a Queen");
    println!("----------------------------------------");
    total_tests += 1;
    let fen = "rnb1kbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let board = parse_fen(fen).unwrap();
    let score = eval.evaluate(&board);
    println!("FEN: {}", fen);
    println!("Score: {} cp", score);
    println!("Expected: ~+900 cp (White up a queen)");
    if score > 700 {
        println!("✓ PASS - White has large advantage\n");
        passed_tests += 1;
    } else {
        println!("✗ FAIL - Expected score > 700, got {}\n", score);
    }

    // Test 3: Black up a queen
    println!("Test 3: Black Up a Queen");
    println!("----------------------------------------");
    total_tests += 1;
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNB1KBNR b KQkq - 0 1";
    let board = parse_fen(fen).unwrap();
    let score = eval.evaluate(&board);
    println!("FEN: {}", fen);
    println!("Score: {} cp (from Black's perspective)", score);
    println!("Expected: ~+900 cp (Black up a queen, Black to move)");
    if score > 700 {
        println!("✓ PASS - Black has large advantage\n");
        passed_tests += 1;
    } else {
        println!("✗ FAIL - Expected score > 700, got {}\n", score);
    }

    // Test 4: Free rook in center (CRITICAL - from bug report)
    println!("Test 4: FREE ROOK IN CENTER (Critical Test)");
    println!("----------------------------------------");
    total_tests += 1;
    let fen = "rnbqkbnr/pppppppp/8/8/4r3/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let board = parse_fen(fen).unwrap();
    let score = eval.evaluate(&board);
    println!("FEN: {}", fen);
    println!("Black has free rook on e4");
    println!("Score: {} cp (from White's perspective)", score);
    println!("Expected: ~-500 cp (White should be DOWN a rook)");
    if score < -300 {
        println!("✓ PASS - White correctly sees disadvantage\n");
        passed_tests += 1;
    } else {
        println!("✗ FAIL - Expected score < -300, got {}", score);
        println!("🚨 THIS IS THE BUG! Engine should see it's down material!\n");
    }

    // Test 5: Free knight in center (CRITICAL)
    println!("Test 5: FREE KNIGHT IN CENTER (Critical Test)");
    println!("----------------------------------------");
    total_tests += 1;
    let fen = "rnbqkbnr/pppppppp/8/8/4n3/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let board = parse_fen(fen).unwrap();
    let score = eval.evaluate(&board);
    println!("FEN: {}", fen);
    println!("Black has free knight on e4");
    println!("Score: {} cp (from White's perspective)", score);
    println!("Expected: ~-320 cp (White should be DOWN a knight)");
    if score < -200 {
        println!("✓ PASS - White correctly sees disadvantage\n");
        passed_tests += 1;
    } else {
        println!("✗ FAIL - Expected score < -200, got {}", score);
        println!("🚨 THIS IS THE BUG! Engine should see it's down material!\n");
    }

    // Test 6: Free bishop in center (CRITICAL)
    println!("Test 6: FREE BISHOP IN CENTER (Critical Test)");
    println!("----------------------------------------");
    total_tests += 1;
    let fen = "rnbqkbnr/pppppppp/8/8/4b3/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let board = parse_fen(fen).unwrap();
    let score = eval.evaluate(&board);
    println!("FEN: {}", fen);
    println!("Black has free bishop on e4");
    println!("Score: {} cp (from White's perspective)", score);
    println!("Expected: ~-330 cp (White should be DOWN a bishop)");
    if score < -200 {
        println!("✓ PASS - White correctly sees disadvantage\n");
        passed_tests += 1;
    } else {
        println!("✗ FAIL - Expected score < -200, got {}", score);
        println!("🚨 THIS IS THE BUG! Engine should see it's down material!\n");
    }

    // Test 7: Free queen in center (CRITICAL)
    println!("Test 7: FREE QUEEN IN CENTER (Critical Test)");
    println!("----------------------------------------");
    total_tests += 1;
    let fen = "rnbqkbnr/pppppppp/8/8/3q4/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let board = parse_fen(fen).unwrap();
    let score = eval.evaluate(&board);
    println!("FEN: {}", fen);
    println!("Black has free queen on d4");
    println!("Score: {} cp (from White's perspective)", score);
    println!("Expected: ~-900 cp (White should be DOWN a queen)");
    if score < -700 {
        println!("✓ PASS - White correctly sees disadvantage\n");
        passed_tests += 1;
    } else {
        println!("✗ FAIL - Expected score < -700, got {}", score);
        println!("🚨 THIS IS THE BUG! Engine should see it's down material!\n");
    }

    // Test 8: Material counting directly (bypass eval)
    println!("Test 8: Direct Material Counting");
    println!("----------------------------------------");
    total_tests += 1;
    let fen = "rnbqkbnr/pppppppp/8/8/4r3/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let board = parse_fen(fen).unwrap();
    let white_material = evaluate_material(&board, Color::White);
    let black_material = evaluate_material(&board, Color::Black);
    println!("FEN: {}", fen);
    println!("White material: {} cp", white_material);
    println!("Black material: {} cp", black_material);
    println!("Difference: {} cp (Black advantage)", black_material - white_material);
    println!("Expected: ~500 cp (value of rook)");
    let diff = black_material - white_material;
    if diff == 500 {
        println!("✓ PASS - Material counting is correct\n");
        passed_tests += 1;
    } else {
        println!("✗ FAIL - Expected difference of 500, got {}\n", diff);
    }

    // Test 9: Symmetry test
    println!("Test 9: Evaluation Symmetry");
    println!("----------------------------------------");
    total_tests += 1;
    let fen1 = "rnbqkbnr/pppppppp/8/8/4r3/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let fen2 = "rnbqkbnr/pppppppp/8/8/4R3/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1";
    let board1 = parse_fen(fen1).unwrap();
    let board2 = parse_fen(fen2).unwrap();
    let score1 = eval.evaluate(&board1); // White to move, Black up rook
    let score2 = eval.evaluate(&board2); // Black to move, White up rook
    println!("Position 1: Black rook on e4, White to move");
    println!("Score 1: {} cp", score1);
    println!("Position 2: White rook on e4, Black to move");
    println!("Score 2: {} cp", score2);
    println!("Expected: score2 ≈ -score1 (symmetry)");
    if (score1 + score2).abs() < 100 {
        println!("✓ PASS - Evaluation is symmetric\n");
        passed_tests += 1;
    } else {
        println!("✗ FAIL - Expected score2 ≈ -score1, got {} and {}\n", score1, score2);
    }

    // Test 10: Test all piece values directly
    println!("Test 10: Piece Value Constants");
    println!("----------------------------------------");
    total_tests += 1;
    println!("Pawn:   {} cp", piece_value(PieceType::Pawn));
    println!("Knight: {} cp", piece_value(PieceType::Knight));
    println!("Bishop: {} cp", piece_value(PieceType::Bishop));
    println!("Rook:   {} cp", piece_value(PieceType::Rook));
    println!("Queen:  {} cp", piece_value(PieceType::Queen));
    println!("King:   {} cp", piece_value(PieceType::King));
    let expected = [100, 320, 330, 500, 900, 20_000];
    let actual = [
        piece_value(PieceType::Pawn),
        piece_value(PieceType::Knight),
        piece_value(PieceType::Bishop),
        piece_value(PieceType::Rook),
        piece_value(PieceType::Queen),
        piece_value(PieceType::King),
    ];
    if expected == actual {
        println!("✓ PASS - All piece values are correct\n");
        passed_tests += 1;
    } else {
        println!("✗ FAIL - Piece values don't match expected\n");
    }

    // Summary
    println!("\n========================================");
    println!("SUMMARY");
    println!("========================================");
    println!("Total tests:  {}", total_tests);
    println!("Passed:       {} ({:.1}%)", passed_tests, (passed_tests as f64 / total_tests as f64) * 100.0);
    println!("Failed:       {} ({:.1}%)", total_tests - passed_tests, ((total_tests - passed_tests) as f64 / total_tests as f64) * 100.0);
    println!("========================================\n");

    if passed_tests == total_tests {
        println!("✅ ALL TESTS PASSED");
        println!("Evaluation function is working correctly.");
        println!("The bug is likely in:");
        println!("  - Move ordering (captures not prioritized)");
        println!("  - Search logic (alpha-beta or TT issues)");
        println!("  - Quiescence search");
    } else {
        println!("🚨 SOME TESTS FAILED");
        println!("The bug is in the evaluation function.");
        println!("Focus on:");
        println!("  - Tests 4-7 (free pieces evaluation)");
        println!("  - Check if PST or other components override material");
    }
}
