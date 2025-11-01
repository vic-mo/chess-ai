/// Test what responses the engine considers after Ng5
use engine::io::parse_fen;
use engine::search::Searcher;

fn main() {
    println!("\n=== Testing Black's responses to Ng5 ===\n");

    // After 1.Nf3 d5 2.Ng5
    let fen = "rnbqkbnr/ppp1pppp/8/3p2N1/8/8/PPPPPPPP/RNBQKB1R b KQkq - 1 2";
    println!("Position: After 1.Nf3 d5 2.Ng5 (Black to move)");
    println!("FEN: {}", fen);
    println!();

    let board = parse_fen(fen).expect("Valid FEN");
    let mut searcher = Searcher::new();

    println!("Searching at depth 8...");
    let result = searcher.search(&board, 8);

    println!();
    println!("Best move: {}", result.best_move);
    println!("Score: {} cp (from Black's perspective)", result.score);
    println!("Nodes: {}", result.nodes);
    println!();

    // Compare to what Stockfish thinks
    println!("Stockfish's best move: d5d4 (push the pawn)");
    println!("Our engine's best move: {} ({})",
        result.best_move,
        if result.best_move.to_string().starts_with("g8") {
            "developing knight"
        } else if result.best_move.to_string().contains("d4") {
            "pushing pawn - good!"
        } else {
            "something else"
        }
    );
}
