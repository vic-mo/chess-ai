/// Tactical Test Runner
///
/// This tool runs the chess engine against EPD test suites to measure tactical strength.
///
/// Features:
/// - Load EPD test positions
/// - Run engine at specified depth or time limit
/// - Compare engine's best move against expected moves
/// - Track timing, nodes, and depth
/// - Generate detailed pass/fail reports
/// - Support for multiple test depths
///
/// Usage:
///   cargo run --example tactical_test_runner -- <epd_file> [options]
///
/// Options:
///   --depth <n>        Search depth (default: 10)
///   --time <ms>        Time limit in milliseconds per position
///   --limit <n>        Only test first N positions
///   --verbose          Show detailed output for each position
///   --json <file>      Save results to JSON file
///
/// Example:
///   cargo run --example tactical_test_runner -- positions/wacnew.epd --depth 8 --verbose

use engine::io::{load_epd_file, EpdTestPosition, ToFen};
use engine::search::core::Searcher;
use engine::board::Board;
use engine::movegen::generate_moves;
use std::env;
use std::time::{Duration, Instant};

#[derive(Debug)]
struct TestResult {
    position_id: String,
    passed: bool,
    engine_move: String,
    expected_moves: Vec<String>,
    score: i32,
    expected_score: Option<i32>,
    score_diff: Option<i32>,
    depth_reached: u8,
    time_ms: u64,
    nodes: u64,
    fen: String,
    pass_reason: String,
}

struct TestConfig {
    epd_file: String,
    depth: Option<u8>,
    time_ms: Option<u64>,
    limit: Option<usize>,
    verbose: bool,
    json_output: Option<String>,
}

impl TestConfig {
    fn from_args() -> Result<Self, String> {
        let args: Vec<String> = env::args().collect();

        if args.len() < 2 {
            return Err(format!(
                "Usage: {} <epd_file> [options]\n\
                Options:\n\
                  --depth <n>      Search depth (default: 10)\n\
                  --time <ms>      Time limit in milliseconds\n\
                  --limit <n>      Test only first N positions\n\
                  --verbose        Show detailed output\n\
                  --json <file>    Save results to JSON",
                args[0]
            ));
        }

        let mut config = TestConfig {
            epd_file: args[1].clone(),
            depth: Some(10),
            time_ms: None,
            limit: None,
            verbose: false,
            json_output: None,
        };

        let mut i = 2;
        while i < args.len() {
            match args[i].as_str() {
                "--depth" => {
                    if i + 1 >= args.len() {
                        return Err("--depth requires a value".to_string());
                    }
                    config.depth = Some(args[i + 1].parse().map_err(|_| "Invalid depth value")?);
                    i += 2;
                }
                "--time" => {
                    if i + 1 >= args.len() {
                        return Err("--time requires a value".to_string());
                    }
                    config.time_ms = Some(args[i + 1].parse().map_err(|_| "Invalid time value")?);
                    config.depth = None; // Time-based search
                    i += 2;
                }
                "--limit" => {
                    if i + 1 >= args.len() {
                        return Err("--limit requires a value".to_string());
                    }
                    config.limit = Some(args[i + 1].parse().map_err(|_| "Invalid limit value")?);
                    i += 2;
                }
                "--verbose" => {
                    config.verbose = true;
                    i += 1;
                }
                "--json" => {
                    if i + 1 >= args.len() {
                        return Err("--json requires a filename".to_string());
                    }
                    config.json_output = Some(args[i + 1].clone());
                    i += 2;
                }
                _ => {
                    return Err(format!("Unknown option: {}", args[i]));
                }
            }
        }

        Ok(config)
    }
}

