/// Test if extensions are working
use engine::board::Board;
use engine::io::parse_fen;
use engine::search::Searcher;

fn main() {
    println!("Testing if search extensions are working...\n");

    // Position with check - should extend
    let fen = "rnbqkbnr/pppp1ppp/8/4p3/6P1/5P2/PPPPP2P/RNBQKBNR b KQkq - 0 2";
    let board = parse_fen(fen).unwrap();

    println!("FEN: {}", fen);
    println!("(Black can give check with Qh4+)\n");

    let mut searcher = Searcher::new();

    // Search at depth 4
    let result = searcher.search(&board, 4);

    println!("Best move: {}", result.best_move.to_uci());
    println!("Score: {}", result.score);
    println!("Nodes: {}", result.nodes);
    println!("Depth: {}", result.depth);

    // If extensions are working, checks should be explored more
    println!("\nIf Qh4+ is found, extensions are helping");
}
