/// Debug why the engine thinks Ng5 is good
use engine::board::Board;
use engine::io::{parse_fen, ToFen};
use engine::movegen::generate_moves;
use engine::search::core::Searcher;
use engine::eval::Evaluator;

fn main() {
    let mut searcher = Searcher::new();
    let mut evaluator = Evaluator::new();

    // Position after 1. Nf3 d5 2. Ng5
    let fen = "rnbqkbnr/ppp1pppp/8/3p2N1/8/8/PPPPPPPP/RNBQKB1R b KQkq - 1 2";
    let board = parse_fen(fen).expect("Valid FEN");

    println!("Position after: 1. Nf3 d5 2. Ng5");
    println!("FEN: {}", fen);
    println!();

    // Static evaluation
    let static_eval = evaluator.evaluate(&board);
    println!("Static evaluation (Black to move): {:.2} pawns", static_eval);
    println!();

    // Search from Black's perspective
    println!("Searching at depth 8 (Black to move)...");
    let result = searcher.search(&board, 8);

    println!("\nSearch result:");
    println!("  Best move: {}", result.best_move.to_uci());
    println!("  Score: {} cp (from Black's perspective)", result.score);
    println!("  Depth: {}", result.depth);
    println!("  Nodes: {}", result.nodes);
    println!("  PV: {}", result.pv.iter().map(|m| m.to_uci()).collect::<Vec<_>>().join(" "));
    println!();

    // Show what Black can do
    let moves = generate_moves(&board);
    println!("Black has {} legal moves", moves.len());
    println!();

    // Check specific responses
    let candidates = vec![
        ("e7e6", "e6 (block)"),
        ("e7e5", "e5 (attack center)"),
        ("h7h6", "h6 (kick knight)"),
        ("f7f6", "f6 (kick knight)"),
        ("c7c6", "c6 (support d5)"),
    ];

    println!("Analyzing Black's candidate responses:");
    for (uci, desc) in candidates {
        if let Some(&mv) = moves.iter().find(|m| m.to_uci() == uci) {
            let mut new_board = board.clone();
            new_board.make_move(mv);

            // Search White's response
            let white_result = searcher.search(&new_board, 7);
            let black_score = -white_result.score;

            println!("  {:6} ({}): {:+5} cp for Black", uci, desc, black_score);
        }
    }

    println!();
    println!("Now let's trace back: what did White think would happen?");
    println!();

    // Go back to the position before Ng5
    let before_fen = "rnbqkbnr/ppp1pppp/8/3p4/8/5N2/PPPPPPPP/RNBQKB1R w KQkq d6 0 2";
    let before_board = parse_fen(before_fen).expect("Valid FEN");

    println!("Position: 1. Nf3 d5 (White to move)");

    // Search Ng5 specifically
    let moves_before = generate_moves(&before_board);
    let ng5_move = moves_before.iter().find(|m| m.to_uci() == "f3g5").copied();
    if let Some(ng5) = ng5_move {
        let mut after_ng5 = before_board.clone();
        after_ng5.make_move(ng5);

        // Search from Black's perspective
        let black_response = searcher.search(&after_ng5, 7);

        println!("After 2. Ng5, Black's best response:");
        println!("  Move: {}", black_response.best_move.to_uci());
        println!("  Score: {} cp (from Black's view)", black_response.score);
        println!();

        // Apply Black's response and see White's position
        after_ng5.make_move(black_response.best_move);
        println!("After Black's response: {}", after_ng5.to_fen());

        let white_eval_after = evaluator.evaluate(&after_ng5);
        println!("Evaluation (White to move): {:.2} pawns", white_eval_after);
        println!();

        // Search White's next move
        let white_next = searcher.search(&after_ng5, 6);
        println!("White's next best move: {}", white_next.best_move.to_uci());
        println!("Score: {} cp", white_next.score);
    }
}