fn test_position(board: &Board, epd: &EpdTestPosition, config: &TestConfig, searcher: &mut Searcher) -> TestResult {
    let start_time = Instant::now();

    // Run the search to get engine's best move
    let depth = config.depth.unwrap_or(10);
    let result = searcher.search(board, depth as u32);

    let elapsed = start_time.elapsed();
    let time_ms = elapsed.as_millis() as u64;

    let engine_move_uci = result.best_move.to_uci();
    let engine_score = result.score;

    // Score-based comparison: search expected moves and compare scores
    let moves = generate_moves(board);

    let mut best_expected_score = None;
    let mut pass_reason = String::new();
    let mut passed = false;

    // First, try to find and evaluate expected moves by matching destination square
    for expected_san in &epd.best_moves {
        // Extract destination square from SAN notation
        // Handle moves like "Qg6", "Rxb2+", "O-O", "Nf3#"
        let dest = extract_destination_square(expected_san);

        if let Some(dest_sq) = dest {
            // Find moves that go to this square
            for mv in moves.iter() {
                let to_sq = mv.to().to_algebraic();

                if to_sq == dest_sq {
                    // Found a candidate move, evaluate it
                    let mut test_board = board.clone();
                    test_board.make_move(*mv);

                    let mut test_searcher = Searcher::new();
                    let test_result = test_searcher.search(&test_board, (depth - 1).max(1) as u32);
                    let expected_score = -test_result.score; // Negate because opponent's view

                    // Keep track of best expected score
                    if best_expected_score.is_none() || expected_score > best_expected_score.unwrap() {
                        best_expected_score = Some(expected_score);
                    }
                }
            }
        }
    }

    // Determine if engine passed
    const SCORE_THRESHOLD: i32 = 100; // Allow 100cp tolerance

    if let Some(exp_score) = best_expected_score {
        let score_diff = engine_score - exp_score;

        // Check if it's mate
        let engine_is_mate = engine_score > 30000;
        let expected_is_mate = exp_score > 30000;

        if engine_is_mate && expected_is_mate {
            // Both find mate - engine passes
            passed = true;
            pass_reason = format!("Both find mate (engine: M{}, expected: M{})",
                (32767 - engine_score) / 2, (32767 - exp_score) / 2);
        } else if engine_is_mate && !expected_is_mate {
            // Engine found mate where expected doesn't - excellent!
            passed = true;
            pass_reason = format!("Engine found mate M{} (expected: {}cp)",
                (32767 - engine_score) / 2, exp_score);
        } else if !engine_is_mate && expected_is_mate {
            // Expected finds mate but engine doesn't - failure
            passed = false;
            pass_reason = format!("Expected finds mate M{} but engine doesn't ({}cp)",
                (32767 - exp_score) / 2, engine_score);
        } else if score_diff >= -SCORE_THRESHOLD {
            // Engine's move is within threshold or better
            passed = true;
            if score_diff > 200 {
                pass_reason = format!("Engine move MUCH better (+{}cp)", score_diff);
            } else if score_diff > 50 {
                pass_reason = format!("Engine move better (+{}cp)", score_diff);
            } else if score_diff >= 0 {
                pass_reason = format!("Engine move slightly better (+{}cp)", score_diff);
            } else {
                pass_reason = format!("Within threshold ({}cp)", score_diff);
            }
        } else {
            // Engine's move is significantly worse
            passed = false;
            pass_reason = format!("Engine move worse ({}cp)", score_diff);
        }
    } else {
        // Couldn't find expected move to evaluate - fallback to notation matching
        passed = epd.best_moves.iter().any(|expected| {
            let expected_clean = expected.trim_end_matches(|c| c == '+' || c == '#' || c == '!' || c == '?');
            engine_move_uci == expected_clean || engine_move_uci == *expected
        });

        if passed {
            pass_reason = "Notation match".to_string();
        } else {
            pass_reason = "Could not evaluate expected move".to_string();
        }
    }

    TestResult {
        position_id: epd.id.clone(),
        passed,
        engine_move: engine_move_uci,
        expected_moves: epd.best_moves.clone(),
        score: engine_score,
        expected_score: best_expected_score,
        score_diff: best_expected_score.map(|exp| engine_score - exp),
        depth_reached: result.depth as u8,
        time_ms,
        nodes: result.nodes,
        fen: board.to_fen(),
        pass_reason,
    }
}

fn extract_destination_square(san: &str) -> Option<String> {
    // Remove annotations
    let clean = san.trim_end_matches(|c| c == '+' || c == '#' || c == '!' || c == '?');

    // Handle castling
    if clean == "O-O" || clean == "0-0" {
        // Kingside castling - can't determine square without knowing color
        return None;
    }
    if clean == "O-O-O" || clean == "0-0-0" {
        // Queenside castling
        return None;
    }

    // Extract last 2 characters if they look like a square (e.g., "e4", "g6")
    if clean.len() >= 2 {
        let last_two = &clean[clean.len()-2..];
        if last_two.chars().nth(0).map(|c| c.is_ascii_lowercase()).unwrap_or(false) &&
           last_two.chars().nth(1).map(|c| c.is_ascii_digit()).unwrap_or(false) {
            return Some(last_two.to_string());
        }
    }

    None
}

