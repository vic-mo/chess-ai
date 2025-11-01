/// Debug WAC.271 to understand the "failure"
use engine::io::parse_fen;
use engine::movegen::generate_moves;
use engine::search::Searcher;
use engine::piece::PieceType;

fn main() {
    let fen = "2kr4/ppp3Pp/4RP1B/2r5/5P2/1P6/P2p4/3K4 w - - 0 1";
    println!("WAC.271");
    println!("FEN: {}", fen);
    println!("Expected: Rd6\n");

    let board = parse_fen(fen).unwrap();

    // Show all rook moves
    let moves = generate_moves(&board);
    println!("All rook moves:");
    for mv in moves.iter() {
        if let Some(piece) = board.piece_at(mv.from()) {
            if piece.piece_type == PieceType::Rook {
                println!("  {} (from {})", mv.to_uci(), mv.from().to_algebraic());
            }
        }
    }

    // Search
    println!("\nEngine search at depth 8:");
    let mut searcher = Searcher::new();
    let result = searcher.search(&board, 8);
    println!("Best move: {}", result.best_move.to_uci());
    println!("Score: {} cp", result.score);

    // Test e6d6
    println!("\nTesting e6d6:");
    let e6d6 = moves.iter().find(|m| m.to_uci() == "e6d6");
    if let Some(mv) = e6d6 {
        let mut test_board = board.clone();
        test_board.make_move(*mv);

        let mut test_searcher = Searcher::new();
        let test_result = test_searcher.search(&test_board, 7);
        let score = -test_result.score;

        println!("e6d6 scores: {} cp", score);
    }

    // Test f6d6
    println!("\nTesting f6d6:");
    let f6d6 = moves.iter().find(|m| m.to_uci() == "f6d6");
    if let Some(mv) = f6d6 {
        let mut test_board = board.clone();
        test_board.make_move(*mv);

        let mut test_searcher = Searcher::new();
        let test_result = test_searcher.search(&test_board, 7);
        let score = -test_result.score;

        println!("f6d6 scores: {} cp", score);
    }
}
