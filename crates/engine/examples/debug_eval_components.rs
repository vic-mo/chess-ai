/// Detailed evaluation component breakdown
use engine::board::Board;
use engine::eval::{
    Evaluator,
    evaluate_material,
    evaluate_positional,
    evaluate_pawns_cached,
    evaluate_king_safety,
    evaluate_piece_activity,
    PawnHashTable,
    PieceSquareTables,
};
use engine::eval::phase::calculate_phase;
use engine::io::parse_fen;
use engine::piece::Color;

fn print_eval_breakdown(fen: &str, description: &str) {
    println!("\n======================================================================");
    println!("{}", description);
    println!("======================================================================");
    println!("FEN: {}", fen);
    println!();

    let board = parse_fen(fen).expect("Valid FEN");
    let mut evaluator = Evaluator::new();
    let mut pawn_hash = PawnHashTable::default();
    let pst = PieceSquareTables::default();

    // Get game phase
    let phase = calculate_phase(&board);
    println!("Game phase: {} (0=opening, 128=middlegame, 256=endgame)", phase);
    println!();

    // Material
    let white_material = evaluate_material(&board, Color::White);
    let black_material = evaluate_material(&board, Color::Black);
    let material_diff = white_material - black_material;
    println!("MATERIAL:");
    println!("  White: {} cp", white_material);
    println!("  Black: {} cp", black_material);
    println!("  Diff:  {:+} cp ({:+.2} pawns)", material_diff, material_diff as f64 / 100.0);
    println!();

    // PST
    let white_pst = pst.evaluate_position(&board, Color::White);
    let black_pst = pst.evaluate_position(&board, Color::Black);
    let pst_diff = white_pst - black_pst;
    println!("PIECE-SQUARE TABLES:");
    println!("  White: {} cp", white_pst);
    println!("  Black: {} cp", black_pst);
    println!("  Diff:  {:+} cp ({:+.2} pawns)", pst_diff, pst_diff as f64 / 100.0);
    println!();

    // Pawns
    let (white_pawn_mg, white_pawn_eg, black_pawn_mg, black_pawn_eg) =
        evaluate_pawns_cached(&board, &mut pawn_hash);
    let pawn_mg_diff = white_pawn_mg - black_pawn_mg;
    let pawn_eg_diff = white_pawn_eg - black_pawn_eg;
    println!("PAWN STRUCTURE:");
    println!("  White MG: {} cp, EG: {} cp", white_pawn_mg, white_pawn_eg);
    println!("  Black MG: {} cp, EG: {} cp", black_pawn_mg, black_pawn_eg);
    println!("  MG Diff:  {:+} cp", pawn_mg_diff);
    println!("  EG Diff:  {:+} cp", pawn_eg_diff);
    println!();

    // King safety
    let (white_king_mg, white_king_eg) = evaluate_king_safety(&board, Color::White, phase);
    let (black_king_mg, black_king_eg) = evaluate_king_safety(&board, Color::Black, phase);
    let king_mg_diff = white_king_mg - black_king_mg;
    let king_eg_diff = white_king_eg - black_king_eg;
    println!("KING SAFETY:");
    println!("  White MG: {} cp, EG: {} cp", white_king_mg, white_king_eg);
    println!("  Black MG: {} cp, EG: {} cp", black_king_mg, black_king_eg);
    println!("  MG Diff:  {:+} cp", king_mg_diff);
    println!("  EG Diff:  {:+} cp", king_eg_diff);
    println!();

    // Piece activity
    let (white_pieces_mg, white_pieces_eg) = evaluate_piece_activity(&board, Color::White, phase);
    let (black_pieces_mg, black_pieces_eg) = evaluate_piece_activity(&board, Color::Black, phase);
    let pieces_mg_diff = white_pieces_mg - black_pieces_mg;
    let pieces_eg_diff = white_pieces_eg - black_pieces_eg;
    println!("PIECE ACTIVITY:");
    println!("  White MG: {} cp, EG: {} cp", white_pieces_mg, white_pieces_eg);
    println!("  Black MG: {} cp, EG: {} cp", black_pieces_mg, black_pieces_eg);
    println!("  MG Diff:  {:+} cp", pieces_mg_diff);
    println!("  EG Diff:  {:+} cp", pieces_eg_diff);
    println!();

    // Mobility/positional
    let white_mobility = evaluate_positional(&board, Color::White);
    let black_mobility = evaluate_positional(&board, Color::Black);
    let mobility_diff = white_mobility - black_mobility;
    println!("MOBILITY/POSITIONAL:");
    println!("  White: {} cp", white_mobility);
    println!("  Black: {} cp", black_mobility);
    println!("  Diff:  {:+} cp ({:+.2} pawns)", mobility_diff, mobility_diff as f64 / 100.0);
    println!();

    // Total
    let total_eval = evaluator.evaluate(&board);
    println!("TOTAL EVALUATION:");
    println!("  Score: {:+} cp ({:+.2} pawns)", total_eval, total_eval as f64 / 100.0);
    println!("  From {}'s perspective", if board.side_to_move() == Color::White { "White" } else { "Black" });
    println!();

    // Calculate what total SHOULD be based on components
    let mg_score = material_diff + pst_diff + pawn_mg_diff + king_mg_diff + pieces_mg_diff + mobility_diff;
    let eg_score = material_diff + pst_diff + pawn_eg_diff + king_eg_diff + pieces_eg_diff + mobility_diff;

    // Interpolate
    use engine::eval::phase::interpolate;
    let calculated_score = interpolate(mg_score, eg_score, phase);
    let calculated_from_stm = if board.side_to_move() == Color::Black {
        -calculated_score
    } else {
        calculated_score
    };

    println!("VERIFICATION:");
    println!("  Calculated (MG): {:+} cp", mg_score);
    println!("  Calculated (EG): {:+} cp", eg_score);
    println!("  Interpolated:    {:+} cp", calculated_score);
    println!("  From STM:        {:+} cp", calculated_from_stm);
    println!("  Actual total:    {:+} cp", total_eval);
    println!("  Match: {}", if calculated_from_stm == total_eval { "✓" } else { "✗" });
    println!();

    // Analysis
    println!("ANALYSIS:");
    let components = vec![
        ("Material", material_diff),
        ("PST", pst_diff),
        ("Pawns (MG)", pawn_mg_diff),
        ("King Safety (MG)", king_mg_diff),
        ("Piece Activity (MG)", pieces_mg_diff),
        ("Mobility", mobility_diff),
    ];

    let mut sorted = components.clone();
    sorted.sort_by_key(|(_, score)| -score.abs());

    println!("  Components by impact:");
    for (name, score) in sorted.iter() {
        if score.abs() > 5 {
            println!("    {:20} {:+6} cp", name, score);
        }
    }
    println!();

    // Warnings
    if mobility_diff.abs() > 100 {
        println!("  ⚠️  WARNING: Mobility difference is very large (> 1 pawn)");
    }
    if pieces_mg_diff.abs() > 100 {
        println!("  ⚠️  WARNING: Piece activity difference is very large (> 1 pawn)");
    }
    if pst_diff.abs() > 100 {
        println!("  ⚠️  WARNING: PST difference is very large (> 1 pawn)");
    }
}

