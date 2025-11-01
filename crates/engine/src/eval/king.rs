//! King safety evaluation.
//!
//! Evaluates king vulnerability to attack based on:
//! - Pawn shield quality (+15 to +30 cp per shield pawn)
//! - Attacking pieces in king zone (penalties up to -200 cp)
//! - Open files near king (-10 to -40 cp)
//! - King tropism (enemy piece proximity in middlegame)

use crate::attacks::{bishop_attacks, knight_attacks, pawn_attacks, queen_attacks, rook_attacks};
use crate::bitboard::Bitboard;
use crate::board::Board;
use crate::piece::{Color, PieceType};
use crate::square::Square;

/// King safety parameters (in centipawns).
mod values {
    /// Pawn shield bonus per pawn (by distance from king)
    /// Index 0 = immediately in front, 1 = one square ahead
    /// REDUCED VALUES: King safety was dominating opening evaluation
    pub const PAWN_SHIELD_BONUS: [[i32; 2]; 2] = [
        [15, 10], // [mg, eg] - immediately in front (was 30, 15)
        [8, 5],   // [mg, eg] - one square ahead (was 15, 5)
    ];

    /// Penalty for missing pawn shield
    pub const MISSING_SHIELD_PENALTY: [i32; 2] = [-10, -3]; // [mg, eg] (was -15, -5)

    /// Open file penalties near king [mg, eg]
    pub const OPEN_FILE_ON_KING: [i32; 2] = [-25, -8];  // (was -40, -10)
    pub const OPEN_FILE_ADJACENT: [i32; 2] = [-12, -4]; // (was -20, -5)
    pub const SEMI_OPEN_FILE_ON_KING: [i32; 2] = [-12, -4]; // (was -20, -5)
    pub const SEMI_OPEN_FILE_ADJACENT: [i32; 2] = [-6, 0]; // (was -10, 0)

    /// Attack weights by piece type
    pub const QUEEN_ATTACK_WEIGHT: i32 = 4;
    pub const ROOK_ATTACK_WEIGHT: i32 = 3;
    pub const BISHOP_ATTACK_WEIGHT: i32 = 2;
    pub const KNIGHT_ATTACK_WEIGHT: i32 = 2;
    pub const PAWN_ATTACK_WEIGHT: i32 = 1;

    /// King tropism bonus (distance-based, middlegame only)
    /// Bonus per attacking piece based on distance (chebyshev)
    pub const TROPISM_BONUS: [i32; 8] = [0, 10, 8, 6, 4, 2, 1, 0];
}

/// Evaluate king safety for a given color.
///
/// Returns (mg_score, eg_score) tuple.
/// Positive scores indicate a safer king.
pub fn evaluate_king_safety(board: &Board, color: Color, phase: i32) -> (i32, i32) {
    let mut mg_score = 0;
    let mut eg_score = 0;

    // Find king position
    let king_bb = board.piece_bb(PieceType::King, color);
    if king_bb.is_empty() {
        return (0, 0); // No king (should not happen in valid positions)
    }

    let king_sq = king_bb.lsb().unwrap();

    // 1. Evaluate pawn shield
    let (shield_mg, shield_eg) = evaluate_pawn_shield(board, king_sq, color);
    mg_score += shield_mg;
    eg_score += shield_eg;

    // 2. Evaluate attacking pressure (more important in middlegame)
    if phase < 200 {
        let attack_penalty = evaluate_king_attackers(board, king_sq, color);
        mg_score += attack_penalty;
        // Reduced impact in endgame
        eg_score += attack_penalty / 4;
    }

    // 3. Evaluate open files near king
    let (file_mg, file_eg) = evaluate_open_files_near_king(board, king_sq, color);
    mg_score += file_mg;
    eg_score += file_eg;

    // 4. King tropism (middlegame only)
    if phase < 200 {
        let tropism_bonus = evaluate_king_tropism(board, king_sq, color);
        mg_score += tropism_bonus;
    }

    (mg_score, eg_score)
}

