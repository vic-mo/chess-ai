//! Basic usage example for the chess engine core.
//!
//! Demonstrates:
//! - Creating boards
//! - Making and unmaking moves
//! - Checking board state

use engine::board::Board;
use engine::r#move::{Move, MoveFlags};
use engine::square::Square;

fn main() {
    println!("=== Chess Engine Core - Basic Usage ===\n");

    // Create a board from the starting position
    let mut board = Board::startpos();
    println!("Starting position:");
    println!("{:?}\n", board);

    // Generate legal moves
    let legal_moves = board.generate_legal_moves();
    println!("Legal moves from start: {}", legal_moves.len());
    println!("First 5 moves:");
    for m in legal_moves.iter().take(5) {
        println!("  {}", m.to_uci());
    }
    println!();

    // Make a move: e2-e4
    println!("Making move e2e4 (pawn to e4)...");
    let m = Move::new(Square::E2, Square::E4, MoveFlags::DOUBLE_PAWN_PUSH);
    let undo = board.make_move(m);

    println!("{:?}\n", board);
    println!("Side to move: {:?}", board.side_to_move());
    println!("Hash: 0x{:016x}\n", board.hash());

    // Check if in check
    println!("Is in check: {}\n", board.is_in_check());

    // Unmake the move
    println!("Unmaking move...");
    board.unmake_move(m, undo);

    println!("{:?}\n", board);
    println!("Back to starting position!");
}
