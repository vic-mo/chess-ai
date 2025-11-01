//! Validate tuned parameters and suggest safe values.
//!
//! This program analyzes optimized parameters and checks for unrealistic values.

use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: validate_params <optimized_params.txt>");
        println!();
        println!("Analyzes tuned parameters and suggests safe values for testing.");
        return;
    }

    let params_file = &args[1];

    println!("=== Parameter Validation ===");
    println!("Reading: {}", params_file);
    println!();

    match validate_file(params_file) {
        Ok(_) => println!("\nValidation complete!"),
        Err(e) => eprintln!("Error: {}", e),
    }
}

fn validate_file(path: &str) -> std::io::Result<()> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut pst_scale = 4;
    let mut pawn_divisor = 4;
    let mut mobility_divisor = 8;

    println!("📊 Analyzing tuned parameters...");
    println!();

    for line in reader.lines() {
        let line = line?;

        if line.contains("PST scale:") {
            if let Some(val) = extract_value(&line) {
                pst_scale = val;
                println!("PST scale: {} {}", val, validate_range(val, 2, 8, 4));
            }
        } else if line.contains("Pawn structure divisor:") {
            if let Some(val) = extract_value(&line) {
                pawn_divisor = val;
                println!("Pawn divisor: {} {}", val, validate_range(val, 2, 8, 4));
            }
        } else if line.contains("Mobility divisor:") {
            if let Some(val) = extract_value(&line) {
                mobility_divisor = val;
                println!("Mobility divisor: {} {}", val, validate_range(val, 4, 16, 8));
            }
        }
    }

    println!();
    println!("💡 Recommendations:");
    println!();

    // PST scale
    if pst_scale < 3 {
        println!("⚠️  PST scale too low ({}). Suggests PST too strong.", pst_scale);
        println!("   Recommended: Start with 4, try 3 if tactical tests pass.");
    } else if pst_scale > 8 {
        println!("⚠️  PST scale too high ({}). Suggests PST too weak.", pst_scale);
        println!("   Recommended: Start with 4-6, carefully test tactical accuracy.");
    } else {
        println!("✅ PST scale ({}) is reasonable. Safe to test.", pst_scale);
    }

    // Pawn divisor
    if pawn_divisor < 2 {
        println!("⚠️  Pawn divisor too low ({}). Pawn structure too strong!", pawn_divisor);
        println!("   Recommended: Use 3-4 for safety. Value of 1 is likely overfit.");
    } else if pawn_divisor > 8 {
        println!("⚠️  Pawn divisor too high ({}). Pawn structure too weak.", pawn_divisor);
        println!("   Recommended: Start with 4-6.");
    } else {
        println!("✅ Pawn divisor ({}) is reasonable. Safe to test.", pawn_divisor);
    }

    // Mobility divisor
    if mobility_divisor < 4 {
        println!("⚠️  Mobility divisor too low ({}). Mobility too strong!", mobility_divisor);
        println!("   Recommended: Use 6-8 for safety.");
    } else if mobility_divisor > 20 {
        println!("⚠️  Mobility divisor too high ({}). Mobility too weak.", mobility_divisor);
        println!("   Recommended: Start with 8-12.");
    } else {
        println!("✅ Mobility divisor ({}) is reasonable. Safe to test.", mobility_divisor);
    }

    println!();
    println!("📝 Suggested Conservative Test:");
    println!();
    println!("   PST scale: {}", clamp(pst_scale, 3, 6));
    println!("   Pawn divisor: {}", clamp(pawn_divisor, 3, 6));
    println!("   Mobility divisor: {}", clamp(mobility_divisor, 6, 12));
    println!();
    println!("These values are safer and less likely to hurt tactical play.");
    println!();
    println!("🧪 Testing Steps:");
    println!();
    println!("1. Update crates/engine/src/eval.rs with conservative values");
    println!("2. cargo build --release --example uci_stdio");
    println!("3. Test tactical: cargo run --release --example tactical_test_runner \\");
    println!("                  crates/engine/positions/wacnew.epd --depth 8 --limit 50");
    println!("4. If tactical ≥95%: Test 10 games vs SF1800");
    println!("5. If games improve: Test 50 games for ELO measurement");

    Ok(())
}

fn extract_value(line: &str) -> Option<i32> {
    line.split(':')
        .nth(1)?
        .trim()
        .split_whitespace()
        .next()?
        .parse()
        .ok()
}

fn validate_range(val: i32, min: i32, max: i32, default: i32) -> String {
    if val < min {
        format!("(too low, default: {})", default)
    } else if val > max {
        format!("(too high, default: {})", default)
    } else {
        String::from("(reasonable)")
    }
}

fn clamp(val: i32, min: i32, max: i32) -> i32 {
    if val < min {
        min
    } else if val > max {
        max
    } else {
        val
    }
}
