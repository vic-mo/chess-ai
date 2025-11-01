/// Debug piece counting
use engine::board::Board;
use engine::io::{parse_fen, ToFen};
use engine::piece::{Color, PieceType};

fn main() {
    // Position where White is up a bishop
    let fen = "rnbqkb1r/pppppppp/5n2/8/8/5N2/PPPPPPPP/RNBQKB1R w KQkq - 0 1";
    let board = parse_fen(fen).unwrap();

    println!("Input FEN: {}", fen);
    println!("Parsed back to FEN: {}", board.to_fen());
    println!();

    println!("Piece count for White:");
    for pt in PieceType::all() {
        let bb = board.piece_bb(pt, Color::White);
        let count = bb.count();
        println!("  {:?}: {}", pt, count);
    }
    println!();

    println!("Piece count for Black:");
    for pt in PieceType::all() {
        let bb = board.piece_bb(pt, Color::Black);
        let count = bb.count();
        println!("  {:?}: {}", pt, count);
    }
    println!();

    println!("Expected:");
    println!("  White: 8 pawns, 2 knights, 2 bishops, 1 rook, 1 queen, 1 king");
    println!("  Black: 8 pawns, 2 knights, 1 bishop, 2 rooks, 1 queen, 1 king");
}
