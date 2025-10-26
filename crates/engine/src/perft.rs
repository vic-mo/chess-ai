/// Performance test (Perft) for move generation validation.
///
/// Perft recursively counts all leaf nodes at a given depth. It's the gold standard
/// for validating move generation correctness by comparing against canonical values.
use crate::board::Board;

/// Count all leaf nodes at the given depth.
///
/// # Arguments
/// * `board` - The position to test
/// * `depth` - The search depth (0 = count this position)
///
/// # Returns
/// The number of leaf nodes at the given depth
///
/// # Example
/// ```
/// use engine::board::Board;
/// use engine::perft::perft;
///
/// let board = Board::startpos();
/// assert_eq!(perft(&board, 0), 1);
/// assert_eq!(perft(&board, 1), 20);
/// assert_eq!(perft(&board, 2), 400);
/// ```
pub fn perft(board: &Board, depth: u32) -> u64 {
    if depth == 0 {
        return 1;
    }

    let mut nodes = 0u64;
    let moves = board.generate_legal_moves();

    // At depth 1, just count the moves (optimization)
    if depth == 1 {
        return moves.len() as u64;
    }

    // Recurse for deeper depths
    for m in moves.iter() {
        let mut new_board = board.clone();
        new_board.make_move(*m);
        nodes += perft(&new_board, depth - 1);
    }

    nodes
}