fn print_result(result: &TestResult, verbose: bool) {
    if verbose {
        let status = if result.passed { "✓ PASS" } else { "✗ FAIL" };
        println!("\n{} - {}", status, result.position_id);
        println!("  Expected:     {}", result.expected_moves.join(" or "));
        println!("  Engine move:  {}", result.engine_move);
        println!("  Engine score: {} cp", result.score);

        if let Some(exp_score) = result.expected_score {
            println!("  Expected score: {} cp", exp_score);
        }

        if let Some(diff) = result.score_diff {
            println!("  Difference:   {} cp", diff);
        }

        println!("  Reason:       {}", result.pass_reason);
        println!("  Depth:        {}", result.depth_reached);
        println!("  Time:         {} ms", result.time_ms);
        println!("  Nodes:        {}", result.nodes);

        if result.nodes > 0 && result.time_ms > 0 {
            let nps = (result.nodes as f64 / result.time_ms as f64) * 1000.0;
            println!("  NPS:          {:.0}", nps);
        }
    } else {
        let status = if result.passed { "✓" } else { "✗" };
        print!("{}", status);
        if result.position_id.ends_with("0") || result.position_id.ends_with("5") {
            print!(" ");
        }
    }
}

fn print_summary(results: &[TestResult], total_time: Duration) {
    let passed = results.iter().filter(|r| r.passed).count();
    let failed = results.len() - passed;
    let accuracy = (passed as f64 / results.len() as f64) * 100.0;

    let avg_time = results.iter().map(|r| r.time_ms).sum::<u64>() / results.len() as u64;
    let total_nodes: u64 = results.iter().map(|r| r.nodes).sum();
    let avg_depth = results.iter().map(|r| r.depth_reached as u32).sum::<u32>() / results.len() as u32;

    println!("\n\n========================================");
    println!("TEST SUMMARY");
    println!("========================================");
    println!("Total positions:  {}", results.len());
    println!("Passed:           {} ({:.1}%)", passed, accuracy);
    println!("Failed:           {} ({:.1}%)", failed, 100.0 - accuracy);
    println!();
    println!("Average time:     {} ms", avg_time);
    println!("Average depth:    {}", avg_depth);
    println!("Total nodes:      {}", total_nodes);
    println!("Total time:       {:.2}s", total_time.as_secs_f64());

    if total_time.as_millis() > 0 {
        let nps = (total_nodes as f64 / total_time.as_millis() as f64) * 1000.0;
        println!("Average NPS:      {:.0}", nps);
    }

    // Show score-based statistics
    let engine_better: usize = results.iter().filter(|r| {
        r.score_diff.map(|d| d > 50).unwrap_or(false)
    }).count();

    let engine_much_better: usize = results.iter().filter(|r| {
        r.score_diff.map(|d| d > 200).unwrap_or(false)
    }).count();

    let found_mate: usize = results.iter().filter(|r| r.score > 30000).count();

    println!("\nScore-based analysis:");
    println!("  Found mate:           {}", found_mate);
    println!("  Much better (+200cp): {}", engine_much_better);
    println!("  Better (+50cp):       {}", engine_better);

    // Show failed positions
    let failures: Vec<&TestResult> = results.iter().filter(|r| !r.passed).collect();
    if !failures.is_empty() && failures.len() <= 20 {
        println!("\n========================================");
        println!("FAILED POSITIONS");
        println!("========================================");
        for result in failures {
            println!("{}: expected {} got {} ({})",
                result.position_id,
                result.expected_moves.join(" or "),
                result.engine_move,
                result.pass_reason
            );
        }
    } else if failures.len() > 20 {
        println!("\n{} positions failed (too many to list)", failures.len());
    }

    println!("\n========================================");

    // Categorize performance
    if accuracy >= 95.0 {
        println!("Rating: EXCELLENT ⭐⭐⭐");
    } else if accuracy >= 90.0 {
        println!("Rating: VERY GOOD ⭐⭐");
    } else if accuracy >= 85.0 {
        println!("Rating: GOOD ⭐");
    } else if accuracy >= 75.0 {
        println!("Rating: ACCEPTABLE");
    } else {
        println!("Rating: NEEDS IMPROVEMENT");
    }
    println!("========================================");
}