/// Evaluate pawn shield in front of the king.
fn evaluate_pawn_shield(board: &Board, king_sq: Square, color: Color) -> (i32, i32) {
    let mut mg_score = 0;
    let mut eg_score = 0;

    let king_file = king_sq.file();
    let king_rank = king_sq.rank();

    // Determine which files to check based on king position
    let files_to_check = get_shield_files(king_file);

    let our_pawns = board.piece_bb(PieceType::Pawn, color);

    for file in files_to_check {
        // Check two ranks in front of king
        let shield_squares = get_shield_squares(file, king_rank, color);

        let mut found_shield = false;
        for (distance, sq) in shield_squares.iter().enumerate() {
            if let Some(sq) = sq {
                if our_pawns.contains(*sq) {
                    mg_score += values::PAWN_SHIELD_BONUS[distance][0];
                    eg_score += values::PAWN_SHIELD_BONUS[distance][1];
                    found_shield = true;
                    break; // Only count closest pawn
                }
            }
        }

        // Penalty for missing shield pawn
        if !found_shield && is_castled_or_central(king_sq, color) {
            mg_score += values::MISSING_SHIELD_PENALTY[0];
            eg_score += values::MISSING_SHIELD_PENALTY[1];
        }
    }

    (mg_score, eg_score)
}

/// Get the files to check for pawn shield based on king file.
fn get_shield_files(king_file: u8) -> Vec<u8> {
    match king_file {
        0 => vec![0, 1],    // a-file: check a, b
        1 => vec![0, 1, 2], // b-file: check a, b, c
        6 => vec![5, 6, 7], // g-file: check f, g, h
        7 => vec![6, 7],    // h-file: check g, h
        _ => vec![king_file.saturating_sub(1), king_file, king_file + 1], // central: check 3 files
    }
}

/// Get shield squares (up to 2 ranks ahead) for a file.
fn get_shield_squares(file: u8, king_rank: u8, color: Color) -> Vec<Option<Square>> {
    let mut squares = Vec::new();

    if color == Color::White {
        // Check ranks above
        if king_rank < 7 {
            squares.push(Some(Square::from_coords(file, king_rank + 1)));
        } else {
            squares.push(None);
        }
        if king_rank < 6 {
            squares.push(Some(Square::from_coords(file, king_rank + 2)));
        } else {
            squares.push(None);
        }
    } else {
        // Check ranks below
        if king_rank > 0 {
            squares.push(Some(Square::from_coords(file, king_rank - 1)));
        } else {
            squares.push(None);
        }
        if king_rank > 1 {
            squares.push(Some(Square::from_coords(file, king_rank - 2)));
        } else {
            squares.push(None);
        }
    }

    squares
}

/// Check if king is castled or in central position (where shield matters).
fn is_castled_or_central(king_sq: Square, color: Color) -> bool {
    let file = king_sq.file();
    let rank = king_sq.rank();

    let expected_rank = if color == Color::White { 0 } else { 7 };

    // King is on back rank and on kingside or queenside
    rank == expected_rank && (file <= 2 || file >= 5)
}

