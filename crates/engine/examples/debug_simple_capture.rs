/// Test the simplest possible capture scenario
use engine::board::Board;
use engine::io::parse_fen;
use engine::movegen::generate_moves;

fn main() {
    println!("========================================");
    println!("SIMPLE CAPTURE TEST");
    println!("========================================\n");

    // Test 1: Black queen on d4, white queen can capture
    // Remove pawns to make path clear
    println!("Test 1: Free black queen on d4, paths clear");
    println!("----------------------------------------");
    let fen = "rnb1kbnr/pppppppp/8/8/3q4/8/PPP1PPPP/RNBQKBNR w KQkq - 0 1";
    println!("FEN: {}", fen);
    println!("(removed d2 pawn so Qd1 can reach d4)\n");

    let board = parse_fen(fen).unwrap();

    // Check pieces
    let d1 = engine::square::Square::from_algebraic("d1").unwrap();
    let d4 = engine::square::Square::from_algebraic("d4").unwrap();

    if let Some(piece) = board.piece_at(d1) {
        println!("Piece on d1: {:?} {:?}", piece.color, piece.piece_type);
    }

    if let Some(piece) = board.piece_at(d4) {
        println!("Piece on d4: {:?} {:?}", piece.color, piece.piece_type);
    }

    // Check queen attacks
    println!("\nChecking queen attacks...");
    let queen_attacks = engine::attacks::queen_attacks(d1, board.occupied());
    println!("Queen on d1 attacks {} squares", queen_attacks.count());
    println!("Can queen reach d4? {}", queen_attacks.contains(d4));

    // Generate moves
    println!("\nGenerating moves...");
    let moves = generate_moves(&board);
    println!("Total moves: {}", moves.len());

    // Find Qxd4
    let mut found_capture = false;
    for i in 0..moves.len() {
        let mv = moves[i];
        if mv.from().to_algebraic() == "d1" && mv.to().to_algebraic() == "d4" {
            println!("\n✓ Found Qd1xd4: {}", mv.to_uci());
            println!("  Is capture: {}", mv.is_capture());
            found_capture = true;
            break;
        }
    }

    if !found_capture {
        println!("\n✗ FAIL: Couldn't find Qd1xd4");
        println!("\nAll queen moves from d1:");
        for i in 0..moves.len() {
            let mv = moves[i];
            if mv.from().to_algebraic() == "d1" {
                println!("  {} (capture: {})", mv.to_uci(), mv.is_capture());
            }
        }

        println!("\nAll captures:");
        for i in 0..moves.len() {
            let mv = moves[i];
            if mv.is_capture() {
                println!("  {}", mv.to_uci());
            }
        }
    }

    println!("\n========================================");

    // Test 2: Even simpler - knight can take
    println!("\nTest 2: Knight takes undefended pawn");
    println!("----------------------------------------");
    let fen = "rnbqkbnr/pppppppp/8/8/8/5p2/PPPPPNPP/RNBQKB1R w KQkq - 0 1";
    println!("FEN: {}", fen);
    println!("(Black pawn on f3, White knight on f2 can capture)\n");

    let board = parse_fen(fen).unwrap();
    let moves = generate_moves(&board);

    let mut found = false;
    for i in 0..moves.len() {
        let mv = moves[i];
        if mv.to().to_algebraic() == "f3" && mv.is_capture() {
            println!("✓ Found capture to f3: {}", mv.to_uci());
            found = true;
            break;
        }
    }

    if !found {
        println!("✗ No capture to f3 found");
        println!("All captures:");
        for i in 0..moves.len() {
            let mv = moves[i];
            if mv.is_capture() {
                println!("  {}", mv.to_uci());
            }
        }
    }
}
