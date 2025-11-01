//! Texel tuning framework for optimizing evaluation parameters.
//!
//! This module implements the Texel tuning method, which uses gradient descent
//! to optimize evaluation function weights based on game outcomes.

use crate::eval::Evaluator;
use crate::io::parse_fen;
use std::cell::RefCell;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

thread_local! {
    /// Thread-local storage for tunable parameters during optimization.
    /// When set, the evaluation function will use these parameters instead of defaults.
    pub static TUNING_PARAMS: RefCell<Option<TuningParams>> = RefCell::new(None);
}

/// Set the thread-local tuning parameters.
/// Call this before evaluating positions during tuning.
pub fn set_tuning_params(params: TuningParams) {
    TUNING_PARAMS.with(|p| *p.borrow_mut() = Some(params));
}

/// Clear the thread-local tuning parameters.
/// Call this to return to using default evaluation parameters.
pub fn clear_tuning_params() {
    TUNING_PARAMS.with(|p| *p.borrow_mut() = None);
}

/// Get a specific parameter value, using tuning params if set, otherwise default.
pub fn get_param_or_default<F>(tuning_getter: F, default: i32) -> i32
where
    F: FnOnce(&TuningParams) -> i32,
{
    TUNING_PARAMS.with(|p| {
        p.borrow()
            .as_ref()
            .map(tuning_getter)
            .unwrap_or(default)
    })
}

/// A training position with its game outcome.
#[derive(Debug, Clone)]
pub struct TrainingPosition {
    /// FEN string of the position
    pub fen: String,
    /// Game result from white's perspective (1.0 = white win, 0.5 = draw, 0.0 = black loss)
    pub result: f64,
}

/// Tunable evaluation parameters.
///
/// This structure contains all the weights that can be optimized.
#[derive(Debug, Clone)]
pub struct TuningParams {
    // PST scaling
    pub pst_scale: i32,

    // Pawn structure (middlegame, endgame)
    pub doubled_pawn_mg: i32,
    pub doubled_pawn_eg: i32,
    pub isolated_pawn_mg: i32,
    pub isolated_pawn_eg: i32,
    pub backward_pawn_mg: i32,
    pub backward_pawn_eg: i32,
    pub protected_pawn_mg: i32,
    pub protected_pawn_eg: i32,
    pub pawn_island_mg: i32,
    pub pawn_island_eg: i32,

    // Passed pawns by rank (ranks 2-7, middlegame and endgame)
    pub passed_pawn_mg: [i32; 8],
    pub passed_pawn_eg: [i32; 8],

    // Mobility scaling
    pub mobility_scale: i32,

    // King safety (if we decide to re-enable it)
    pub king_safety_scale: i32,

    // Overall scaling divisors
    pub pawn_structure_divisor: i32,
    pub mobility_divisor: i32,
    pub king_safety_divisor: i32,
    pub threat_divisor: i32,
}

impl TuningParams {
    /// Create params from current evaluation constants.
    pub fn from_current_eval() -> Self {
        Self {
            pst_scale: 4,  // Current divisor is 4

            // Current pawn values (from eval/pawns.rs)
            doubled_pawn_mg: -15,
            doubled_pawn_eg: -15,
            isolated_pawn_mg: -15,
            isolated_pawn_eg: -20,
            backward_pawn_mg: -10,
            backward_pawn_eg: -15,
            protected_pawn_mg: 5,
            protected_pawn_eg: 10,
            pawn_island_mg: -10,
            pawn_island_eg: -15,

            // Passed pawns by rank
            passed_pawn_mg: [0, 0, 10, 15, 30, 50, 80, 0],
            passed_pawn_eg: [0, 0, 15, 25, 50, 90, 150, 0],

            mobility_scale: 8,  // Current divisor is 8
            king_safety_scale: 0,  // Currently disabled

            pawn_structure_divisor: 4,
            mobility_divisor: 8,
            king_safety_divisor: 12,  // Optimal (50% vs SF1800, +65 ELO)
            threat_divisor: 8,  // Initial value for threat evaluation
        }
    }

    /// Get the number of tunable parameters.
    pub fn param_count() -> usize {
        14 + 12  // 14 scalar params + 12 passed pawn params (2 per rank for ranks 2-7)
    }

    /// Get a parameter value by index.
    pub fn get_param(&self, index: usize) -> i32 {
        match index {
            0 => self.pst_scale,
            1 => self.doubled_pawn_mg,
            2 => self.doubled_pawn_eg,
            3 => self.isolated_pawn_mg,
            4 => self.isolated_pawn_eg,
            5 => self.backward_pawn_mg,
            6 => self.backward_pawn_eg,
            7 => self.protected_pawn_mg,
            8 => self.protected_pawn_eg,
            9 => self.pawn_island_mg,
            10 => self.pawn_island_eg,
            11 => self.mobility_scale,
            12 => self.pawn_structure_divisor,
            13 => self.mobility_divisor,
            i if i >= 14 && i < 26 => {
                let rank_idx = (i - 14) / 2 + 2;
                if (i - 14) % 2 == 0 {
                    self.passed_pawn_mg[rank_idx]
                } else {
                    self.passed_pawn_eg[rank_idx]
                }
            }
            _ => panic!("Invalid parameter index"),
        }
    }

