/// Trace search results step by step
use engine::board::Board;
use engine::io::{parse_fen, ToFen};
use engine::movegen::generate_moves;
use engine::search::core::Searcher;

fn main() {
    let mut searcher = Searcher::new();

    println!("=== STARTING POSITION ===");
    let board = Board::default();
    let result = searcher.search(&board, 8);
    println!("Best move: {}", result.best_move.to_uci());
    println!("Score: {} cp", result.score);
    println!();

    println!("=== After 1.Nf3 ===");
    let fen1 = "rnbqkbnr/pppppppp/8/8/8/5N2/PPPPPPPP/RNBQKB1R b KQkq - 1 1";
    let board1 = parse_fen(fen1).unwrap();
    let result1 = searcher.search(&board1, 8);
    println!("Black's best move: {}", result1.best_move.to_uci());
    println!("Score (from Black's view): {} cp", result1.score);
    println!();

    println!("=== After 1.Nf3 d5 (White to move) ===");
    let fen2 = "rnbqkbnr/ppp1pppp/8/3p4/8/5N2/PPPPPPPP/RNBQKB1R w KQkq d6 0 2";
    let board2 = parse_fen(fen2).unwrap();

    // Get all legal moves
    let moves = generate_moves(&board2);
    println!("Legal moves: {}", moves.len());

    let result2 = searcher.search(&board2, 8);
    println!("White's best move: {}", result2.best_move.to_uci());
    println!("Score (from White's view): {} cp", result2.score);
    println!("PV: {}", result2.pv.iter().map(|m| m.to_uci()).collect::<Vec<_>>().join(" "));
    println!();

    // Now manually play Ng5 and see what happens
    println!("=== After 1.Nf3 d5 2.Ng5 (Black to move) ===");
    let fen3 = "rnbqkbnr/ppp1pppp/8/3p2N1/8/8/PPPPPPPP/RNBQKB1R b KQkq - 1 2";
    let board3 = parse_fen(fen3).unwrap();
    let result3 = searcher.search(&board3, 8);
    println!("Black's best move: {}", result3.best_move.to_uci());
    println!("Score (from Black's view): {} cp", result3.score);
    println!("PV: {}", result3.pv.iter().map(|m| m.to_uci()).collect::<Vec<_>>().join(" "));
    println!();

    // Apply Black's response and continue
    let mut board4 = board3.clone();
    board4.make_move(result3.best_move);
    println!("=== After Black's response: {} ===", board4.to_fen());
    let result4 = searcher.search(&board4, 8);
    println!("White's best move: {}", result4.best_move.to_uci());
    println!("Score (from White's view): {} cp", result4.score);
    println!();

    println!("=== ANALYSIS ===");
    println!("The score after Ng5 from Black's view is: {} cp", result3.score);
    println!("From White's view, that's: {} cp", -result3.score);
    println!();
    println!("If the score is positive from White's view after Ng5, then Ng5 looks good.");
    println!("But objectively, Ng5 should be roughly equal or slightly worse for White.");
}
