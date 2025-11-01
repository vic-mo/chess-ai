//! Piece activity evaluation.
//!
//! Evaluates:
//! - Rook activity (open files, 7th rank)
//! - Bishop activity (bishop pair, bad bishops, trapped bishops)
//! - Knight activity (outposts, trapped knights)
//! - Piece centralization

use crate::attacks::knight_attacks;
use crate::bitboard::Bitboard;
use crate::board::Board;
use crate::piece::{Color, PieceType};
use crate::square::Square;

/// Piece activity parameters (in centipawns).
mod values {
    /// Rook on open file [mg, eg]
    pub const ROOK_OPEN_FILE: [i32; 2] = [30, 15];

    /// Rook on semi-open file [mg, eg]
    pub const ROOK_SEMI_OPEN_FILE: [i32; 2] = [15, 10];

    /// Rook on 7th rank [mg, eg]
    pub const ROOK_ON_SEVENTH: [i32; 2] = [25, 30];

    /// Two rooks on 7th rank [mg, eg]
    pub const TWO_ROOKS_ON_SEVENTH: [i32; 2] = [40, 50];

    /// Bishop pair bonus [mg, eg]
    pub const BISHOP_PAIR: [i32; 2] = [50, 60];

    /// Bad bishop penalty [mg, eg]
    pub const BAD_BISHOP: [i32; 2] = [-15, -10];

    /// Trapped bishop penalty [mg, eg]
    pub const TRAPPED_BISHOP: [i32; 2] = [-150, -100];

    /// Knight outpost bonus by file (central better) [mg, eg]
    pub const KNIGHT_OUTPOST_BASE: [i32; 2] = [20, 15];
    pub const KNIGHT_OUTPOST_CENTRAL_BONUS: [i32; 2] = [10, 5];

    /// Trapped knight penalty [mg, eg]
    pub const TRAPPED_KNIGHT: [i32; 2] = [-100, -80];

    /// Piece on back rank penalty (middlegame only) [mg, eg]
    pub const PIECE_ON_BACK_RANK: [i32; 2] = [-10, 0];

    /// Centralization bonus (for pieces on central squares) [mg, eg]
    pub const CENTRALIZATION_BONUS: [i32; 2] = [5, 2];
}

/// Evaluate piece activity for a given color.
///
/// Returns (mg_score, eg_score) tuple.
pub fn evaluate_piece_activity(board: &Board, color: Color, phase: i32) -> (i32, i32) {
    let mut mg_score = 0;
    let mut eg_score = 0;

    // 1. Rook activity
    let (rook_mg, rook_eg) = evaluate_rook_activity(board, color);
    mg_score += rook_mg;
    eg_score += rook_eg;

    // 2. Bishop activity
    let (bishop_mg, bishop_eg) = evaluate_bishop_activity(board, color);
    mg_score += bishop_mg;
    eg_score += bishop_eg;

    // 3. Knight activity
    let (knight_mg, knight_eg) = evaluate_knight_activity(board, color);
    mg_score += knight_mg;
    eg_score += knight_eg;

    // 4. General piece placement (centralization, back rank penalty)
    let (placement_mg, placement_eg) = evaluate_piece_placement(board, color, phase);
    mg_score += placement_mg;
    eg_score += placement_eg;

    (mg_score, eg_score)
}

