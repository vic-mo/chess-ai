/// Debug move generation for free piece positions
use engine::board::Board;
use engine::io::parse_fen;
use engine::movegen::generate_moves;

fn main() {
    println!("========================================");
    println!("DEBUG MOVE GENERATION");
    println!("========================================\n");

    // Test 1: Free black rook on e4, white to move
    println!("Test 1: Free black rook on e4");
    println!("----------------------------------------");
    let fen = "rnbqkbnr/pppppppp/8/8/4r3/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    println!("FEN: {}", fen);

    let board = parse_fen(fen).unwrap();
    println!("Board parsed successfully");
    println!("Side to move: {:?}", board.side_to_move());

    // Check what's on e4
    let e4 = engine::square::Square::from_algebraic("e4").unwrap();
    println!("Square e4: {:?}", e4);

    if let Some(piece) = board.piece_at(e4) {
        println!("Piece on e4: {:?} {:?}", piece.color, piece.piece_type);
    } else {
        println!("No piece on e4!");
    }

    // Check what's on d1 (white queen)
    let d1 = engine::square::Square::from_algebraic("d1").unwrap();
    if let Some(piece) = board.piece_at(d1) {
        println!("Piece on d1: {:?} {:?}", piece.color, piece.piece_type);
    } else {
        println!("No piece on d1!");
    }

    // Check what can attack e4
    println!("\nChecking what White pieces can attack e4...");

    // Check if d2 pawn can move to e3
    let e2 = engine::square::Square::from_algebraic("e2").unwrap();
    let e3 = engine::square::Square::from_algebraic("e3").unwrap();
    if let Some(piece) = board.piece_at(e2) {
        println!("Piece on e2: {:?} {:?}", piece.color, piece.piece_type);
        println!("e2 blocks queen's path to e4!");
    }

    // Check queen attacks
    println!("\nChecking queen attacks from d1...");
    let queen_attacks = engine::attacks::queen_attacks(d1, board.occupied());
    println!("Queen attacks: {} squares", queen_attacks.count());
    println!("Does queen attack e4? {}", queen_attacks.contains(e4));
    println!("Does queen attack e2? {}", queen_attacks.contains(e2));
    println!("Does queen attack e3? {}", queen_attacks.contains(e3));

    // Check color bitboards
    println!("\nColor bitboards:");
    println!("White pieces: {} pieces", board.color_bb(engine::piece::Color::White).count());
    println!("Black pieces: {} pieces", board.color_bb(engine::piece::Color::Black).count());
    println!("Total occupied: {} pieces", board.occupied().count());

    // Check if e4 is in black pieces
    let black_bb = board.color_bb(engine::piece::Color::Black);
    println!("Is e4 in black bitboard? {}", black_bb.contains(e4));

    println!("\nGenerating moves...");
    let moves = generate_moves(&board);
    println!("Total moves generated: {}", moves.len());

    println!("\nAll moves:");
    for i in 0..moves.len() {
        let mv = moves[i];
        println!("  {} (is_capture: {})", mv.to_uci(), mv.is_capture());
    }

    println!("\nCaptures only:");
    let mut capture_count = 0;
    for i in 0..moves.len() {
        let mv = moves[i];
        if mv.is_capture() {
            println!("  {}", mv.to_uci());
            capture_count += 1;
        }
    }
    println!("Total captures: {}", capture_count);

    println!("\nMoves to e4:");
    let mut e4_moves = 0;
    for i in 0..moves.len() {
        let mv = moves[i];
        if mv.to().to_algebraic() == "e4" {
            println!("  {} (is_capture: {})", mv.to_uci(), mv.is_capture());
            e4_moves += 1;
        }
    }
    println!("Total moves to e4: {}", e4_moves);
}
