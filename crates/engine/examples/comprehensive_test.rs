use engine::io::parse_fen;
use engine::search::Searcher;

struct TestCase {
    name: &'static str,
    fen: &'static str,
    expected_move: &'static str, // Expected best move in UCI format
    description: &'static str,
    depth: u32,
}

fn main() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║      COMPREHENSIVE CHESS ENGINE TACTICAL TEST SUITE       ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    let test_cases = vec![
        // ===== BASIC TACTICS =====
        TestCase {
            name: "Hanging Piece #1",
            fen: "rnbqkb1r/pppppppp/5n2/8/3N4/8/PPPPPPPP/RNBQKB1R b KQkq - 0 1",
            expected_move: "f6d4", // or any move capturing the knight
            description: "Black should capture the free knight on d4",
            depth: 6,
        },
        TestCase {
            name: "Hanging Queen",
            fen: "rnbqkb1r/pppppppp/5n2/8/4Q3/8/PPPPPPPP/RNB1KBNR b KQkq - 0 1",
            expected_move: "f6e4",
            description: "Black should capture the free queen",
            depth: 6,
        },
        TestCase {
            name: "Back Rank Mate #1",
            fen: "6k1/5ppp/8/8/8/8/5PPP/R5K1 w - - 0 1",
            expected_move: "a1a8",
            description: "White delivers back rank mate",
            depth: 4,
        },
        TestCase {
            name: "Scholar's Mate",
            fen: "r1bqkb1r/pppp1ppp/2n2n2/4p2Q/2B1P3/8/PPPP1PPP/RNB1K1NR w KQkq - 0 1",
            expected_move: "h5f7",
            description: "Qxf7# is checkmate",
            depth: 4,
        },
        TestCase {
            name: "Simple Fork",
            fen: "rnbqkb1r/pppppppp/8/8/8/2N5/PPPPPPPP/R1BQKBNR w KQkq - 0 1",
            expected_move: "c3d5", // Fork king and something, or other strong move
            description: "Knight fork opportunity",
            depth: 6,
        },

        // ===== KING SAFETY =====
        TestCase {
            name: "King Safety - Don't expose king",
            fen: "rnbqkb1r/pppp1ppp/5n2/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 0 1",
            expected_move: "f1d3", // Or similar development, NOT g2g4 exposing king
            description: "Develop pieces, don't weaken king",
            depth: 6,
        },

        // ===== MATERIAL TRADES =====
        TestCase {
            name: "Win Pawn Cleanly",
            fen: "rnbqkb1r/pppp1ppp/5n2/4p3/4P3/3P1N2/PPP2PPP/RNBQKB1R w KQkq - 0 1",
            expected_move: "f3e5",
            description: "Nxe5 wins a pawn cleanly",
            depth: 6,
        },
        TestCase {
            name: "Don't Sacrifice Knight for Pawn",
            fen: "rnbqkb1r/pppp1ppp/5n2/4p3/4P3/3P1N2/PPP2PPP/RNBQKB1R w KQkq - 0 1",
            expected_move: "f3e5", // NOT Nxf7 which loses the knight
            description: "Take the free pawn, don't sacrifice knight",
            depth: 6,
        },
        TestCase {
            name: "Free Piece - Don't Miss It",
            fen: "r1bqkbnr/pppp1ppp/2n5/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R b KQkq - 0 1",
            expected_move: "c6d4", // Attack bishop or other strong move
            description: "Black should attack or capture",
            depth: 6,
        },

        // ===== POSITIONAL =====
        TestCase {
            name: "Develop Pieces",
            fen: "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1",
            expected_move: "b8c6", // or g8f6, or other development
            description: "Develop knights toward center",
            depth: 6,
        },
        TestCase {
            name: "Don't Move Same Piece Twice in Opening",
            fen: "r1bqkbnr/pppp1ppp/2n5/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 0 1",
            expected_move: "f1c4", // Develop, not moving knight again
            description: "Develop new pieces in opening",
            depth: 6,
        },

        // ===== SIMPLE MATES =====
        TestCase {
            name: "Mate in 2 - Back Rank",
            fen: "6k1/5ppp/8/8/8/8/5PPP/1R4K1 w - - 0 1",
            expected_move: "b1b8",
            description: "Rb8+ leads to mate in 2",
            depth: 6,
        },
        TestCase {
            name: "Smothered Mate Setup",
            fen: "r1bqk2r/pppp1ppp/2n2n2/2b1p3/2B1P3/3P1N2/PPP2PPP/RNBQ1RK1 b kq - 0 1",
            expected_move: "c6d4", // Good move, checks/develops
            description: "Find strong tactical moves",
            depth: 6,
        },

        // ===== PIECE COORDINATION =====
        TestCase {
            name: "Coordinate Attack",
            fen: "r1bqk2r/pppp1ppp/2n2n2/2b1p3/2B1P3/2NP1N2/PPP2PPP/R1BQK2R w KQkq - 0 1",
            expected_move: "c3d5", // Or other attacking move
            description: "Create threats with coordinated pieces",
            depth: 6,
        },

        // ===== ENDGAME =====
        TestCase {
            name: "KPK - Push Passed Pawn",
            fen: "8/8/8/4k3/8/8/4P3/4K3 w - - 0 1",
            expected_move: "e2e4", // Or e1d2 activating king
            description: "In KPK, push the pawn and activate king",
            depth: 8,
        },
        TestCase {
            name: "Rook Endgame - Cut Off King",
            fen: "8/8/8/4k3/8/8/4P3/R3K3 w - - 0 1",
            expected_move: "a1a5", // Cut off black king
            description: "Use rook to cut off opponent king",
            depth: 8,
        },

        // ===== DON'T HANG PIECES =====
        TestCase {
            name: "Don't Hang Bishop",
            fen: "rnbqkbnr/pppp1ppp/8/4p3/2B1P3/8/PPPP1PPP/RNBQK1NR b KQkq - 0 1",
            expected_move: "d8g5", // Attack bishop, don't let it get trapped
            description: "Don't allow pieces to be trapped",
            depth: 6,
        },
        TestCase {
            name: "Don't Walk Into Pin",
            fen: "r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 0 1",
            expected_move: "e1g1", // Castle, not other moves that hang pieces
            description: "Be aware of pins and tactical threats",
            depth: 6,
        },
    ];

    let mut passed = 0;
    let mut failed = 0;
    let mut failures = Vec::new();

    for (idx, test) in test_cases.iter().enumerate() {
        println!("╔═══════════════════════════════════════════════════════════");
        println!("║ Test {}/{}: {}", idx + 1, test_cases.len(), test.name);
        println!("╠═══════════════════════════════════════════════════════════");
        println!("║ Description: {}", test.description);
        println!("║ FEN: {}", test.fen);
        println!("║ Expected: {} at depth {}", test.expected_move, test.depth);
        println!("╚═══════════════════════════════════════════════════════════");

        let board = match parse_fen(test.fen) {
            Ok(b) => b,
            Err(e) => {
                println!("❌ INVALID FEN: {}\n", e);
                failed += 1;
                continue;
            }
        };

        let mut searcher = Searcher::new();
        let result = searcher.search(&board, test.depth);

        let best_move = result.best_move.to_uci();
        let score = result.score;
        let pv = result.pv.iter().map(|m| m.to_uci()).collect::<Vec<_>>().join(" ");

        println!("ENGINE RESULT:");
        println!("  Best move: {}", best_move);
        println!("  Score: {} cp", score);
        println!("  PV: {}", pv);
        println!("  Nodes: {}", result.nodes);

        // Check if move matches (could be multiple correct moves)
        let expected_moves: Vec<&str> = test.expected_move.split('|').collect();
        let found_expected = expected_moves.iter().any(|&m| best_move == m || pv.contains(m));

        if found_expected {
            println!("✅ PASS\n");
            passed += 1;
        } else {
            println!("❌ FAIL - Expected {} but got {}\n", test.expected_move, best_move);
            failed += 1;
            failures.push((test.name, test.expected_move, best_move.clone(), score));
        }
    }

    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║                      FINAL RESULTS                         ║");
    println!("╠════════════════════════════════════════════════════════════╣");
    println!("║ Total Tests: {}", test_cases.len());
    println!("║ Passed: {} ({}%)", passed, (passed * 100) / test_cases.len());
    println!("║ Failed: {} ({}%)", failed, (failed * 100) / test_cases.len());
    println!("╚════════════════════════════════════════════════════════════╝");

    if !failures.is_empty() {
        println!("\n╔════════════════════════════════════════════════════════════╗");
        println!("║                      FAILED TESTS                          ║");
        println!("╚════════════════════════════════════════════════════════════╝");
        for (name, expected, actual, score) in failures {
            println!("❌ {}", name);
            println!("   Expected: {}", expected);
            println!("   Got: {} (score: {} cp)", actual, score);
            println!();
        }
    }

    if failed == 0 {
        println!("\n🎉 ALL TESTS PASSED! Engine is playing correctly!");
    } else if (passed * 100) / test_cases.len() >= 80 {
        println!("\n⚠️  Engine passed most tests but has some tactical issues");
    } else {
        println!("\n❌ Engine has significant tactical problems - needs investigation");
    }
}
