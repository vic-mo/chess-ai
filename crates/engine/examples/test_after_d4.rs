/// See what happens after 1.Nf3 d5 2.Ng5 d4
use engine::io::parse_fen;
use engine::search::Searcher;

fn main() {
    println!("\n=== After 1.Nf3 d5 2.Ng5 d4 ===\n");

    let fen = "rnbqkbnr/ppp1pppp/8/6N1/3p4/8/PPPPPPPP/RNBQKB1R w KQkq - 0 3";
    println!("Position: White to move");
    println!("FEN: {}", fen);
    println!();

    let board = parse_fen(fen).expect("Valid FEN");
    let mut searcher = Searcher::new();

    println!("Searching at depth 6...");
    let result = searcher.search(&board, 6);

    println!();
    println!("Best move: {}", result.best_move);
    println!("Score: {:+} cp (from White's perspective)", result.score);
    println!("Nodes: {}", result.nodes);
    println!();

    println!("Stockfish evaluation:");
    println!("  Stockfish says this position is about equal");
    println!("  White's knight on g5 is somewhat out of play");
    println!("  Black's d4 pawn is advanced and controls center");
    println!();

    println!("Our engine thinks:");
    if result.score > 400 {
        println!("  White is WINNING (+{} cp = +{:.2} pawns)", result.score, result.score as f64 / 100.0);
        println!("  This is likely WRONG!");
    } else if result.score > 100 {
        println!("  White has an advantage");
    } else if result.score > -100 {
        println!("  Position is roughly equal");
    } else {
        println!("  Black has an advantage");
    }
}
