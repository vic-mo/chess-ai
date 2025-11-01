/// Test basic evaluation
use engine::board::Board;
use engine::eval::Evaluator;
use engine::io::parse_fen;

fn main() {
    let mut evaluator = Evaluator::new();

    // Starting position
    let board = Board::default();
    let eval_start = evaluator.evaluate(&board);
    println!("Starting position eval: {} cp ({:.2} pawns)", eval_start, eval_start as f64 / 100.0);
    println!("  (Should be near 0, slightly positive for White)");
    println!();

    // After 1.e4
    let fen_e4 = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
    let board_e4 = parse_fen(fen_e4).unwrap();
    let eval_e4 = evaluator.evaluate(&board_e4);
    println!("After 1.e4: {} cp ({:.2} pawns)", eval_e4, eval_e4 as f64 / 100.0);
    println!("  (Should be small positive for White, from Black's view should be negative)");
    println!();

    // After 1.e4 e5
    let fen_e4e5 = "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq e6 0 2";
    let board_e4e5 = parse_fen(fen_e4e5).unwrap();
    let eval_e4e5 = evaluator.evaluate(&board_e4e5);
    println!("After 1.e4 e5: {} cp ({:.2} pawns)", eval_e4e5, eval_e4e5 as f64 / 100.0);
    println!("  (Should be near 0, roughly equal)");
    println!();

    // Position where White is up a piece
    let fen_up_piece = "rnbqkb1r/pppppppp/5n2/8/8/5N2/PPPPPPPP/RNBQKB1R w KQkq - 0 1";
    let board_up_piece = parse_fen(fen_up_piece).unwrap();
    let eval_up_piece = evaluator.evaluate(&board_up_piece);
    println!("White up a bishop: {} cp ({:.2} pawns)", eval_up_piece, eval_up_piece as f64 / 100.0);
    println!("  (Should be around +330 cp = +3.3 pawns)");
    println!();

    // Position where Black is up a piece
    let fen_down_piece = "rnbqkb1r/pppppppp/5n2/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let board_down_piece = parse_fen(fen_down_piece).unwrap();
    let eval_down_piece = evaluator.evaluate(&board_down_piece);
    println!("White down a knight: {} cp ({:.2} pawns)", eval_down_piece, eval_down_piece as f64 / 100.0);
    println!("  (Should be around -320 cp = -3.2 pawns)");
}