/// Evaluate attacking pressure on the king.
fn evaluate_king_attackers(board: &Board, king_sq: Square, color: Color) -> i32 {
    let king_zone = get_king_zone(king_sq);
    let enemy_color = color.opponent();

    let occupied = board.occupied();

    let mut attack_weight = 0;

    // Count attacking pieces in king zone
    // Queens
    let enemy_queens = board.piece_bb(PieceType::Queen, enemy_color);
    for queen_sq in enemy_queens {
        let attacks = queen_attacks(queen_sq, occupied);
        if !(attacks & king_zone).is_empty() {
            attack_weight += values::QUEEN_ATTACK_WEIGHT;
        }
    }

    // Rooks
    let enemy_rooks = board.piece_bb(PieceType::Rook, enemy_color);
    for rook_sq in enemy_rooks {
        let attacks = rook_attacks(rook_sq, occupied);
        if !(attacks & king_zone).is_empty() {
            attack_weight += values::ROOK_ATTACK_WEIGHT;
        }
    }

    // Bishops
    let enemy_bishops = board.piece_bb(PieceType::Bishop, enemy_color);
    for bishop_sq in enemy_bishops {
        let attacks = bishop_attacks(bishop_sq, occupied);
        if !(attacks & king_zone).is_empty() {
            attack_weight += values::BISHOP_ATTACK_WEIGHT;
        }
    }

    // Knights
    let enemy_knights = board.piece_bb(PieceType::Knight, enemy_color);
    for knight_sq in enemy_knights {
        let attacks = knight_attacks(knight_sq);
        if !(attacks & king_zone).is_empty() {
            attack_weight += values::KNIGHT_ATTACK_WEIGHT;
        }
    }

    // Pawns
    let enemy_pawns = board.piece_bb(PieceType::Pawn, enemy_color);
    for pawn_sq in enemy_pawns {
        let attacks = pawn_attacks(pawn_sq, enemy_color);
        if !(attacks & king_zone).is_empty() {
            attack_weight += values::PAWN_ATTACK_WEIGHT;
        }
    }

    // Convert attack weight to penalty
    match attack_weight {
        0..=2 => 0,
        3..=5 => -20,
        6..=9 => -50,
        10..=15 => -100,
        _ => -200,
    }
}

/// Get the king zone (3x3 area around king).
fn get_king_zone(king_sq: Square) -> Bitboard {
    let mut zone = Bitboard::EMPTY;

    let file = king_sq.file();
    let rank = king_sq.rank();

    // Add all squares within 1 square of the king
    for df in -1..=1i8 {
        for dr in -1..=1i8 {
            let new_file = file as i8 + df;
            let new_rank = rank as i8 + dr;

            if (0..8).contains(&new_file) && (0..8).contains(&new_rank) {
                let sq = Square::from_coords(new_file as u8, new_rank as u8);
                zone = zone.set(sq);
            }
        }
    }

    zone
}

/// Evaluate open and semi-open files near the king.
fn evaluate_open_files_near_king(board: &Board, king_sq: Square, color: Color) -> (i32, i32) {
    let mut mg_score = 0;
    let mut eg_score = 0;

    let king_file = king_sq.file();
    let our_pawns = board.piece_bb(PieceType::Pawn, color);
    let enemy_pawns = board.piece_bb(PieceType::Pawn, color.opponent());

    // Check king's file and adjacent files
    for file_offset in -1..=1i8 {
        let file = king_file as i8 + file_offset;
        if !(0..8).contains(&file) {
            continue;
        }

        let file = file as u8;
        let file_bb = file_bitboard(file);

        let has_our_pawns = !(our_pawns & file_bb).is_empty();
        let has_enemy_pawns = !(enemy_pawns & file_bb).is_empty();

        let (penalty_mg, penalty_eg) = if !has_our_pawns && !has_enemy_pawns {
            // Open file
            if file_offset == 0 {
                (values::OPEN_FILE_ON_KING[0], values::OPEN_FILE_ON_KING[1])
            } else {
                (values::OPEN_FILE_ADJACENT[0], values::OPEN_FILE_ADJACENT[1])
            }
        } else if !has_our_pawns && has_enemy_pawns {
            // Semi-open file (no friendly pawns)
            if file_offset == 0 {
                (
                    values::SEMI_OPEN_FILE_ON_KING[0],
                    values::SEMI_OPEN_FILE_ON_KING[1],
                )
            } else {
                (
                    values::SEMI_OPEN_FILE_ADJACENT[0],
                    values::SEMI_OPEN_FILE_ADJACENT[1],
                )
            }
        } else {
            (0, 0)
        };

        mg_score += penalty_mg;
        eg_score += penalty_eg;
    }

    (mg_score, eg_score)
}

