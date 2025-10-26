//! FEN parsing and serialization example.
//!
//! Demonstrates how to parse FEN strings and convert boards back to FEN.

use engine::board::Board;
use engine::io::{parse_fen, ToFen};

fn main() {
    println!("=== FEN Parsing Examples ===\n");

    // Example 1: Starting position
    println!("1. Starting Position");
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    test_fen(fen);

    // Example 2: After 1.e4
    println!("\n2. After 1.e4");
    let fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
    test_fen(fen);

    // Example 3: Kiwipete (complex midgame)
    println!("\n3. Kiwipete Position");
    let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
    test_fen(fen);

    // Example 4: Endgame with castling rights lost
    println!("\n4. Endgame Position");
    let fen = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1";
    test_fen(fen);

    // Example 5: Using Board API directly
    println!("\n5. Creating Board Programmatically");
    let board = Board::startpos();
    let fen_out = board.to_fen();
    println!("FEN: {}", fen_out);
    println!("Legal moves: {}", board.generate_legal_moves().len());
}

fn test_fen(fen: &str) {
    println!("FEN: {}", fen);

    match parse_fen(fen) {
        Ok(board) => {
            println!("{:?}", board);

            // Round-trip test
            let fen_out = board.to_fen();
            if fen == fen_out {
                println!("✓ Round-trip successful");
            } else {
                println!("✗ Round-trip mismatch!");
                println!("  Output: {}", fen_out);
            }

            println!("Legal moves: {}", board.generate_legal_moves().len());
        }
        Err(e) => {
            println!("✗ Parse error: {}", e);
        }
    }
}
