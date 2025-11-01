//! Texel tuning for optimizing evaluation parameters.
//!
//! This program loads training positions and optimizes evaluation weights
//! to minimize the error between predicted and actual game results.

use engine::tune::{load_training_positions, optimize};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: texel_tune <training_data.epd> [max_iterations] [learning_rate]");
        println!();
        println!("Runs Texel tuning to optimize evaluation parameters.");
        println!();
        println!("Arguments:");
        println!("  training_data.epd  File with training positions (FEN; result;)");
        println!("  max_iterations     Maximum optimization iterations (default: 500)");
        println!("  learning_rate      Learning rate for gradient descent (default: 1)");
        println!();
        println!("Example:");
        println!("  texel_tune training.epd 1000 1");
        return;
    }

    let training_file = &args[1];
    let max_iterations = args.get(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(500);
    let learning_rate = args.get(3)
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);

    println!("=== Texel Tuning ===");
    println!("Training file: {}", training_file);
    println!("Max iterations: {}", max_iterations);
    println!("Learning rate: {}", learning_rate);
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

    // Run optimization
    let optimized_params = optimize(&positions, max_iterations, learning_rate);

    // Save results
    let output_file = "optimized_params.txt";
    match optimized_params.save_to_file(output_file) {
        Ok(_) => println!("\nOptimized parameters saved to: {}", output_file),
        Err(e) => eprintln!("Error saving parameters: {}", e),
    }

    println!();
    println!("=== Optimized Parameters ===");
    println!("PST scale: {}", optimized_params.pst_scale);
    println!("Pawn structure divisor: {}", optimized_params.pawn_structure_divisor);
    println!("Mobility divisor: {}", optimized_params.mobility_divisor);
    println!();
    println!("Pawn structure (MG, EG):");
    println!("  Doubled: [{}, {}]", optimized_params.doubled_pawn_mg, optimized_params.doubled_pawn_eg);
    println!("  Isolated: [{}, {}]", optimized_params.isolated_pawn_mg, optimized_params.isolated_pawn_eg);
    println!("  Backward: [{}, {}]", optimized_params.backward_pawn_mg, optimized_params.backward_pawn_eg);
    println!("  Protected: [{}, {}]", optimized_params.protected_pawn_mg, optimized_params.protected_pawn_eg);
    println!("  Island penalty: [{}, {}]", optimized_params.pawn_island_mg, optimized_params.pawn_island_eg);
    println!();
    println!("Passed pawns (MG, EG by rank):");
    for rank in 2..=7 {
        println!("  Rank {}: [{}, {}]",
            rank,
            optimized_params.passed_pawn_mg[rank],
            optimized_params.passed_pawn_eg[rank]
        );
    }
}
