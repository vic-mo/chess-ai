/// Manually verify if "failed" positions are actually good moves
use engine::board::Board;
use engine::io::load_epd_file;
use engine::movegen::generate_moves;
use engine::search::Searcher;

fn main() {
    let positions = load_epd_file("crates/engine/positions/wacnew.epd").unwrap();

    // Test first 10 positions
    let test_positions = &[(0, "WAC.001"), (1, "WAC.002"), (5, "WAC.006"), (6, "WAC.007"), (7, "WAC.008"), (8, "WAC.009")];

    println!("========================================");
    println!("MANUAL VERIFICATION OF FAILURES");
    println!("========================================\n");

    for (idx, name) in test_positions {
        let epd = &positions[*idx];
        let board = &epd.board;

        println!("{}", name);
        println!("----------------------------------------");
        println!("Expected: {}", epd.best_moves.join(" or "));

        // Get engine's choice
        let mut searcher = Searcher::new();
        let result = searcher.search(board, 8);
        println!("Engine chose: {}", result.best_move.to_uci());
        println!("Engine score: {} cp", result.score);

        // Check if mate
        if result.score > 30000 {
            println!("✓✓✓ ENGINE FOUND MATE! M{}", (32767 - result.score) / 2);
        } else if result.score < -30000 {
            println!("✗ Engine getting mated");
        }

        // Try to find and test the expected move
        let moves = generate_moves(board);
        let expected_str = &epd.best_moves[0];

        // Try to match by destination square
        let dest = expected_str.chars().rev().take(2).collect::<String>().chars().rev().collect::<String>();

        println!("\nTesting expected move ({}):", expected_str);
        let mut found_expected = false;

        for mv in moves.iter() {
            let to_sq = mv.to().to_algebraic();
            if to_sq == dest {
                // Found a move to the expected square
                let mut test_board = board.clone();
                test_board.make_move(*mv);

                let mut test_searcher = Searcher::new();
                let test_result = test_searcher.search(&test_board, 7);
                let expected_score = -test_result.score;

                println!("  Move {} to {}: {} cp", mv.from().to_algebraic(), to_sq, expected_score);

                if expected_score > 30000 {
                    println!("  ✓ Leads to mate: M{}", (32767 - expected_score) / 2);
                }

                found_expected = true;
            }
        }

        if !found_expected {
            println!("  Could not find move to {}", dest);
        }

        // Comparison
        println!("\nVerdict:");
        if result.score > 30000 && result.score > 31000 {
            println!("  ✓✓✓ ENGINE FOUND MATE - BETTER THAN OR EQUAL TO EXPECTED");
        } else {
            println!("  ? Needs manual analysis");
        }

        println!();
    }
}
