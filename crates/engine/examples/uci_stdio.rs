/// Standard UCI interface using stdin/stdout for testing with cutechess/fastchess
use engine::board::Board;
use engine::io::parse_fen;
use engine::movegen::generate_moves;
use engine::search::core::Searcher;
use engine::search_params;
use std::io::{self, BufRead, Write};

/// Get parameter bounds for UCI option reporting.
fn get_param_bounds(param_name: &str) -> (i32, i32) {
    match param_name {
        "lmr_base_reduction" => (1, 4),
        "lmr_move_threshold" => (2, 10),
        "lmr_depth_threshold" => (2, 8),
        "null_move_r" => (2, 4),
        "null_move_min_depth" => (2, 5),
        "futility_margin_d1" => (50, 200),
        "futility_margin_d2" => (100, 300),
        "futility_margin_d3" => (200, 400),
        "rfp_margin_d1" => (50, 200),
        "rfp_margin_d2" => (100, 300),
        "rfp_margin_d3" => (200, 400),
        "rfp_margin_d4" => (300, 500),
        "rfp_margin_d5" => (400, 600),
        "razor_margin_d1" => (100, 300),
        "razor_margin_d2" => (200, 400),
        "razor_margin_d3" => (300, 500),
        "lmp_threshold_d1" => (2, 6),
        "lmp_threshold_d2" => (4, 10),
        "lmp_threshold_d3" => (8, 20),
        "aspiration_delta" => (20, 100),
        "iid_depth_reduction" => (1, 4),
        "iir_depth_reduction" => (1, 3),
        "iid_min_depth" => (2, 6),
        "singular_margin" => (50, 200),
        "singular_depth_reduction" => (2, 5),
        "singular_min_depth" => (6, 12),
        "king_safety_divisor" => (6, 20),
        _ => (0, 1000), // Default bounds
    }
}

