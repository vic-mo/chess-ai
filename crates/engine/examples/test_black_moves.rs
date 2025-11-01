/// Test scores for different Black responses to Ng5
use engine::io::parse_fen;
use engine::movegen::generate_moves;
use engine::search::Searcher;

fn main() {
    println!("\n=== Testing Black's move options after Ng5 ===\n");

    // After 1.Nf3 d5 2.Ng5
    let fen = "rnbqkbnr/ppp1pppp/8/3p2N1/8/8/PPPPPPPP/RNBQKB1R b KQkq - 1 2";
    println!("Position: After 1.Nf3 d5 2.Ng5 (Black to move)");
    println!();

    let board = parse_fen(fen).expect("Valid FEN");
    let moves = generate_moves(&board);

    // Find specific moves
    let nh6 = moves.iter().find(|m| {
        let from = m.from();
        let to = m.to();
        from.rank() == 7 && from.file() == 6 && // g8
        to.rank() == 5 && to.file() == 7         // h6
    }).copied();

    // Debug: print all pawn moves from d5
    println!("DEBUG: All moves from d-file:");
    for mv in moves.iter() {
        let from = mv.from();
        if from.file() == 3 {  // d-file
            println!("  {} (from rank={}, to rank={})", mv, from.rank(), mv.to().rank());
        }
    }
    println!();

    let d4 = moves.iter().find(|m| {
        let from = m.from();
        let to = m.to();
        from.rank() == 4 && from.file() == 3 && // d5 (rank 4 in 0-indexed)
        to.rank() == 3 && to.file() == 3         // d4 (rank 3 in 0-indexed)
    }).copied();

    let e6 = moves.iter().find(|m| {
        let from = m.from();
        let to = m.to();
        from.rank() == 6 && from.file() == 4 && // e7
        to.rank() == 5 && to.file() == 4         // e6
    }).copied();

    println!("Testing three candidate moves:");
    println!("  1. Nh6 - blocks knight from attacking f7");
    println!("  2. d4  - pushes pawn forward (Stockfish's choice)");
    println!("  3. e6  - develops position");
    println!();

    // Test each move
    let mut searcher = Searcher::new();

    if let Some(mv) = nh6 {
        let mut b = board.clone();
        b.make_move(mv);
        let result = searcher.search(&b, 7); // Depth 7 after the move
        println!("Nh6: White's response scores {:+} cp (from White's view)", result.score);
        println!("     = {:+} cp from Black's view", -result.score);
    }

    let mut searcher = Searcher::new();
    if let Some(mv) = d4 {
        let mut b = board.clone();
        b.make_move(mv);
        let result = searcher.search(&b, 7);
        println!("d4:  White's response scores {:+} cp (from White's view)", result.score);
        println!("     = {:+} cp from Black's view", -result.score);
    }

    let mut searcher = Searcher::new();
    if let Some(mv) = e6 {
        let mut b = board.clone();
        b.make_move(mv);
        let result = searcher.search(&b, 7);
        println!("e6:  White's response scores {:+} cp (from White's view)", result.score);
        println!("     = {:+} cp from Black's view", -result.score);
    }

    println!();
    println!("Analysis:");
    println!("  The engine should choose the move that gives Black the BEST score");
    println!("  (i.e., the most positive from Black's view, or least negative)");
}