    /// Set a parameter value by index.
    pub fn set_param(&mut self, index: usize, value: i32) {
        match index {
            0 => self.pst_scale = value,
            1 => self.doubled_pawn_mg = value,
            2 => self.doubled_pawn_eg = value,
            3 => self.isolated_pawn_mg = value,
            4 => self.isolated_pawn_eg = value,
            5 => self.backward_pawn_mg = value,
            6 => self.backward_pawn_eg = value,
            7 => self.protected_pawn_mg = value,
            8 => self.protected_pawn_eg = value,
            9 => self.pawn_island_mg = value,
            10 => self.pawn_island_eg = value,
            11 => self.mobility_scale = value,
            12 => self.pawn_structure_divisor = value,
            13 => self.mobility_divisor = value,
            i if i >= 14 && i < 26 => {
                let rank_idx = (i - 14) / 2 + 2;
                if (i - 14) % 2 == 0 {
                    self.passed_pawn_mg[rank_idx] = value;
                } else {
                    self.passed_pawn_eg[rank_idx] = value;
                }
            }
            _ => panic!("Invalid parameter index"),
        }
    }

    /// Get parameter names for logging.
    pub fn param_names() -> Vec<&'static str> {
        let mut names = vec![
            "pst_scale",
            "doubled_pawn_mg",
            "doubled_pawn_eg",
            "isolated_pawn_mg",
            "isolated_pawn_eg",
            "backward_pawn_mg",
            "backward_pawn_eg",
            "protected_pawn_mg",
            "protected_pawn_eg",
            "pawn_island_mg",
            "pawn_island_eg",
            "mobility_scale",
            "pawn_structure_divisor",
            "mobility_divisor",
        ];

        // Add passed pawn names
        for rank in 2..=7 {
            names.push(Box::leak(format!("passed_pawn_mg_r{}", rank).into_boxed_str()));
            names.push(Box::leak(format!("passed_pawn_eg_r{}", rank).into_boxed_str()));
        }

        names
    }

    /// Save parameters to a file.
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        let mut file = File::create(path)?;

        writeln!(file, "// Optimized evaluation parameters from Texel tuning")?;
        writeln!(file)?;
        writeln!(file, "PST scale: {}", self.pst_scale)?;
        writeln!(file)?;
        writeln!(file, "Pawn structure:")?;
        writeln!(file, "  doubled_pawn: [{}, {}]", self.doubled_pawn_mg, self.doubled_pawn_eg)?;
        writeln!(file, "  isolated_pawn: [{}, {}]", self.isolated_pawn_mg, self.isolated_pawn_eg)?;
        writeln!(file, "  backward_pawn: [{}, {}]", self.backward_pawn_mg, self.backward_pawn_eg)?;
        writeln!(file, "  protected_pawn: [{}, {}]", self.protected_pawn_mg, self.protected_pawn_eg)?;
        writeln!(file, "  pawn_island: [{}, {}]", self.pawn_island_mg, self.pawn_island_eg)?;
        writeln!(file)?;
        writeln!(file, "Passed pawns (by rank):")?;
        for rank in 2..=7 {
            writeln!(file, "  rank {}: [{}, {}]", rank,
                self.passed_pawn_mg[rank], self.passed_pawn_eg[rank])?;
        }
        writeln!(file)?;
        writeln!(file, "Mobility scale: {}", self.mobility_scale)?;
        writeln!(file, "Pawn structure divisor: {}", self.pawn_structure_divisor)?;
        writeln!(file, "Mobility divisor: {}", self.mobility_divisor)?;

        Ok(())
    }
}

/// Compute the error between predicted and actual results.
///
/// Uses mean squared error with sigmoid function to convert centipawns to win probability.
pub fn compute_error(positions: &[TrainingPosition], params: &TuningParams) -> f64 {
    // Set thread-local params so evaluation uses them
    set_tuning_params(params.clone());

    let mut total_error = 0.0;
    let mut evaluator = Evaluator::new();

    // Tuning constant for sigmoid (optimized from real data)
    // K=0.50 indicates our evaluation is less predictive than strong engines (K=1.2-1.4)
    // This is honest - we need to be less confident about our evaluations
    let k = 0.50;

    for pos in positions {
        let board = match parse_fen(&pos.fen) {
            Ok(b) => b,
            Err(_) => continue,
        };

        // Evaluate position using tuning parameters
        let eval = evaluator.evaluate(&board);

        // Convert centipawns to winning probability using sigmoid
        // P(win) = 1 / (1 + 10^(-k * eval / 400))
        let eval_f64 = eval as f64;
        let predicted = 1.0 / (1.0 + 10.0_f64.powf(-k * eval_f64 / 400.0));

        // Mean squared error
        let error = (predicted - pos.result).powi(2);
        total_error += error;
    }

    total_error / positions.len() as f64
}