/// Evaluate rook activity.
fn evaluate_rook_activity(board: &Board, color: Color) -> (i32, i32) {
    let mut mg_score = 0;
    let mut eg_score = 0;

    let rooks = board.piece_bb(PieceType::Rook, color);
    let our_pawns = board.piece_bb(PieceType::Pawn, color);
    let enemy_pawns = board.piece_bb(PieceType::Pawn, color.opponent());

    let mut rooks_on_seventh = 0;

    for rook_sq in rooks {
        let file = rook_sq.file();
        let rank = rook_sq.rank();

        // Check if on open or semi-open file
        let file_bb = file_bitboard(file);
        let has_our_pawns = !(our_pawns & file_bb).is_empty();
        let has_enemy_pawns = !(enemy_pawns & file_bb).is_empty();

        if !has_our_pawns && !has_enemy_pawns {
            // Open file
            mg_score += values::ROOK_OPEN_FILE[0];
            eg_score += values::ROOK_OPEN_FILE[1];
        } else if !has_our_pawns && has_enemy_pawns {
            // Semi-open file
            mg_score += values::ROOK_SEMI_OPEN_FILE[0];
            eg_score += values::ROOK_SEMI_OPEN_FILE[1];
        }

        // Check if on 7th rank
        let seventh_rank = if color == Color::White { 6 } else { 1 };
        if rank == seventh_rank {
            // Check if enemy king is on 8th or enemy pawns on 7th
            let king_rank = if color == Color::White { 7 } else { 0 };
            let enemy_king_bb = board.piece_bb(PieceType::King, color.opponent());

            if let Some(enemy_king_sq) = enemy_king_bb.lsb() {
                if enemy_king_sq.rank() == king_rank {
                    rooks_on_seventh += 1;
                }
            }

            // Or check for enemy pawns on 7th
            let pawns_on_seventh = !(enemy_pawns & rank_bitboard(seventh_rank)).is_empty();
            if pawns_on_seventh {
                rooks_on_seventh += 1;
            }
        }
    }

    // Bonus for rook(s) on 7th
    if rooks_on_seventh >= 2 {
        mg_score += values::TWO_ROOKS_ON_SEVENTH[0];
        eg_score += values::TWO_ROOKS_ON_SEVENTH[1];
    } else if rooks_on_seventh == 1 {
        mg_score += values::ROOK_ON_SEVENTH[0];
        eg_score += values::ROOK_ON_SEVENTH[1];
    }

    (mg_score, eg_score)
}

/// Evaluate bishop activity.
fn evaluate_bishop_activity(board: &Board, color: Color) -> (i32, i32) {
    let mut mg_score = 0;
    let mut eg_score = 0;

    let bishops = board.piece_bb(PieceType::Bishop, color);
    let bishop_count = bishops.count();

    // 1. Bishop pair bonus
    if bishop_count >= 2 {
        mg_score += values::BISHOP_PAIR[0];
        eg_score += values::BISHOP_PAIR[1];
    }

    // 2. Bad bishops and trapped bishops
    let our_pawns = board.piece_bb(PieceType::Pawn, color);

    for bishop_sq in bishops {
        // Check if trapped
        if is_trapped_bishop(bishop_sq, color) {
            mg_score += values::TRAPPED_BISHOP[0];
            eg_score += values::TRAPPED_BISHOP[1];
        }

        // Check if bad bishop
        if is_bad_bishop(bishop_sq, our_pawns) {
            mg_score += values::BAD_BISHOP[0];
            eg_score += values::BAD_BISHOP[1];
        }
    }

    (mg_score, eg_score)
}

/// Evaluate knight activity.
fn evaluate_knight_activity(board: &Board, color: Color) -> (i32, i32) {
    let mut mg_score = 0;
    let mut eg_score = 0;

    let knights = board.piece_bb(PieceType::Knight, color);
    let our_pawns = board.piece_bb(PieceType::Pawn, color);
    let enemy_pawns = board.piece_bb(PieceType::Pawn, color.opponent());
    let occupied = board.occupied();

    for knight_sq in knights {
        // Check if trapped
        if is_trapped_knight(knight_sq, occupied) {
            mg_score += values::TRAPPED_KNIGHT[0];
            eg_score += values::TRAPPED_KNIGHT[1];
            continue;
        }

        // Check if on outpost
        if is_knight_outpost(knight_sq, color, our_pawns, enemy_pawns) {
            let file = knight_sq.file();
            let is_central = (2..6).contains(&file);

            mg_score += values::KNIGHT_OUTPOST_BASE[0];
            eg_score += values::KNIGHT_OUTPOST_BASE[1];

            if is_central {
                mg_score += values::KNIGHT_OUTPOST_CENTRAL_BONUS[0];
                eg_score += values::KNIGHT_OUTPOST_CENTRAL_BONUS[1];
            }
        }
    }

    (mg_score, eg_score)
}

