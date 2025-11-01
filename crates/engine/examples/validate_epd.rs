/// Validate EPD test positions from a file
///
/// This tool:
/// 1. Loads all positions from an EPD file
/// 2. Validates that FEN positions are legal
/// 3. Validates that best moves are legal in each position
/// 4. Reports any issues found
///
/// Usage: cargo run --example validate_epd -- <epd_file>

use engine::io::{load_epd_file, validate_epd_moves};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <epd_file>", args[0]);
        eprintln!("Example: {} positions/wacnew.epd", args[0]);
        std::process::exit(1);
    }

    let file_path = &args[1];

    println!("Loading EPD file: {}", file_path);
    println!("========================================");

    let positions = match load_epd_file(file_path) {
        Ok(pos) => pos,
        Err(e) => {
            eprintln!("Error loading EPD file: {}", e);
            std::process::exit(1);
        }
    };

    println!("Loaded {} positions", positions.len());
    println!();

    // Validate each position
    let mut valid_count = 0;
    let mut invalid_count = 0;
    let mut errors = Vec::new();

    for (idx, pos) in positions.iter().enumerate() {
        if let Some(illegal_move) = validate_epd_moves(pos) {
            invalid_count += 1;
            let error_msg = format!(
                "Position {} ({}): Illegal move '{}' in legal moves",
                idx + 1,
                pos.id,
                illegal_move
            );
            errors.push(error_msg);
        } else {
            valid_count += 1;
        }
    }

    println!("Validation Results:");
    println!("========================================");
    println!("Total positions: {}", positions.len());
    println!("Valid positions: {} ({:.1}%)", valid_count, (valid_count as f64 / positions.len() as f64) * 100.0);
    println!("Invalid positions: {} ({:.1}%)", invalid_count, (invalid_count as f64 / positions.len() as f64) * 100.0);
    println!();

    if !errors.is_empty() {
        println!("Errors found:");
        println!("========================================");
        for error in errors.iter().take(20) {
            println!("  {}", error);
        }
        if errors.len() > 20 {
            println!("  ... and {} more errors", errors.len() - 20);
        }
        println!();
    }

    if invalid_count > 0 {
        eprintln!("Warning: {} positions have invalid best moves", invalid_count);
        eprintln!("These may be due to:");
        eprintln!("  - SAN notation (e.g., Nf3) vs UCI notation (e.g., g1f3)");
        eprintln!("  - Move annotations (+, #, !, ?) that need to be stripped");
        eprintln!("  - Actual illegal moves in the test suite");
        std::process::exit(1);
    }

    println!("✓ All positions validated successfully!");
}
