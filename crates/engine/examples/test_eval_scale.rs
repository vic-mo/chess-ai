use engine::eval::Evaluator;
use engine::io::parse_fen;

fn main() {
    let test_positions = vec![
        ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", "Starting position"),
        ("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1", "After 1.e4"),
        ("rnbqkb1r/pppppppp/5n2/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 1 2", "After 1.e4 Nf6"),
        ("r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4", "Italian opening"),
        ("rnbqkb1r/ppp2ppp/4pn2/3p4/2PP4/2N5/PP2PPPP/R1BQKBNR w KQkq - 0 4", "Queen's Gambit"),
        ("8/8/8/4k3/8/8/4K3/8 w - - 0 1", "K vs K endgame"),
        ("8/8/8/4k3/8/8/4KP2/8 w - - 0 1", "K+P vs K endgame"),
        ("rnbqkb1r/pppppppp/5n2/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 2 2", "After 1.e4 Nf6 symmetrical"),
    ];

    println!("=== Evaluation Scale Test ===\n");
    println!("Testing evaluation scale on various positions...\n");

    let mut evaluator = Evaluator::new();

    for (fen, description) in test_positions {
        let board = parse_fen(fen).expect("Valid FEN");
        let eval = evaluator.evaluate(&board);

        println!("{}", description);
        println!("  FEN: {}", fen);
        println!("  Eval: {} centipawns", eval);
        println!("  In pawns: {:.2}", eval as f64 / 100.0);
        println!();
    }

    println!("\n=== Analysis ===\n");
    println!("Standard engine evaluation scales:");
    println!("  - Pawn value = 100 centipawns");
    println!("  - Knight/Bishop = 300-350 cp");
    println!("  - Rook = 500 cp");
    println!("  - Queen = 900 cp");
    println!();
    println!("For Texel tuning:");
    println!("  - If evaluations are ~100 for typical positions: Good");
    println!("  - If evaluations are ~500-1000: Need to divide by 5-10");
    println!("  - K parameter should be 1.2-1.4 for correct scale");
}