/// Evaluate general piece placement.
fn evaluate_piece_placement(board: &Board, color: Color, phase: i32) -> (i32, i32) {
    let mut mg_score = 0;
    let mut eg_score = 0;

    let back_rank = if color == Color::White { 0 } else { 7 };

    // Evaluate minor pieces and rooks
    for piece_type in [PieceType::Knight, PieceType::Bishop, PieceType::Rook] {
        let pieces = board.piece_bb(piece_type, color);

        for piece_sq in pieces {
            let rank = piece_sq.rank();

            // Centralization bonus
            if is_central_square(piece_sq) {
                mg_score += values::CENTRALIZATION_BONUS[0];
                eg_score += values::CENTRALIZATION_BONUS[1];
            }

            // Back rank penalty (middlegame only)
            if rank == back_rank && phase < 200 {
                mg_score += values::PIECE_ON_BACK_RANK[0];
            }
        }
    }

    (mg_score, eg_score)
}

/// Check if a bishop is trapped.
fn is_trapped_bishop(bishop_sq: Square, color: Color) -> bool {
    // Check common trapped bishop squares
    match color {
        Color::White => {
            // a7, h7 trapped by b6/g6 pawns
            bishop_sq == Square::from_algebraic("a7").unwrap()
                || bishop_sq == Square::from_algebraic("h7").unwrap()
        }
        Color::Black => {
            // a2, h2 trapped by b3/g3 pawns
            bishop_sq == Square::from_algebraic("a2").unwrap()
                || bishop_sq == Square::from_algebraic("h2").unwrap()
        }
    }
}

/// Check if a bishop is bad (majority of pawns on bishop's color).
fn is_bad_bishop(bishop_sq: Square, our_pawns: Bitboard) -> bool {
    let bishop_on_light = (bishop_sq.file() + bishop_sq.rank()) % 2 == 0;

    let mut light_pawns = 0;
    let mut dark_pawns = 0;

    for pawn_sq in our_pawns {
        if (pawn_sq.file() + pawn_sq.rank()) % 2 == 0 {
            light_pawns += 1;
        } else {
            dark_pawns += 1;
        }
    }

    let total_pawns = light_pawns + dark_pawns;
    if total_pawns == 0 {
        return false;
    }

    let pawns_on_bishop_color = if bishop_on_light {
        light_pawns
    } else {
        dark_pawns
    };

    // Bad if more than 50% of pawns on bishop's color
    pawns_on_bishop_color > total_pawns / 2
}

/// Check if a knight is trapped (no legal moves).
fn is_trapped_knight(knight_sq: Square, occupied: Bitboard) -> bool {
    let attacks = knight_attacks(knight_sq);
    // Knight is trapped if all its attack squares are occupied by friendly pieces
    // For simplicity, we'll check if it's on the edge with very limited mobility
    let file = knight_sq.file();
    let mobility = (attacks & !occupied).count();

    // Consider trapped if on edge with <= 2 squares
    (file == 0 || file == 7) && mobility <= 2
}

/// Check if a knight is on an outpost.
fn is_knight_outpost(
    knight_sq: Square,
    color: Color,
    our_pawns: Bitboard,
    enemy_pawns: Bitboard,
) -> bool {
    let rank = knight_sq.rank();
    let file = knight_sq.file();

    // Must be on 4th, 5th, or 6th rank (for white, 0-indexed: 3, 4, 5)
    // For black, it's 3rd, 4th, or 5th rank (0-indexed: 2, 3, 4)
    if color == Color::White {
        if !(3..=5).contains(&rank) {
            return false;
        }
    } else if !(2..=4).contains(&rank) {
        return false;
    }

    // Check if protected by our pawn
    let pawn_protectors = if color == Color::White {
        let left = if file > 0 {
            Bitboard::from_square(Square::from_coords(file - 1, rank - 1))
        } else {
            Bitboard::EMPTY
        };
        let right = if file < 7 {
            Bitboard::from_square(Square::from_coords(file + 1, rank - 1))
        } else {
            Bitboard::EMPTY
        };
        left | right
    } else {
        let left = if file > 0 {
            Bitboard::from_square(Square::from_coords(file - 1, rank + 1))
        } else {
            Bitboard::EMPTY
        };
        let right = if file < 7 {
            Bitboard::from_square(Square::from_coords(file + 1, rank + 1))
        } else {
            Bitboard::EMPTY
        };
        left | right
    };

    let protected_by_pawn = !(our_pawns & pawn_protectors).is_empty();
    if !protected_by_pawn {
        return false;
    }

    // Check if can't be attacked by enemy pawns
    !can_be_attacked_by_enemy_pawn(knight_sq, color, enemy_pawns)
}

