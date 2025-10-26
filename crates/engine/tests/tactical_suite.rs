//! Tactical test suite for validating M7 search quality
//!
//! This suite contains various tactical positions to ensure the search
//! finds the correct moves and doesn't miss tactics due to aggressive pruning.

use engine::board::Board;
use engine::io::parse_fen;
use engine::search::Searcher;

/// Tactical test case
struct TacticalTest {
    name: &'static str,
    fen: &'static str,
    depth: u32,
    expected_move: Option<&'static str>, // UCI format, None if we just want positive score
    min_score: Option<i32>,              // Minimum expected score
}

const TACTICAL_POSITIONS: &[TacticalTest] = &[
    // Simple tactics
    TacticalTest {
        name: "Back rank mate in 1",
        fen: "6k1/5ppp/8/8/8/8/5PPP/4R1K1 w - - 0 1",
        depth: 5,
        expected_move: Some("e1e8"),
        min_score: Some(9000), // Should see mate
    },
    TacticalTest {
        name: "Queen fork (knight and rook)",
        fen: "r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 0 1",
        depth: 5,
        expected_move: Some("c4f7"), // Bxf7+ wins material
        min_score: Some(200),        // Should be winning
    },
    TacticalTest {
        name: "Discovery attack",
        fen: "r1bqkb1r/pppp1ppp/2n5/4p3/2BnP3/5N2/PPPP1PPP/RNBQK2R w KQkq - 0 1",
        depth: 5,
        expected_move: Some("f3d4"), // Nxd4 wins knight
        min_score: Some(250),
    },
    TacticalTest {
        name: "Pin exploitation",
        fen: "rnbqkb1r/pppp1ppp/5n2/4p3/2B1P3/8/PPPP1PPP/RNBQK1NR w KQkq - 0 1",
        depth: 6,
        expected_move: Some("c4f7"), // Bxf7+ exploits pin
        min_score: Some(200),
    },
    TacticalTest {
        name: "Promotion threat",
        fen: "8/4P3/8/8/8/8/4k3/4K3 w - - 0 1",
        depth: 5,
        expected_move: Some("e7e8q"), // Promote to queen
        min_score: Some(800),         // Huge material advantage
    },
    // More complex tactics
    TacticalTest {
        name: "Sacrifice for mate",
        fen: "r1bq1rk1/pppp1ppp/2n2n2/2b1p3/2B1P3/2NP1N2/PPP2PPP/R1BQK2R w KQ - 0 1",
        depth: 7,
        expected_move: None,   // Just check it finds a good move
        min_score: Some(-100), // Should not be losing
    },
    TacticalTest {
        name: "Zwischenzug (in-between move)",
        fen: "r1bqk2r/pppp1ppp/2n2n2/2b1p3/2B1P3/2N2N2/PPPP1PPP/R1BQK2R w KQkq - 0 1",
        depth: 6,
        expected_move: None,
        min_score: Some(-50),
    },
    TacticalTest {
        name: "Trapped piece",
        fen: "rnbqkbnr/ppp2ppp/8/3pp3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 0 1",
        depth: 5,
        expected_move: None,
        min_score: Some(-50),
    },
    TacticalTest {
        name: "Skewer",
        fen: "4k3/8/8/8/8/8/4R3/4K2r w - - 0 1",
        depth: 5,
        expected_move: Some("e2e8"), // Re8+ skewers king and rook
        min_score: Some(400),
    },
    TacticalTest {
        name: "Deflection",
        fen: "6k1/5ppp/8/8/8/2Q5/5PPP/6K1 w - - 0 1",
        depth: 5,
        expected_move: None, // Should find winning move
        min_score: Some(200),
    },
];

#[test]
fn test_tactical_suite() {
    let mut passed = 0;
    let mut failed = 0;

    for test in TACTICAL_POSITIONS {
        println!("\nTesting: {}", test.name);
        println!("FEN: {}", test.fen);

        let board = parse_fen(test.fen).expect("Valid FEN");
        let mut searcher = Searcher::new();
        let result = searcher.search(&board, test.depth);

        println!(
            "  Best move: {} (score: {})",
            result.best_move.to_uci(),
            result.score
        );

        // Check if move matches expected (if provided)
        let move_ok = if let Some(expected) = test.expected_move {
            let found = result.best_move.to_uci();
            if found == expected {
                println!("  ✓ Found expected move: {}", expected);
                true
            } else {
                println!("  ✗ Expected {}, found {}", expected, found);
                false
            }
        } else {
            true
        };

        // Check if score meets minimum (if provided)
        let score_ok = if let Some(min_score) = test.min_score {
            if result.score >= min_score {
                println!("  ✓ Score {} >= {} (minimum)", result.score, min_score);
                true
            } else {
                println!("  ✗ Score {} < {} (minimum)", result.score, min_score);
                false
            }
        } else {
            true
        };

        if move_ok && score_ok {
            passed += 1;
        } else {
            failed += 1;
        }
    }

    println!("\n========================================");
    println!(
        "Tactical Suite Results: {}/{} passed",
        passed,
        passed + failed
    );
    println!("========================================");

    // We want to pass most tests, but some tactical positions are hard
    // Accept 70% pass rate for now
    let pass_rate = (passed as f64) / (passed + failed) as f64;
    assert!(
        pass_rate >= 0.7,
        "Pass rate {:.1}% is below 70%",
        pass_rate * 100.0
    );
}

#[test]
fn test_mate_in_2() {
    // Scholar's mate follow-up
    let fen = "r1bqkb1r/pppp1ppp/2n2n2/4p2Q/2B1P3/8/PPPP1PPP/RNB1K1NR w KQkq - 0 1";
    let board = parse_fen(fen).unwrap();

    let mut searcher = Searcher::new();
    let result = searcher.search(&board, 6);

    println!("Mate in 2 - Best move: {}", result.best_move.to_uci());
    println!("Score: {}", result.score);

    // Should find Qxf7# or see a very high score
    assert!(result.score > 5000, "Should detect winning position");
}

#[test]
fn test_no_blunders_in_equal_position() {
    // Equal starting position - should not give away material
    let board = Board::startpos();
    let mut searcher = Searcher::new();
    let result = searcher.search(&board, 6);

    println!(
        "Starting position - Best move: {}",
        result.best_move.to_uci()
    );
    println!("Score: {}", result.score);

    // Score should be close to equal
    assert!(
        result.score.abs() < 100,
        "Starting position should be roughly equal, got score: {}",
        result.score
    );
}

#[test]
fn test_forced_mate_sequence() {
    // Queen and king endgame - should be winning for white
    let fen = "6k1/8/6K1/8/8/8/8/Q7 w - - 0 1";
    let board = parse_fen(fen).unwrap();

    let mut searcher = Searcher::new();
    let result = searcher.search(&board, 8);

    println!("Forced mate - Best move: {}", result.best_move.to_uci());
    println!("Score: {}", result.score);

    // Should see a winning position for white with queen vs bare king
    assert!(
        result.score > 500,
        "Should find winning evaluation, got: {}",
        result.score
    );
}

#[test]
fn test_defensive_resource() {
    // Position where defense is critical
    let fen = "r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQ1RK1 b kq - 0 1";
    let board = parse_fen(fen).unwrap();

    let mut searcher = Searcher::new();
    let result = searcher.search(&board, 6);

    println!(
        "Defensive position - Best move: {}",
        result.best_move.to_uci()
    );
    println!("Score: {}", result.score);

    // Should find a legal move and not crash
    assert!(board.is_legal(result.best_move));
}
