/// Debug test to understand why mate in 1 is not found
use engine::io::parse_fen;
use engine::movelist::MoveList;

#[test]
fn debug_mate_position() {
    // Mate in 1: Ra8# is checkmate (back rank mate)
    // King trapped by its own pawns, rook delivers mate on 8th rank
    let fen = "6k1/5ppp/8/8/8/8/8/R6K w - - 0 1";
    let board = parse_fen(fen).unwrap();

    println!("\n=== Board Position ===");
    println!("{}", fen);
    println!("\nSide to move: {:?}", board.side_to_move());

    // Generate all legal moves
    let moves = board.generate_legal_moves();

    println!("\n=== All Legal Moves ({}) ===", moves.len());
    for (i, mv) in moves.iter().enumerate() {
        println!("{}: {}", i + 1, mv.to_uci());
    }

    // Check if a1a8 is in the list (Ra8#)
    let ra8_found = moves.iter().any(|mv| mv.to_uci() == "a1a8");

    println!("\n=== Analysis ===");
    println!("a1a8 (Ra8#) in legal moves: {}", ra8_found);

    if !ra8_found {
        println!("❌ BUG: a1a8 is NOT being generated!");
        println!("This is a move generation bug.");
    } else {
        println!("✅ a1a8 IS generated correctly");
        println!("Now checking if it's checkmate...");

        // Find a1a8 and print details
        for mv in moves.iter() {
            if mv.to_uci() == "a1a8" {
                println!("\nMove details:");
                println!("  From: {:?}", mv.from());
                println!("  To: {:?}", mv.to());
                println!("  Flags: {:?}", mv.flags());

                // Make the move and check if it's checkmate
                let mut new_board = board.clone();
                new_board.make_move(*mv);

                println!("\nAfter a1a8 (Ra8+):");
                println!("  In check: {}", new_board.is_in_check());

                let opponent_moves = new_board.generate_legal_moves();
                println!("  Opponent has {} legal moves", opponent_moves.len());

                if opponent_moves.len() > 0 {
                    println!("  Opponent legal moves:");
                    for (i, opp_mv) in opponent_moves.iter().enumerate() {
                        println!("    {}: {}", i + 1, opp_mv.to_uci());
                    }
                }

                if new_board.is_in_check() && opponent_moves.len() == 0 {
                    println!("  ✅ This is CHECKMATE!");
                } else {
                    println!("  ❌ Not checkmate (check: {}, moves: {})",
                             new_board.is_in_check(), opponent_moves.len());
                }
            }
        }
    }

    assert!(ra8_found, "a1a8 (Ra8#) must be a legal move in this position");
}