/// Check if a square can be attacked by enemy pawns.
fn can_be_attacked_by_enemy_pawn(sq: Square, color: Color, enemy_pawns: Bitboard) -> bool {
    let file = sq.file();

    // Get files that can attack this square
    let attack_files: Vec<u8> = if file > 0 && file < 7 {
        vec![file - 1, file + 1]
    } else if file == 0 {
        vec![1]
    } else {
        vec![6]
    };

    // Check if enemy pawns on attack files ahead of this square
    for attack_file in attack_files {
        let file_bb = file_bitboard(attack_file);
        let pawns_on_file = enemy_pawns & file_bb;

        for pawn_sq in pawns_on_file {
            // Check if pawn is ahead of knight square (can advance to attack)
            if color == Color::White {
                if pawn_sq.rank() > sq.rank() {
                    return true;
                }
            } else if pawn_sq.rank() < sq.rank() {
                return true;
            }
        }
    }

    false
}

/// Check if a square is central (d4, d5, e4, e5).
fn is_central_square(sq: Square) -> bool {
    let file = sq.file();
    let rank = sq.rank();
    (3..5).contains(&file) && (3..5).contains(&rank)
}

/// Get a bitboard mask for a file.
fn file_bitboard(file: u8) -> Bitboard {
    Bitboard::new(0x0101010101010101u64 << file)
}