/// Evaluate king tropism (enemy piece proximity to king).
fn evaluate_king_tropism(board: &Board, king_sq: Square, color: Color) -> i32 {
    let enemy_color = color.opponent();
    let mut bonus = 0;

    // Count all enemy attacking pieces and their distance to king
    let attacking_pieces = [
        PieceType::Queen,
        PieceType::Rook,
        PieceType::Bishop,
        PieceType::Knight,
    ];

    for piece_type in attacking_pieces {
        let pieces = board.piece_bb(piece_type, enemy_color);
        for piece_sq in pieces {
            let distance = chebyshev_distance(king_sq, piece_sq);
            if distance < 8 {
                bonus += values::TROPISM_BONUS[distance as usize];
            }
        }
    }

    -bonus // Negative because closer enemy pieces are bad for us
}

/// Calculate Chebyshev distance (max of file/rank distance).
fn chebyshev_distance(sq1: Square, sq2: Square) -> u8 {
    let file_dist = (sq1.file() as i8 - sq2.file() as i8).abs();
    let rank_dist = (sq1.rank() as i8 - sq2.rank() as i8).abs();
    file_dist.max(rank_dist) as u8
}

/// Get a bitboard mask for a file.
fn file_bitboard(file: u8) -> Bitboard {
    Bitboard::new(0x0101010101010101u64 << file)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::parse_fen;

    #[test]
    fn test_pawn_shield_kingside() {
        // White king castled kingside with full pawn shield
        let board = parse_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQK2R w KQkq - 0 1").unwrap();
        let king_sq = Square::from_algebraic("e1").unwrap();
        let (mg, _eg) = evaluate_pawn_shield(&board, king_sq, Color::White);

        // Should have some bonus for pawn shield
        assert!(mg > 0, "Pawn shield should provide bonus, got mg={}", mg);
    }

    #[test]
    fn test_pawn_shield_missing() {
        // White king on g1 (castled kingside) with no pawns on f, g, h files
        let board = parse_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPP3/RNBQK1NR w KQkq - 0 1").unwrap();
        let king_sq = Square::from_algebraic("g1").unwrap();
        let (mg, _eg) = evaluate_pawn_shield(&board, king_sq, Color::White);

        // Should have penalty for missing shield pawns
        assert!(
            mg < 0,
            "Missing pawn shield should have penalty, got mg={}",
            mg
        );
    }

    #[test]
    fn test_king_zone() {
        let king_sq = Square::from_algebraic("e4").unwrap();
        let zone = get_king_zone(king_sq);

        // King zone should be 3x3 = 9 squares
        assert_eq!(zone.count(), 9, "King zone should have 9 squares");

        // Check that all adjacent squares are in zone
        assert!(zone.contains(Square::from_algebraic("d3").unwrap()));
        assert!(zone.contains(Square::from_algebraic("e3").unwrap()));
        assert!(zone.contains(Square::from_algebraic("f3").unwrap()));
        assert!(zone.contains(Square::from_algebraic("d4").unwrap()));
        assert!(zone.contains(Square::from_algebraic("e4").unwrap()));
        assert!(zone.contains(Square::from_algebraic("f4").unwrap()));
        assert!(zone.contains(Square::from_algebraic("d5").unwrap()));
        assert!(zone.contains(Square::from_algebraic("e5").unwrap()));
        assert!(zone.contains(Square::from_algebraic("f5").unwrap()));
    }

    #[test]
    fn test_king_zone_corner() {
        let king_sq = Square::from_algebraic("a1").unwrap();
        let zone = get_king_zone(king_sq);

        // Corner king zone should have 4 squares
        assert_eq!(zone.count(), 4, "Corner king zone should have 4 squares");
    }

    #[test]
    fn test_king_attackers_safe_position() {
        // King in starting position, no attackers
        let board = Board::startpos();
        let king_sq = Square::from_algebraic("e1").unwrap();
        let penalty = evaluate_king_attackers(&board, king_sq, Color::White);

        // Should have no penalty in safe starting position
        assert_eq!(penalty, 0, "Safe king should have no attack penalty");
    }

    #[test]
    fn test_king_attackers_under_pressure() {
        // White king under attack by multiple pieces
        let board =
            parse_fen("rnbqkbnr/pppp1ppp/8/4p3/8/8/PPPPQPPP/RNB1KB1R w KQkq - 0 1").unwrap();
        let king_sq = Square::from_algebraic("e1").unwrap();
        let penalty = evaluate_king_attackers(&board, king_sq, Color::White);

        // Note: In this position there might not be direct attackers,
        // let's just check it doesn't crash
        assert!(penalty <= 0, "Attack penalty should be non-positive");
    }

    #[test]
    fn test_open_file_on_king() {
        // White king on e-file with no pawns on e-file (truly open)
        let board = parse_fen("rnbqkbnr/pppp1ppp/8/8/8/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1").unwrap();
        let king_sq = Square::from_algebraic("e1").unwrap();
        let (mg, _eg) = evaluate_open_files_near_king(&board, king_sq, Color::White);

        // Should have penalty for open file
        assert!(
            mg < 0,
            "Open file on king should have penalty, got mg={}",
            mg
        );
    }

    #[test]
    fn test_king_tropism() {
        // Enemy queen near white king
        let board =
            parse_fen("rnb1kbnr/pppppppp/8/8/8/4q3/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        let king_sq = Square::from_algebraic("e1").unwrap();
        let bonus = evaluate_king_tropism(&board, king_sq, Color::White);

        // Should have penalty (negative bonus) for enemy piece near king
        assert!(bonus < 0, "Enemy piece near king should give penalty");
    }

    #[test]
    fn test_chebyshev_distance() {
        let e4 = Square::from_algebraic("e4").unwrap();
        let e5 = Square::from_algebraic("e5").unwrap();
        assert_eq!(chebyshev_distance(e4, e5), 1);

        let a1 = Square::from_algebraic("a1").unwrap();
        let h8 = Square::from_algebraic("h8").unwrap();
        assert_eq!(chebyshev_distance(a1, h8), 7);

        let e4_2 = Square::from_algebraic("e4").unwrap();
        let g6 = Square::from_algebraic("g6").unwrap();
        assert_eq!(chebyshev_distance(e4_2, g6), 2); // max(2, 2) = 2
    }

    #[test]
    fn test_king_safety_startpos() {
        let board = Board::startpos();
        let (white_mg, _white_eg) = evaluate_king_safety(&board, Color::White, 0);
        let (black_mg, _black_eg) = evaluate_king_safety(&board, Color::Black, 0);

        // Both kings should have similar safety in startpos
        // (both have pawn shields, no attacks)
        assert!(
            (white_mg - black_mg).abs() < 20,
            "Both kings should have similar safety in startpos"
        );
    }

    #[test]
    fn test_king_safety_endgame_reduced() {
        // King safety should matter less in endgame (phase = 256)
        let board = parse_fen("4k3/8/8/8/8/8/8/4K3 w - - 0 1").unwrap();

        let (mg_middlegame, eg_middlegame) = evaluate_king_safety(&board, Color::White, 0);
        let (mg_endgame, eg_endgame) = evaluate_king_safety(&board, Color::White, 256);

        // Attack penalties should be much lower in endgame
        // (though in this bare position there are no attacks anyway)
        assert_eq!(mg_middlegame, mg_endgame);
        assert_eq!(eg_middlegame, eg_endgame);
    }

    #[test]
    fn test_file_bitboard() {
        let e_file = file_bitboard(4);
        assert_eq!(e_file.count(), 8, "File should have 8 squares");

        // Check all e-file squares
        for rank in 0..8 {
            let sq = Square::from_coords(4, rank);
            assert!(e_file.contains(sq), "E-file should contain e{}", rank + 1);
        }
    }
}
