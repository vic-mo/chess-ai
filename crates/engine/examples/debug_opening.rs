/// Debug opening move selection issue
use engine::board::Board;
use engine::io::parse_fen;
use engine::movegen::generate_moves;
use engine::search::core::Searcher;
use engine::eval::Evaluator;

fn main() {
    let mut searcher = Searcher::new();

    // Test position after 1. Nf3 d5
    let fen = "rnbqkbnr/ppp1pppp/8/3p4/8/5N2/PPPPPPPP/RNBQKB1R w KQkq d6 0 2";
    let board = parse_fen(fen).expect("Valid FEN");

    println!("Position: 1. Nf3 d5");
    println!("FEN: {}", fen);
    println!();

    // Generate all legal moves
    let moves = generate_moves(&board);
    println!("Legal moves: {}", moves.len());

    // Search at depth 8
    println!("\nSearching at depth 8...");
    let result = searcher.search(&board, 8);

    println!("\nSearch result:");
    println!("  Best move: {}", result.best_move.to_uci());
    println!("  Score: {} cp", result.score);
    println!("  Depth: {}", result.depth);
    println!("  Nodes: {}", result.nodes);
    println!();

    // Also search some reasonable moves and compare
    println!("Comparing candidate moves:");
    println!("  (searching each to depth 7)\n");

    let candidates = vec![
        ("d2d4", "d4 (control center)"),
        ("c2c4", "c4 (Queen's Gambit)"),
        ("g2g3", "g3 (King's Fianchetto)"),
        ("f3g5", "Ng5 (engine choice)"),
        ("b1c3", "Nc3 (develop knight)"),
    ];

    let mut evaluator = Evaluator::new();

    for (uci, desc) in candidates {
        // Find the move
        let move_option = moves.iter().find(|m| m.to_uci() == uci);

        if let Some(&mv) = move_option {
            let mut new_board = board.clone();
            new_board.make_move(mv);

            // Search from opponent's perspective
            let opp_result = searcher.search(&new_board, 7);

            // The score from opponent's perspective (negated is our score)
            let our_score = -opp_result.score;

            println!("  {:6} ({}): {:+5} cp", uci, desc, our_score);
        } else {
            println!("  {:6} ({}): ILLEGAL", uci, desc);
        }
    }

    println!();
    println!("Static evaluation of starting position:");
    let static_eval = evaluator.evaluate(&board);
    println!("  {:.2} pawns", static_eval);
}