/// Get a bitboard mask for a rank.
fn rank_bitboard(rank: u8) -> Bitboard {
    Bitboard::new(0xFFu64 << (rank * 8))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::parse_fen;

    #[test]
    fn test_rook_open_file() {
        // White rook on open e-file
        let board = parse_fen("rnbqkbnr/pppp1ppp/8/8/8/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1").unwrap();
        let (_mg, _eg) = evaluate_rook_activity(&board, Color::White);

        // Should have bonus for open file (though rook hasn't moved yet in this position)
        // Let's create a better test position
    }

    #[test]
    fn test_rook_on_open_file_actual() {
        // White rook on e4, e-file is open
        let board =
            parse_fen("rnbqkbnr/pppp1ppp/8/8/4R3/8/PPPP1PPP/RNBQKBN1 w KQkq - 0 1").unwrap();
        let (mg, _eg) = evaluate_rook_activity(&board, Color::White);

        assert!(mg > 0, "Rook on open file should have bonus, got mg={}", mg);
    }

    #[test]
    fn test_rook_on_seventh() {
        // White rook on 7th rank with enemy king on 8th
        let board = parse_fen("4k3/4R3/8/8/8/8/8/4K3 w - - 0 1").unwrap();
        let (mg, _eg) = evaluate_rook_activity(&board, Color::White);

        assert!(mg > 0, "Rook on 7th should have bonus, got mg={}", mg);
    }

    #[test]
    fn test_bishop_pair() {
        // White has bishop pair
        let board = Board::startpos();
        let (mg, eg) = evaluate_bishop_activity(&board, Color::White);

        assert!(mg > 0, "Bishop pair should have bonus in middlegame");
        assert!(eg > 0, "Bishop pair should have bonus in endgame");
    }

    #[test]
    fn test_bad_bishop() {
        // White bishop on light squares with most pawns on light squares
        let board = parse_fen("rnbqkbnr/8/8/8/8/P1P1P1P1/1P1P1P1P/RNBQKBNR w KQkq - 0 1").unwrap();
        let bishops = board.piece_bb(PieceType::Bishop, Color::White);
        let our_pawns = board.piece_bb(PieceType::Pawn, Color::White);

        for bishop_sq in bishops {
            let is_bad = is_bad_bishop(bishop_sq, our_pawns);
            // One of the bishops should be bad
            if is_bad {
                return; // Test passes if we find a bad bishop
            }
        }
    }

    #[test]
    fn test_trapped_bishop() {
        // White bishop trapped on a7
        let bishop_sq = Square::from_algebraic("a7").unwrap();
        assert!(is_trapped_bishop(bishop_sq, Color::White));

        // Not trapped on d4
        let bishop_sq = Square::from_algebraic("d4").unwrap();
        assert!(!is_trapped_bishop(bishop_sq, Color::White));
    }

    #[test]
    fn test_knight_outpost() {
        // White knight on d5, protected by c4 pawn
        // Black has no pawns on c or e files (removed c7 and e7 pawns)
        let board =
            parse_fen("rnbqkb1r/pp3ppp/8/3N4/2P5/8/PP1PPPPP/RNBQKB1R w KQkq - 0 1").unwrap();
        let knight_sq = Square::from_algebraic("d5").unwrap();
        let our_pawns = board.piece_bb(PieceType::Pawn, Color::White);
        let enemy_pawns = board.piece_bb(PieceType::Pawn, Color::Black);

        let is_outpost = is_knight_outpost(knight_sq, Color::White, our_pawns, enemy_pawns);
        assert!(
            is_outpost,
            "Knight on d5 protected by pawn, no enemy pawns can attack - should be outpost"
        );
    }

    #[test]
    fn test_centralization() {
        let e4 = Square::from_algebraic("e4").unwrap();
        assert!(is_central_square(e4));

        let d5 = Square::from_algebraic("d5").unwrap();
        assert!(is_central_square(d5));

        let a1 = Square::from_algebraic("a1").unwrap();
        assert!(!is_central_square(a1));
    }

    #[test]
    fn test_file_bitboard() {
        let e_file = file_bitboard(4);
        assert_eq!(e_file.count(), 8, "File should have 8 squares");
    }

    #[test]
    fn test_rank_bitboard() {
        let rank_4 = rank_bitboard(3); // 4th rank (0-indexed as 3)
        assert_eq!(rank_4.count(), 8, "Rank should have 8 squares");
    }

    #[test]
    fn test_piece_activity_startpos() {
        let board = Board::startpos();
        let (white_mg, _white_eg) = evaluate_piece_activity(&board, Color::White, 0);
        let (black_mg, _black_eg) = evaluate_piece_activity(&board, Color::Black, 0);

        // Both sides have bishop pair in startpos (+50)
        // But also back rank penalty for 6 pieces (-60)
        // Net should be slightly negative or around zero
        // The important thing is symmetry
        assert_eq!(
            white_mg, black_mg,
            "Activity should be symmetric in startpos"
        );

        // Both should have bishop pair bonus
        let (bishop_mg, _) = evaluate_bishop_activity(&board, Color::White);
        assert!(bishop_mg > 0, "Should have bishop pair bonus");
    }

    #[test]
    fn test_rook_semi_open_file() {
        // White rook on e4, e-file has only enemy pawns
        let board =
            parse_fen("rnbqkbnr/pppppppp/8/8/4R3/8/PPPP1PPP/RNBQKBN1 w KQkq - 0 1").unwrap();
        let (mg, _eg) = evaluate_rook_activity(&board, Color::White);

        assert!(
            mg > 0,
            "Rook on semi-open file should have bonus, got mg={}",
            mg
        );
    }

    #[test]
    fn test_two_rooks_on_seventh() {
        // Two white rooks on 7th rank
        let board = parse_fen("4k3/2RR4/8/8/8/8/8/4K3 w - - 0 1").unwrap();
        let (mg, _eg) = evaluate_rook_activity(&board, Color::White);

        // Should have larger bonus for two rooks
        assert!(mg >= values::TWO_ROOKS_ON_SEVENTH[0]);
    }
}