/// Compute error using only the K sigmoid parameter optimization.
///
/// This finds the best K value for converting centipawn scores to win probability.
/// This is often done as a first step before tuning other parameters.
pub fn optimize_k(positions: &[TrainingPosition]) -> f64 {
    let mut evaluator = Evaluator::new();
    let mut best_k = 1.0;
    let mut best_error = f64::MAX;

    println!("Optimizing K parameter...");

    // Try different K values from 0.5 to 2.0
    for k_times_100 in 50..=200 {
        let k = k_times_100 as f64 / 100.0;
        let mut total_error = 0.0;

        for pos in positions {
            let board = match parse_fen(&pos.fen) {
                Ok(b) => b,
                Err(_) => continue,
            };

            let eval = evaluator.evaluate(&board) as f64;
            let predicted = 1.0 / (1.0 + 10.0_f64.powf(-k * eval / 400.0));
            let error = (predicted - pos.result).powi(2);
            total_error += error;
        }

        let avg_error = total_error / positions.len() as f64;

        if avg_error < best_error {
            best_error = avg_error;
            best_k = k;
        }

        if k_times_100 % 10 == 0 {
            println!("  K = {:.2}, error = {:.6}", k, avg_error);
        }
    }

    println!("Best K = {:.2} (error = {:.6})", best_k, best_error);
    best_k
}

/// Load training positions from an EPD file with results.
///
/// Expected format: FEN; result (1.0, 0.5, or 0.0);
pub fn load_training_positions<P: AsRef<Path>>(path: P) -> std::io::Result<Vec<TrainingPosition>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut positions = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let line = line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Parse line: "FEN; result;"
        let parts: Vec<&str> = line.split(';').collect();
        if parts.len() < 2 {
            continue;
        }

        let fen = parts[0].trim().to_string();
        let result = parts[1].trim().parse::<f64>().unwrap_or(0.5);

        positions.push(TrainingPosition { fen, result });
    }

    Ok(positions)
}

/// Run the Texel tuning optimization.
///
/// Uses simple gradient descent to minimize the error function.
pub fn optimize(
    positions: &[TrainingPosition],
    max_iterations: usize,
    learning_rate: i32,
) -> TuningParams {
    let mut params = TuningParams::from_current_eval();
    let mut best_error = compute_error(positions, &params);

    println!("Starting Texel tuning with {} positions", positions.len());
    println!("Initial error: {:.6}", best_error);
    println!();

    let param_names = TuningParams::param_names();
    let param_count = TuningParams::param_count();

    for iteration in 0..max_iterations {
        let mut improved = false;
        let old_error = best_error;

        // Try adjusting each parameter
        for i in 0..param_count {
            let original = params.get_param(i);
            let name = param_names[i];

            // Skip divisor parameters if they would become invalid
            if name.contains("divisor") && original <= 1 {
                continue;
            }

            // Try increasing
            params.set_param(i, original + learning_rate);
            let error_plus = compute_error(positions, &params);

            // Try decreasing
            params.set_param(i, original - learning_rate);
            let error_minus = compute_error(positions, &params);

            // Keep the best
            if error_plus < best_error {
                params.set_param(i, original + learning_rate);
                best_error = error_plus;
                improved = true;
            } else if error_minus < best_error {
                params.set_param(i, original - learning_rate);
                best_error = error_minus;
                improved = true;
            } else {
                params.set_param(i, original);
            }
        }

        let improvement = old_error - best_error;

        if iteration % 10 == 0 {
            println!(
                "Iteration {}: error = {:.6}, improvement = {:.6}",
                iteration, best_error, improvement
            );
        }

        // Stop if no improvement
        if !improved {
            println!("Converged after {} iterations", iteration);
            break;
        }

        // Stop if improvement is very small
        if improvement < 0.000001 {
            println!("Converged (improvement < 0.000001) after {} iterations", iteration);
            break;
        }
    }

    println!();
    println!("Final error: {:.6}", best_error);
    println!("Optimization complete!");

    params
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sigmoid_conversion() {
        let k = 1.3;

        // +100 cp should be ~55% win rate
        let eval = 100.0;
        let prob = 1.0 / (1.0 + 10.0_f64.powf(-k * eval / 400.0));
        assert!((0.53..0.57).contains(&prob));

        // +400 cp should be ~90% win rate
        let eval = 400.0;
        let prob = 1.0 / (1.0 + 10.0_f64.powf(-k * eval / 400.0));
        assert!((0.85..0.95).contains(&prob));
    }

    #[test]
    fn test_params_initialization() {
        let params = TuningParams::from_current_eval();
        assert_eq!(params.pst_scale, 4);
        assert_eq!(params.pawn_structure_divisor, 4);
        assert_eq!(params.mobility_divisor, 8);
    }
}
