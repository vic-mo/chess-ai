//! Move generation example.
//!
//! Demonstrates how to generate and display legal moves for various positions.

use engine::board::Board;
use engine::io::parse_fen;

fn main() {
    println!("=== Move Generation Examples ===\n");

    // Example 1: Starting position
    println!("1. Starting Position");
    let board = Board::startpos();
    display_moves(&board);

    // Example 2: Position with check
    println!("\n2. Position with King in Check");
    let fen = "4k3/8/8/8/8/8/8/4R2K b - - 0 1";
    let board = parse_fen(fen).unwrap();
    display_moves(&board);

    // Example 3: Position with castling available
    println!("\n3. Position with Castling Rights");
    let fen = "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1";
    let board = parse_fen(fen).unwrap();
    display_moves(&board);

    // Example 4: Position with en passant
    println!("\n4. Position with En Passant");
    let fen = "rnbqkbnr/ppp1p1pp/8/3pPp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3";
    let board = parse_fen(fen).unwrap();
    display_moves(&board);

    // Example 5: Position with promotions
    println!("\n5. Position with Pawn Promotions");
    let fen = "8/4P3/8/8/8/8/4p3/4k1K1 w - - 0 1";
    let board = parse_fen(fen).unwrap();
    display_moves(&board);
}

fn display_moves(board: &Board) {
    println!("{:?}", board);

    let legal_moves = board.generate_legal_moves();
    println!("Legal moves: {}", legal_moves.len());

    if legal_moves.len() <= 30 {
        // Show all moves if not too many
        for (i, m) in legal_moves.iter().enumerate() {
            if i % 5 == 0 {
                if i > 0 {
                    println!();
                }
                print!("  ");
            }
            print!("{:6} ", m.to_uci());
        }
        println!();
    } else {
        // Show first 20 if many moves
        print!("  ");
        for (i, m) in legal_moves.iter().take(20).enumerate() {
            if i > 0 && i % 5 == 0 {
                println!();
                print!("  ");
            }
            print!("{:6} ", m.to_uci());
        }
        println!("\n  ... ({} more)", legal_moves.len() - 20);
    }
}
