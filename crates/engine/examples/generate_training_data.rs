//! Generate training positions from sample games.
//!
//! This creates a small training dataset for testing Texel tuning.

use engine::board::Board;
use engine::io::ToFen;
use engine::r#move::Move;
use std::env;
use std::fs::File;
use std::io::Write;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: generate_training_data <output.epd> [num_games]");
        println!();
        println!("Generates training positions from random games.");
        println!("Each position is saved with a result (1.0, 0.5, or 0.0).");
        return;
    }

    let output_path = &args[1];
    let num_games = args.get(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(100);

    println!("Generating {} training games...", num_games);

    match generate_training_data(output_path, num_games) {
        Ok(count) => println!("Generated {} positions in {}", count, output_path),
        Err(e) => eprintln!("Error: {}", e),
    }
}

fn generate_training_data(output_path: &str, num_games: usize) -> std::io::Result<usize> {
    let mut output = File::create(output_path)?;
    let mut total_positions = 0;

    for game_idx in 0..num_games {
        if game_idx % 10 == 0 {
            println!("Game {}/{}", game_idx, num_games);
        }

        let (positions, result) = generate_random_game();

        for fen in positions {
            writeln!(output, "{}; {};", fen, result)?;
            total_positions += 1;
        }
    }

    Ok(total_positions)
}

/// Generate a random game and return positions with result.
fn generate_random_game() -> (Vec<String>, f64) {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    let mut board = Board::startpos();
    let mut positions = Vec::new();
    let mut move_count = 0;

    // Play random moves until game over or 50 moves
    for _ in 0..100 {
        let legal_moves = board.generate_legal_moves();

        if legal_moves.len() == 0 {
            // Game over
            break;
        }

        // Pick a random move
        let move_idx = rng.gen_range(0..legal_moves.len());
        let mv = legal_moves[move_idx];

        board.make_move(mv);
        move_count += 1;

        // Collect positions after move 10 and before move 50
        if move_count > 10 && move_count < 50 && !board.is_in_check() {
            positions.push(board.to_fen());
        }

        // Stop after 50 moves
        if move_count >= 50 {
            break;
        }
    }

    // Assign random result (in real tuning, use actual game results)
    let result = match rng.gen_range(0..3) {
        0 => 1.0,  // White win
        1 => 0.0,  // Black win
        _ => 0.5,  // Draw
    };

    (positions, result)
}
