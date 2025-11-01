/// Debug evaluation breakdown
use engine::board::Board;
use engine::eval::{Evaluator, evaluate_material};
use engine::io::parse_fen;
use engine::piece::Color;

fn main() {
    let mut evaluator = Evaluator::new();

    // Position where White is up a bishop
    let fen = "rnbqkb1r/pppppppp/5n2/8/8/5N2/PPPPPPPP/RNBQKB1R w KQkq - 0 1";
    let board = parse_fen(fen).unwrap();

    println!("Position: White up a bishop");
    println!("FEN: {}", fen);
    println!();

    // Get total evaluation
    let total_eval = evaluator.evaluate(&board);
    println!("Total evaluation: {} cp ({:.2} pawns)", total_eval, total_eval as f64 / 100.0);
    println!("  Expected: ~+330 cp (+3.3 pawns)");
    println!();

    // Break down material
    let white_material = evaluate_material(&board, Color::White);
    let black_material = evaluate_material(&board, Color::Black);
    let material_diff = white_material - black_material;

    println!("Material breakdown:");
    println!("  White material: {} cp", white_material);
    println!("  Black material: {} cp", black_material);
    println!("  Difference: {} cp ({:.2} pawns)", material_diff, material_diff as f64 / 100.0);
    println!();

    println!("This means non-material evaluation is contributing: {} cp",
        total_eval - material_diff);
    println!("  (PST + pawns + king safety + mobility + pieces)");
}