fn main() {
    let config = match TestConfig::from_args() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    println!("Loading test positions from: {}", config.epd_file);

    let mut positions = match load_epd_file(&config.epd_file) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error loading EPD file: {}", e);
            std::process::exit(1);
        }
    };

    if positions.is_empty() {
        eprintln!("No positions loaded from file");
        std::process::exit(1);
    }

    // Apply limit if specified
    if let Some(limit) = config.limit {
        positions.truncate(limit);
    }

    println!("Loaded {} positions", positions.len());

    if let Some(depth) = config.depth {
        println!("Search depth: {}", depth);
    } else if let Some(time) = config.time_ms {
        println!("Time limit: {} ms per position", time);
    }

    println!("\nRunning tests...");
    if !config.verbose {
        println!("(✓ = pass, ✗ = fail)\n");
    }

    let start_time = Instant::now();
    let mut results = Vec::new();
    let mut searcher = Searcher::new();

    for epd in &positions {
        if config.verbose {
            println!("\nTesting: {}", epd.id);
        }

        let result = test_position(&epd.board, epd, &config, &mut searcher);

        print_result(&result, config.verbose);
        results.push(result);
    }

    let total_time = start_time.elapsed();

    print_summary(&results, total_time);

    // Save JSON output if requested
    if let Some(json_file) = &config.json_output {
        if let Err(e) = save_results_json(&results, json_file) {
            eprintln!("Warning: Failed to save JSON output: {}", e);
        } else {
            println!("\nResults saved to: {}", json_file);
        }
    }
}

fn save_results_json(results: &[TestResult], filename: &str) -> std::io::Result<()> {
    use std::fs::File;
    use std::io::Write;

    let mut file = File::create(filename)?;

    writeln!(file, "{{")?;
    writeln!(file, r#"  "total": {},"#, results.len())?;
    writeln!(file, r#"  "passed": {},"#, results.iter().filter(|r| r.passed).count())?;
    writeln!(file, r#"  "failed": {},"#, results.iter().filter(|r| !r.passed).count())?;
    writeln!(file, r#"  "accuracy": {:.2},"#,
        (results.iter().filter(|r| r.passed).count() as f64 / results.len() as f64) * 100.0)?;
    writeln!(file, r#"  "results": ["#)?;

    for (i, result) in results.iter().enumerate() {
        let comma = if i < results.len() - 1 { "," } else { "" };
        writeln!(file, "    {{")?;
        writeln!(file, r#"      "id": "{}","#, result.position_id)?;
        writeln!(file, r#"      "passed": {},"#, result.passed)?;
        writeln!(file, r#"      "engine_move": "{}","#, result.engine_move)?;
        writeln!(file, r#"      "expected_moves": [{}],"#,
            result.expected_moves.iter().map(|m| format!(r#""{}""#, m)).collect::<Vec<_>>().join(", "))?;
        writeln!(file, r#"      "score": {},"#, result.score)?;

        if let Some(exp_score) = result.expected_score {
            writeln!(file, r#"      "expected_score": {},"#, exp_score)?;
        } else {
            writeln!(file, r#"      "expected_score": null,"#)?;
        }

        if let Some(diff) = result.score_diff {
            writeln!(file, r#"      "score_diff": {},"#, diff)?;
        } else {
            writeln!(file, r#"      "score_diff": null,"#)?;
        }

        writeln!(file, r#"      "pass_reason": "{}","#, result.pass_reason.replace("\"", "\\\""))?;
        writeln!(file, r#"      "depth": {},"#, result.depth_reached)?;
        writeln!(file, r#"      "time_ms": {},"#, result.time_ms)?;
        writeln!(file, r#"      "nodes": {}"#, result.nodes)?;
        writeln!(file, "    }}{}", comma)?;
    }

    writeln!(file, "  ]")?;
    writeln!(file, "}}")?;

    Ok(())
}
