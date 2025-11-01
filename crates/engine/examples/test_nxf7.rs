/// Test what happens after Nxf7 in the d4 line
use engine::io::parse_fen;
use engine::search::Searcher;
use engine::eval::{Evaluator, evaluate_material};
use engine::piece::Color;

fn main() {
    println!("\n=== Testing Nxf7 sacrifice ===\n");

    // After 1.Nf3 d5 2.Ng5 d4 3.Nxf7
    let fen = "rnbqkbnr/ppp1pNpp/8/8/3p4/8/PPPPPPPP/RNBQKB1R b KQkq - 0 3";
    println!("After 3.Nxf7 (Black to move):");
    println!("FEN: {}", fen);
    println!();

    let board = parse_fen(fen).expect("Valid FEN");

    // Material count
    let white_mat = evaluate_material(&board, Color::White);
    let black_mat = evaluate_material(&board, Color::Black);

    println!("Material:");
    println!("  White: {} cp", white_mat);
    println!("  Black: {} cp", black_mat);
    println!("  Diff: {:+} cp ({:+.2} pawns)", white_mat - black_mat, (white_mat - black_mat) as f64 / 100.0);
    println!();

    println!("White has captured a pawn (f7), gaining +100cp");
    println!("But the knight on f7 can be captured!");
    println!();

    // What does the engine think Black should do?
    let mut searcher = Searcher::new();
    println!("Searching at depth 6...");
    let result = searcher.search(&board, 6);

    println!();
    println!("Black's best move: {}", result.best_move);
    println!("Score: {:+} cp (from Black's perspective)", result.score);
    println!();

    // Check if Black captures the knight
    let mv_str = result.best_move.to_string();
    if mv_str.ends_with("f7") {
        println!("✓ Black captures the knight on f7");
    } else {
        println!("✗ Black does NOT capture the knight!");
        println!("  This is wrong - the knight is hanging");
    }
}