fn main() {
    println!("\n");
    println!("╔════════════════════════════════════════════════════════════════════╗");
    println!("║           EVALUATION COMPONENT BREAKDOWN ANALYSIS                 ║");
    println!("╚════════════════════════════════════════════════════════════════════╝");

    // Test 1: Starting position
    print_eval_breakdown(
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        "TEST 1: Starting Position (should be ~0 cp)"
    );

    // Test 2: After 1.e4
    print_eval_breakdown(
        "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1",
        "TEST 2: After 1.e4 (should be +10 to +50 cp)"
    );

    // Test 3: After 1.Nf3 d5
    print_eval_breakdown(
        "rnbqkbnr/ppp1pppp/8/3p4/8/5N2/PPPPPPPP/RNBQKB1R w KQkq d6 0 2",
        "TEST 3: After 1.Nf3 d5 (should be 0 to +50 cp) - ENGINE CHOOSES Ng5 HERE"
    );

    // Test 4: After 1.Nf3 d5 2.Ng5 (the bad move)
    print_eval_breakdown(
        "rnbqkbnr/ppp1pppp/8/3p2N1/8/8/PPPPPPPP/RNBQKB1R b KQkq - 1 2",
        "TEST 4: After 1.Nf3 d5 2.Ng5 (should be -30 to +30 cp)"
    );

    // Test 5: After 1.e4 Nf6 (normal position)
    print_eval_breakdown(
        "rnbqkb1r/pppppppp/5n2/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 1 2",
        "TEST 5: After 1.e4 Nf6 (should be +20 to +60 cp)"
    );

    println!("\n╔════════════════════════════════════════════════════════════════════╗");
    println!("║                         ANALYSIS COMPLETE                          ║");
    println!("╚════════════════════════════════════════════════════════════════════╝\n");
}