fn main() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut board = Board::default();
    let mut searcher = Searcher::new();
    let mut fixed_depth: Option<u32> = None; // For limiting strength via depth

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l.trim().to_string(),
            Err(_) => break,
        };

        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        match parts[0] {
            "uci" => {
                writeln!(stdout, "id name ChessAI v1.0").unwrap();
                writeln!(stdout, "id author Claude & Victor").unwrap();

                // Report tunable parameters as UCI options
                for param_name in search_params::SearchParams::param_names() {
                    if let Ok(value) = search_params::get_param(param_name) {
                        // Determine sensible min/max based on parameter name
                        let (min, max) = get_param_bounds(param_name);
                        writeln!(stdout, "option name {} type spin default {} min {} max {}",
                                param_name, value, min, max).unwrap();
                    }
                }

                // Add FixedDepth option for strength limitation
                writeln!(stdout, "option name FixedDepth type spin default 0 min 0 max 20").unwrap();

                writeln!(stdout, "uciok").unwrap();
                stdout.flush().unwrap();
            }

            "isready" => {
                writeln!(stdout, "readyok").unwrap();
                stdout.flush().unwrap();
            }

            "ucinewgame" => {
                board = Board::default();
                searcher = Searcher::new();
            }

            "position" => {
                if parts.len() < 2 {
                    continue;
                }

                if parts[1] == "startpos" {
                    board = Board::default();

                    // Apply moves if provided
                    if parts.len() > 2 && parts[2] == "moves" {
                        for move_str in &parts[3..] {
                            let moves = generate_moves(&board);
                            let mv_uci_strings: Vec<String> = moves.iter().map(|m| m.to_uci()).collect();
                            if let Some(idx) = mv_uci_strings.iter().position(|uci| uci == *move_str) {
                                board.make_move(moves[idx]);
                            }
                        }
                    }
                } else if parts[1] == "fen" {
                    // Find where "moves" starts (if present)
                    let moves_idx = parts.iter().position(|&p| p == "moves");
                    let fen_end = moves_idx.unwrap_or(parts.len());

                    // Reconstruct FEN (parts 2 through fen_end)
                    let fen = parts[2..fen_end].join(" ");

                    if let Ok(new_board) = parse_fen(&fen) {
                        board = new_board;

                        // Apply moves if provided
                        if let Some(idx) = moves_idx {
                            for move_str in &parts[idx + 1..] {
                                let moves = generate_moves(&board);
                                let mv_uci_strings: Vec<String> = moves.iter().map(|m| m.to_uci()).collect();
                                if let Some(mv_idx) = mv_uci_strings.iter().position(|uci| uci == *move_str) {
                                    board.make_move(moves[mv_idx]);
                                }
                            }
                        }
                    }
                }
            }

            "go" => {
                // Parse go command
                let mut depth = 8u32;
                let mut movetime: Option<u64> = None;
                let mut wtime: Option<u64> = None;
                let mut btime: Option<u64> = None;
                let mut winc: Option<u64> = None;
                let mut binc: Option<u64> = None;

                let mut i = 1;
                while i < parts.len() {
                    match parts[i] {
                        "depth" => {
                            if i + 1 < parts.len() {
                                depth = parts[i + 1].parse().unwrap_or(8);
                                i += 2;
                            } else {
                                i += 1;
                            }
                        }
                        "movetime" => {
                            if i + 1 < parts.len() {
                                movetime = parts[i + 1].parse().ok();
                                i += 2;
                            } else {
                                i += 1;
                            }
                        }
                        "wtime" => {
                            if i + 1 < parts.len() {
                                wtime = parts[i + 1].parse().ok();
                                i += 2;
                            } else {
                                i += 1;
                            }
                        }
                        "btime" => {
                            if i + 1 < parts.len() {
                                btime = parts[i + 1].parse().ok();
                                i += 2;
                            } else {
                                i += 1;
                            }
                        }
                        "winc" => {
                            if i + 1 < parts.len() {
                                winc = parts[i + 1].parse().ok();
                                i += 2;
                            } else {
                                i += 1;
                            }
                        }
                        "binc" => {
                            if i + 1 < parts.len() {
                                binc = parts[i + 1].parse().ok();
                                i += 2;
                            } else {
                                i += 1;
                            }
                        }
                        "infinite" => {
                            depth = 20;
                            i += 1;
                        }
                        _ => {
                            i += 1;
                        }
                    }
                }

                // Use fixed depth if set, otherwise do time management
                if let Some(fixed) = fixed_depth {
                    depth = fixed;
                } else if movetime.is_none() {
                    use engine::piece::Color;
                    let our_time = if board.side_to_move() == Color::White {
                        wtime.unwrap_or(60000) // Default 60 seconds
                    } else {
                        btime.unwrap_or(60000)
                    };

                    // Use 1/30th of time or 1 second minimum
                    let time_for_move = std::cmp::max(our_time / 30, 1000);

                    // Estimate depth based on time (very rough)
                    depth = if time_for_move < 100 {
                        5
                    } else if time_for_move < 500 {
                        6
                    } else if time_for_move < 2000 {
                        7
                    } else if time_for_move < 5000 {
                        8
                    } else {
                        9
                    };
                }

                // Search
                let start = std::time::Instant::now();
                let result = searcher.search(&board, depth);
                let elapsed_ms = start.elapsed().as_millis() as u64;

                // Output info with score
                let score_cp = result.score;
                let nps = if elapsed_ms > 0 { result.nodes * 1000 / elapsed_ms } else { 0 };
                writeln!(stdout, "info depth {} score cp {} nodes {} nps {} time {}",
                    result.depth, score_cp, result.nodes, nps, elapsed_ms
                ).unwrap();

                // Output best move
                writeln!(stdout, "bestmove {}", result.best_move.to_uci()).unwrap();
                stdout.flush().unwrap();
            }

            "setoption" => {
                // Parse: setoption name <id> value <x>
                if parts.len() >= 5 && parts[1] == "name" && parts[3] == "value" {
                    let param_name = parts[2];

                    // Handle FixedDepth option
                    if param_name == "FixedDepth" {
                        if let Ok(value) = parts[4].parse::<u32>() {
                            fixed_depth = if value > 0 { Some(value) } else { None };
                        }
                    } else if let Ok(value) = parts[4].parse::<i32>() {
                        if let Err(e) = search_params::set_param(param_name, value) {
                            eprintln!("Error setting parameter {}: {}", param_name, e);
                        }
                    }
                }
            }

            "d" | "display" => {
                // Debug command - show board
                use engine::io::ToFen;
                eprintln!("{}", board.to_fen());
            }

            "quit" => {
                break;
            }

            _ => {
                // Unknown command, ignore
            }
        }
    }
}
