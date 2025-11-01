/// Test WAC.001 move generation
use engine::board::Board;
use engine::io::parse_fen;
use engine::movegen::generate_moves;

fn main() {
    println!("========================================");
    println!("WAC.001 MOVE GENERATION TEST");
    println!("========================================\n");

    let fen = "2rr3k/pp3pp1/1nnqbN1p/3pN3/2pP4/2P3Q1/PPB4P/R4RK1 w - - 0 1";
    println!("FEN: {}", fen);
    println!("Expected best move: Qg6");
    println!("(Queen from g3 to g6)\n");

    let board = parse_fen(fen).unwrap();

    // Check what's on g3 and g6
    let g3 = engine::square::Square::from_algebraic("g3").unwrap();
    let g6 = engine::square::Square::from_algebraic("g6").unwrap();

    println!("Pieces:");
    if let Some(piece) = board.piece_at(g3) {
        println!("  g3: {:?} {:?}", piece.color, piece.piece_type);
    }
    if let Some(piece) = board.piece_at(g6) {
        println!("  g6: {:?}", board.piece_at(g6));
    }

    // Generate moves
    println!("\nGenerating moves...");
    let moves = generate_moves(&board);
    println!("Total moves: {}", moves.len());

    // Find Qg6
    let mut found = false;
    for i in 0..moves.len() {
        let mv = moves[i];
        if mv.from().to_algebraic() == "g3" && mv.to().to_algebraic() == "g6" {
            println!("\n✓ Found Qg6: {}", mv.to_uci());
            println!("  Is capture: {}", mv.is_capture());
            found = true;
            break;
        }
    }

    if !found {
        println!("\n✗ Qg6 not found");
        println!("\nQueen moves from g3:");
        for i in 0..moves.len() {
            let mv = moves[i];
            if mv.from().to_algebraic() == "g3" {
                println!("  {} (capture: {})", mv.to_uci(), mv.is_capture());
            }
        }
    }

    println!("\n========================================");
}
