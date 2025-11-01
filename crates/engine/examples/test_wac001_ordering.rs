/// Test move ordering for WAC.001
use engine::board::Board;
use engine::io::parse_fen;
use engine::movegen::generate_moves;
use engine::move_order::MoveOrder;
use engine::r#move::Move;

fn main() {
    println!("========================================");
    println!("WAC.001 MOVE ORDERING TEST");
    println!("========================================\n");

    let fen = "2rr3k/pp3pp1/1nnqbN1p/3pN3/2pP4/2P3Q1/PPB4P/R4RK1 w - - 0 1";
    println!("FEN: {}", fen);
    println!("Expected: Qg6 should be ordered highly\n");

    let board = parse_fen(fen).unwrap();
    let mut moves = generate_moves(&board);

    println!("Total moves generated: {}", moves.len());

    // Find Qg6 move
    let mut qg6_move = Move::null();
    for i in 0..moves.len() {
        let mv = moves[i];
        if mv.from().to_algebraic() == "g3" && mv.to().to_algebraic() == "g6" {
            qg6_move = mv;
            break;
        }
    }

    if qg6_move.is_null() {
        println!("✗ Qg6 not found!");
        return;
    }

    println!("Found Qg6: {}", qg6_move.to_uci());

    // Create move orderer and order the moves
    let mut move_orderer = MoveOrder::new();

    // Score all moves
    println!("\nScoring moves...");
    for i in 0..moves.len() {
        let mv = moves[i];
        let score = move_orderer.score_move(&board, mv, 0, None, None);

        // Print moves with high scores or the Qg6 move
        if score > 1_000_000 || mv.to_uci() == qg6_move.to_uci() {
            println!("  {} - score: {} (capture: {})",
                     mv.to_uci(),
                     score,
                     mv.is_capture());
        }
    }

    // Order moves using the move orderer
    move_orderer.order_moves(&board, &mut moves, 0, None, None);

    println!("\nTop 10 ordered moves:");
    for i in 0..10.min(moves.len()) {
        let mv = moves[i];
        let score = move_orderer.score_move(&board, mv, 0, None, None);
        println!("  {}. {} (score: {}, capture: {})",
                 i + 1,
                 mv.to_uci(),
                 score,
                 mv.is_capture());
    }

    // Find where Qg6 ended up
    println!("\nSearching for Qg6 in ordered list...");
    for i in 0..moves.len() {
        let mv = moves[i];
        if mv.to_uci() == qg6_move.to_uci() {
            println!("Qg6 is at position {} out of {}", i + 1, moves.len());
            if i < 5 {
                println!("✓ Good - Qg6 is in top 5");
            } else if i < 20 {
                println!("⚠ Warning - Qg6 is at position {}, might not be searched deeply", i + 1);
            } else {
                println!("✗ Bad - Qg6 is at position {}, likely to be pruned!", i + 1);
            }
            break;
        }
    }
}
