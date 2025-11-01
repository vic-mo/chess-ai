//! Optimize the K parameter for sigmoid conversion.
//!
//! This finds the best K value for converting centipawn scores to win probability.
//! Should be run before tuning other parameters.

use engine::tune::{load_training_positions, optimize_k};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: optimize_k <training_data.epd>");
        println!();
        println!("Finds the optimal K parameter for sigmoid conversion.");
        println!("This should be run before tuning evaluation parameters.");
        println!();
        println!("The K parameter controls how centipawn scores map to win probability:");
        println!("  P(win) = 1 / (1 + 10^(-K * eval / 400))");
        println!();
        println!("Typical values: K = 1.0 to 1.5");
        return;
    }

    let training_file = &args[1];

    println!("=== K Parameter Optimization ===");
    println!("Training file: {}", training_file);
    println!();

    // Load training positions
    println!("Loading training positions...");
    let positions = match load_training_positions(training_file) {
        Ok(pos) => pos,
        Err(e) => {
            eprintln!("Error loading training data: {}", e);
            return;
        }
    };

    println!("Loaded {} positions", positions.len());
    println!();

    if positions.is_empty() {
        eprintln!("No training positions found!");
        return;
    }

    // Optimize K
    let best_k = optimize_k(&positions);

    println!();
    println!("=== Result ===");
    println!("Optimal K = {:.2}", best_k);
    println!();
    println!("Use this K value in texel_tune.rs by updating the 'k' constant.");
    println!();
    println!("Next step: Run texel tuning with optimized K value:");
    println!("  cargo run --release --example texel_tune {} 500 1", training_file);
}