/// Perft with per-move breakdown at the root.
///
/// This is useful for debugging - it shows which moves lead to which counts,
/// making it easier to identify where move generation differs from expected.
///
/// # Example
/// ```
/// use engine::board::Board;
/// use engine::perft::perft_divide;
///
/// let board = Board::startpos();
/// let results = perft_divide(&board, 2);
/// assert_eq!(results.len(), 20); // 20 legal moves from start
/// ```
pub fn perft_divide(board: &Board, depth: u32) -> Vec<(String, u64)> {
    let mut results = Vec::new();
    let moves = board.generate_legal_moves();

    for m in moves.iter() {
        let mut new_board = board.clone();
        new_board.make_move(*m);

        let count = if depth <= 1 {
            1
        } else {
            perft(&new_board, depth - 1)
        };

        results.push((m.to_string(), count));
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::parse_fen;

    #[test]
    fn test_perft_startpos_depth_0() {
        let board = Board::startpos();
        assert_eq!(perft(&board, 0), 1);
    }

    #[test]
    fn test_perft_startpos_depth_1() {
        let board = Board::startpos();
        assert_eq!(perft(&board, 1), 20);
    }

    #[test]
    fn test_perft_startpos_depth_2() {
        let board = Board::startpos();
        assert_eq!(perft(&board, 2), 400);
    }

    #[test]
    fn test_perft_startpos_depth_3() {
        let board = Board::startpos();
        assert_eq!(perft(&board, 3), 8_902);
    }

    #[test]
    fn test_perft_startpos_depth_4() {
        let board = Board::startpos();
        assert_eq!(perft(&board, 4), 197_281);
    }

    #[test]
    #[ignore] // Slow test - run with --ignored
    fn test_perft_startpos_depth_5() {
        let board = Board::startpos();
        assert_eq!(perft(&board, 5), 4_865_609);
    }

    #[test]
    #[ignore] // Very slow test - run with --ignored
    fn test_perft_startpos_depth_6() {
        let board = Board::startpos();
        assert_eq!(perft(&board, 6), 119_060_324);
    }

    // Kiwipete position - complex middle game position with lots of edge cases
    // https://www.chessprogramming.org/Perft_Results
    #[test]
    fn test_perft_kiwipete_depth_1() {
        let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
        let board = parse_fen(fen).unwrap();
        assert_eq!(perft(&board, 1), 48);
    }

    #[test]
    fn test_perft_kiwipete_depth_2() {
        let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
        let board = parse_fen(fen).unwrap();
        assert_eq!(perft(&board, 2), 2_039);
    }

    #[test]
    fn test_perft_kiwipete_depth_3() {
        let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
        let board = parse_fen(fen).unwrap();
        assert_eq!(perft(&board, 3), 97_862);
    }

    #[test]
    #[ignore] // Slow test
    fn test_perft_kiwipete_depth_4() {
        let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
        let board = parse_fen(fen).unwrap();
        assert_eq!(perft(&board, 4), 4_085_603);
    }

    // Position 3 - many castling rights but no castling moves
    #[test]
    fn test_perft_position3_depth_1() {
        let fen = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1";
        let board = parse_fen(fen).unwrap();
        assert_eq!(perft(&board, 1), 14);
    }

    #[test]
    fn test_perft_position3_depth_2() {
        let fen = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1";
        let board = parse_fen(fen).unwrap();
        assert_eq!(perft(&board, 2), 191);
    }

    #[test]
    fn test_perft_position3_depth_3() {
        let fen = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1";
        let board = parse_fen(fen).unwrap();
        assert_eq!(perft(&board, 3), 2_812);
    }

    #[test]
    fn test_perft_position3_depth_4() {
        let fen = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1";
        let board = parse_fen(fen).unwrap();
        assert_eq!(perft(&board, 4), 43_238);
    }

    #[test]
    #[ignore] // Slow test
    fn test_perft_position3_depth_5() {
        let fen = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1";
        let board = parse_fen(fen).unwrap();
        assert_eq!(perft(&board, 5), 674_624);
    }

    // Position 4 - promotions and discovered checks
    #[test]
    fn test_perft_position4_depth_1() {
        let fen = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1";
        let board = parse_fen(fen).unwrap();
        assert_eq!(perft(&board, 1), 6);
    }

    #[test]
    fn test_perft_position4_depth_2() {
        let fen = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1";
        let board = parse_fen(fen).unwrap();
        assert_eq!(perft(&board, 2), 264);
    }

    #[test]
    fn test_perft_position4_depth_3() {
        let fen = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1";
        let board = parse_fen(fen).unwrap();
        assert_eq!(perft(&board, 3), 9_467);
    }

    #[test]
    #[ignore] // Slow test
    fn test_perft_position4_depth_4() {
        let fen = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1";
        let board = parse_fen(fen).unwrap();
        assert_eq!(perft(&board, 4), 422_333);
    }

    // Position 5 - discovered checks and pinned pieces
    #[test]
    fn test_perft_position5_depth_1() {
        let fen = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8";
        let board = parse_fen(fen).unwrap();
        assert_eq!(perft(&board, 1), 44);
    }

    #[test]
    fn test_perft_position5_depth_2() {
        let fen = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8";
        let board = parse_fen(fen).unwrap();
        assert_eq!(perft(&board, 2), 1_486);
    }

    #[test]
    fn test_perft_position5_depth_3() {
        let fen = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8";
        let board = parse_fen(fen).unwrap();
        assert_eq!(perft(&board, 3), 62_379);
    }

    #[test]
    #[ignore] // Slow test
    fn test_perft_position5_depth_4() {
        let fen = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8";
        let board = parse_fen(fen).unwrap();
        assert_eq!(perft(&board, 4), 2_103_487);
    }

    // Position 6 - en passant
    #[test]
    fn test_perft_position6_depth_1() {
        let fen = "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10";
        let board = parse_fen(fen).unwrap();
        assert_eq!(perft(&board, 1), 46);
    }

    #[test]
    fn test_perft_position6_depth_2() {
        let fen = "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10";
        let board = parse_fen(fen).unwrap();
        assert_eq!(perft(&board, 2), 2_079);
    }

    #[test]
    fn test_perft_position6_depth_3() {
        let fen = "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10";
        let board = parse_fen(fen).unwrap();
        assert_eq!(perft(&board, 3), 89_890);
    }

    #[test]
    #[ignore] // Slow test
    fn test_perft_position6_depth_4() {
        let fen = "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10";
        let board = parse_fen(fen).unwrap();
        assert_eq!(perft(&board, 4), 3_894_594);
    }

    #[test]
    fn test_perft_divide_startpos() {
        let board = Board::startpos();
        let results = perft_divide(&board, 1);

        // Should have 20 moves
        assert_eq!(results.len(), 20);

        // Each move at depth 1 should have count of 1
        for (_, count) in results.iter() {
            assert_eq!(*count, 1);
        }
    }

    #[test]
    fn test_perft_divide_startpos_depth_2() {
        let board = Board::startpos();
        let results = perft_divide(&board, 2);

        // Should have 20 moves
        assert_eq!(results.len(), 20);

        // Total should be 400
        let total: u64 = results.iter().map(|(_, count)| count).sum();
        assert_eq!(total, 400);
    }

    #[test]
    #[ignore] // Debug test
    fn debug_startpos_depth_2() {
        let board = Board::startpos();
        let results = perft_divide(&board, 2);

        println!("\n=== Startpos depth 2 breakdown ===");
        let mut total = 0u64;
        for (mv, count) in results.iter() {
            total += count;
            println!("{}: {}", mv, count);
        }
        println!("Total: {} (expected: 400)", total);
    }

    #[test]
    #[ignore] // Debug test
    fn debug_position4_depth_1() {
        let fen = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1";
        let board = parse_fen(fen).unwrap();
        let results = perft_divide(&board, 1);

        println!("\n=== Position 4 depth 1 breakdown ===");
        println!("Current perft(1): {}", perft(&board, 1));
        println!("Expected: 6\n");
        println!("Moves generated:");
        for (mv, _) in results.iter() {
            println!("  {}", mv);
        }
    }

    #[test]
    #[ignore] // Debug test
    fn debug_check_position() {
        use crate::movegen::generate_moves;
        use crate::piece::Color;

        let fen = "rnb1kbnr/pppp1ppp/8/4p3/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 1 3";
        let board = parse_fen(fen).unwrap();

        println!("\n=== Check position debug ===");
        println!("Board:\n{:?}", board);
        println!("Is in check: {}", board.is_in_check());

        let pseudo_legal = generate_moves(&board);
        println!("\nPseudo-legal moves: {}", pseudo_legal.len());

        // Check each pseudo-legal move
        for m in pseudo_legal.iter() {
            let is_legal = board.is_legal(*m);
            let status = if is_legal { "✓" } else { "✗" };
            println!("  {} {}", status, m);

            // For king moves, check destination square
            if m.to() == crate::square::Square::D2 {
                println!("    -> Checking Kd2 specifically");
                let mut test_board = board.clone();
                test_board.make_move(*m);
                let king_sq = test_board
                    .piece_bb(crate::piece::PieceType::King, Color::White)
                    .into_iter()
                    .next()
                    .unwrap();
                println!("    -> King after move: {}", king_sq);
                println!(
                    "    -> Is d2 attacked by Black? {}",
                    test_board.is_square_attacked(crate::square::Square::D2, Color::Black)
                );
            }
        }

        let results = perft_divide(&board, 1);
        println!("\nLegal moves: {}", results.len());
    }
}
