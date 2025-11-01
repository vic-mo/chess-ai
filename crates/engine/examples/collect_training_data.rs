//! Collect training positions from PGN games for Texel tuning.
//!
//! This example extracts quiet positions from high-quality games
//! and saves them with their game results for tuning purposes.

use engine::board::Board;
use engine::io::{parse_fen, ToFen};
use engine::piece::Color;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!("Usage: collect_training_data <input.pgn> <output.epd>");
        println!();
        println!("Extracts quiet positions from PGN games for Texel tuning.");
        println!("Positions are saved in EPD format with game results.");
        return;
    }

    let input_path = &args[1];
    let output_path = &args[2];

    println!("Collecting training data from: {}", input_path);
    println!("Output file: {}", output_path);
    println!();

    match collect_positions(input_path, output_path) {
        Ok(count) => println!("Successfully collected {} positions", count),
        Err(e) => eprintln!("Error: {}", e),
    }
}

/// Extract training positions from PGN file.
fn collect_positions(input_path: &str, output_path: &str) -> Result<usize, Box<dyn std::error::Error>> {
    let input = File::open(input_path)?;
    let reader = BufReader::new(input);

    let mut output = File::create(output_path)?;
    let mut positions_collected = 0;

    let mut current_game = Game::default();
    let mut in_headers = false;

    for line in reader.lines() {
        let line = line?;
        let line = line.trim();

        // Skip empty lines
        if line.is_empty() {
            continue;
        }

        // Header line (starts with [)
        if line.starts_with('[') {
            in_headers = true;

            // Extract game result
            if line.starts_with("[Result ") {
                if let Some(result) = extract_result(line) {
                    current_game.result = Some(result);
                }
            }
            continue;
        }

        // Moves section
        if in_headers {
            in_headers = false;
        }

        // Parse moves and extract positions
        if let Some(result) = current_game.result {
            let positions = extract_positions_from_moves(line, result);
            for (fen, res) in positions {
                writeln!(output, "{}; {};", fen, res)?;
                positions_collected += 1;
            }

            // Reset for next game
            current_game = Game::default();
        }
    }

    Ok(positions_collected)
}

#[derive(Default)]
struct Game {
    result: Option<f64>,
}

/// Extract result from PGN header line.
fn extract_result(line: &str) -> Option<f64> {
    if line.contains("1-0") {
        Some(1.0)  // White win
    } else if line.contains("0-1") {
        Some(0.0)  // Black win
    } else if line.contains("1/2-1/2") {
        Some(0.5)  // Draw
    } else {
        None
    }
}

/// Extract quiet positions from moves string.
fn extract_positions_from_moves(moves_str: &str, result: f64) -> Vec<(String, f64)> {
    let mut positions = Vec::new();
    let mut board = Board::startpos();
    let mut move_count = 0;

    // Simple SAN parser (very basic - may need improvement)
    let moves: Vec<&str> = moves_str
        .split_whitespace()
        .filter(|m| !m.ends_with('.') && !m.is_empty() && *m != "1-0" && *m != "0-1" && *m != "1/2-1/2")
        .collect();

    for move_san in moves {
        move_count += 1;

        // Skip opening (first 10 moves)
        if move_count <= 10 {
            if let Some(mv) = find_move_from_san(&board, move_san) {
                board.make_move(mv);
            } else {
                break;  // Can't parse, skip this game
            }
            continue;
        }

        // Skip late endgame (after move 50)
        if move_count > 50 {
            break;
        }

        // Check if position is quiet
        if is_quiet_position(&board) {
            let fen = board.to_fen();
            positions.push((fen, result));
        }

        // Make the move
        if let Some(mv) = find_move_from_san(&board, move_san) {
            board.make_move(mv);
        } else {
            break;  // Can't parse, skip rest of game
        }

        // Collect at most 5 positions per game
        if positions.len() >= 5 {
            break;
        }
    }

    positions
}

/// Check if a position is quiet (no tactics).
fn is_quiet_position(board: &Board) -> bool {
    // Not in check
    if board.is_in_check() {
        return false;
    }

    // Has pieces (not endgame with just pawns)
    let white_pieces = board.piece_bb(engine::piece::PieceType::Knight, Color::White).count()
        + board.piece_bb(engine::piece::PieceType::Bishop, Color::White).count()
        + board.piece_bb(engine::piece::PieceType::Rook, Color::White).count()
        + board.piece_bb(engine::piece::PieceType::Queen, Color::White).count();

    let black_pieces = board.piece_bb(engine::piece::PieceType::Knight, Color::Black).count()
        + board.piece_bb(engine::piece::PieceType::Bishop, Color::Black).count()
        + board.piece_bb(engine::piece::PieceType::Rook, Color::Black).count()
        + board.piece_bb(engine::piece::PieceType::Queen, Color::Black).count();

    // At least 2 pieces per side
    white_pieces >= 2 && black_pieces >= 2
}

/// Find a move from SAN notation (basic implementation).
fn find_move_from_san(board: &Board, san: &str) -> Option<engine::r#move::Move> {
    let legal_moves = board.generate_legal_moves();

    // Remove check/checkmate symbols
    let san = san.replace('+', "").replace('#', "").replace('!', "").replace('?', "");

    // Try to match each legal move
    for mv in legal_moves.iter() {
        // Simple heuristic: check if SAN contains the destination square
        let dest = mv.to();
        let dest_str = format!("{}", dest);

        if san.contains(&dest_str) {
            // Could be the right move - return first match
            // (This is very crude, but works for simple cases)
            return Some(*mv);
        }
    }

    None
}
