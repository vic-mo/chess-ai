/// Test if the Ng5-h6-h7-f8 line is legal
use engine::io::{parse_fen, ToFen};
use engine::movegen::generate_moves;

fn main() {
    println!("\n=== Testing Ng5 tactic legality ===\n");

    // Position after 1.Nf3 d5 2.Ng5 Nh6
    let fen = "rnbqkb1r/ppp1pppp/7n/3p2N1/8/8/PPPPPPPP/RNBQKB1R w KQkq - 2 3";
    println!("After 1.Nf3 d5 2.Ng5 Nh6:");
    println!("FEN: {}", fen);

    let board = parse_fen(fen).expect("Valid FEN");
    let moves = generate_moves(&board);

    // Look for Nxh7
    let nxh7 = moves.iter().find(|m| {
        let from = m.from();
        let to = m.to();
        from.rank() == 4 && from.file() == 6 && // g5
        to.rank() == 6 && to.file() == 7         // h7
    });

    if let Some(mv) = nxh7 {
        println!("\n✓ Nxh7 is legal ({})", mv);

        // Make the move
        let mut new_board = board.clone();
        new_board.make_move(*mv);

        println!("\nAfter 3.Nxh7:");
        println!("FEN: {}", new_board.to_fen());

        // Now check if Nxf8 is legal from h7
        let moves_after = generate_moves(&new_board);

        println!("\nBlack's legal moves:");
        for m in moves_after.iter().take(10) {
            println!("  {}", m);
        }

        // Black to move - let's say Black plays d4 or something
        // Skip to checking if White can play Nxf8

    } else {
        println!("\n✗ Nxh7 is NOT legal!");
        println!("\nLegal moves from g5:");
        for mv in moves.iter() {
            let from = mv.from();
            if from.rank() == 4 && from.file() == 6 { // g5
                println!("  {}", mv);
            }
        }
    }

    // Also test: After Nxh7, can the knight go to f8?
    println!("\n=== Testing if knight on h7 can reach f8 ===\n");

    let fen2 = "rnbqkb1r/ppp1pppN/7n/3p4/8/8/PPPPPPPP/RNBQKB1R b KQkq - 0 3";
    println!("After 3.Nxh7 (Black to move):");
    let board2 = parse_fen(fen2).expect("Valid FEN");

    // Black moves, then can White play Nxf8?
    // Let's say Black plays ...d4
    let black_moves = generate_moves(&board2);
    let d4_move = black_moves.iter().find(|m| {
        let from = m.from();
        let to = m.to();
        from.rank() == 3 && from.file() == 3 && // d5
        to.rank() == 2 && to.file() == 3         // d4
    });

    if let Some(mv) = d4_move {
        let mut board3 = board2.clone();
        board3.make_move(*mv);

        println!("After 3...d4:");
        println!("FEN: {}", board3.to_fen());

        let white_moves = generate_moves(&board3);

        // Can White play Nxf8?
        let nxf8 = white_moves.iter().find(|m| {
            let from = m.from();
            let to = m.to();
            from.rank() == 6 && from.file() == 7 && // h7
            to.rank() == 7 && to.file() == 5         // f8
        });

        if let Some(mv) = nxf8 {
            println!("✓ Nxf8 is legal from h7!");

            let mut board4 = board3.clone();
            board4.make_move(*mv);

            println!("\nAfter 4.Nxf8:");
            println!("FEN: {}", board4.to_fen());

            // Material count
            use engine::eval::evaluate_material;
            use engine::piece::Color;

            let white_mat = evaluate_material(&board4, Color::White);
            let black_mat = evaluate_material(&board4, Color::Black);

            println!("\nMaterial:");
            println!("  White: {} cp", white_mat);
            println!("  Black: {} cp", black_mat);
            println!("  Diff: {:+} cp", white_mat - black_mat);

            println!("\nBut the knight on f8 can be captured by ...");
            let black_moves_final = generate_moves(&board4);
            for m in black_moves_final.iter() {
                let to = m.to();
                if to.rank() == 7 && to.file() == 5 { // captures on f8
                    println!("  {}", m);
                }
            }

        } else {
            println!("✗ Nxf8 is NOT legal from h7!");
            println!("\nKnight on h7 moves:");
            for m in white_moves.iter() {
                let from = m.from();
                if from.rank() == 6 && from.file() == 7 { // h7
                    println!("  {}", m);
                }
            }
        }
    }
}
