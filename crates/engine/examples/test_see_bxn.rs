/// Test SEE for BxN - with debug output
use engine::io::{parse_fen, ToFen};
use engine::movegen::generate_moves;
use engine::piece::PieceType;

fn main() {
    // BxN with no recapture: +320
    let board = parse_fen("rnbqkb1r/pppppppp/5n2/8/8/2B5/PPPPPPPP/RNBQK1NR w KQkq - 0 1").unwrap();
    println!("Position: {}", board.to_fen());
    println!();

    // Check what pieces are on the squares
    let c3 = engine::square::Square::from_algebraic("c3").unwrap();
    let f6 = engine::square::Square::from_algebraic("f6").unwrap();

    println!("Piece on c3: {:?}", board.piece_at(c3));
    println!("Piece on f6: {:?}", board.piece_at(f6));
    println!();

    let moves = generate_moves(&board);
    let capture = moves.iter()
        .find(|m| m.from().to_string() == "c3" && m.to().to_string() == "f6")
        .unwrap();

    println!("Move: {} (is_capture: {})", capture, capture.is_capture());
    println!();

    // Manually build gains list to debug
    println!("=== Building gains list manually ===");

    let attacker = board.piece_at(c3).unwrap();
    let victim = board.piece_at(f6).unwrap();

    println!("Attacker: {:?} (value: {})", attacker.piece_type, piece_val(attacker.piece_type));
    println!("Victim: {:?} (value: {})", victim.piece_type, piece_val(victim.piece_type));
    println!();

    println!("Gains[0] = {} (captured piece)", piece_val(victim.piece_type));

    // Check for defenders
    use engine::attacks::*;
    use engine::bitboard::Bitboard;
    use engine::piece::Color;

    let occupied = board.occupied().clear(c3);
    println!("Occupied after removing attacker: {:?}", occupied.count());

    // Check black pieces that can attack f6
    let black_pawns = board.piece_bb(PieceType::Pawn, Color::Black) & pawn_attacks(f6, Color::White) & occupied;
    let black_knights = board.piece_bb(PieceType::Knight, Color::Black) & knight_attacks(f6) & occupied;
    let black_bishops = board.piece_bb(PieceType::Bishop, Color::Black) & bishop_attacks(f6, occupied) & occupied;
    let black_rooks = board.piece_bb(PieceType::Rook, Color::Black) & rook_attacks(f6, occupied) & occupied;
    let black_queens = board.piece_bb(PieceType::Queen, Color::Black) & (bishop_attacks(f6, occupied) | rook_attacks(f6, occupied)) & occupied;
    let black_king = board.piece_bb(PieceType::King, Color::Black) & king_attacks(f6) & occupied;

    println!("Black attackers of f6:");
    println!("  Pawns: {}", black_pawns.count());
    println!("  Knights: {}", black_knights.count());
    println!("  Bishops: {}", black_bishops.count());
    println!("  Rooks: {}", black_rooks.count());
    println!("  Queens: {}", black_queens.count());
    println!("  King: {}", black_king.count());

    let value = engine::search::see::see_value(&board, *capture);
    println!("\nSEE value: {} cp", value);
    println!();

    if value > 300 {
        println!("✓ Correct: BxN wins knight");
    } else {
        println!("✗ Wrong: BxN should win knight, got {} cp", value);
    }
}

fn piece_val(piece: PieceType) -> i32 {
    match piece {
        PieceType::Pawn => 100,
        PieceType::Knight => 320,
        PieceType::Bishop => 330,
        PieceType::Rook => 500,
        PieceType::Queen => 900,
        PieceType::King => 20000,
    }
}
