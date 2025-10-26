use crate::bitboard::Bitboard;
use crate::piece::Color;
use crate::square::Square;
use once_cell::sync::Lazy;

/// Precomputed attack tables for non-sliding pieces.
///
/// These tables are computed at startup and provide O(1) lookup
/// for pawn, knight, and king attacks.
// =============================================================================
// PAWN ATTACKS
// =============================================================================
/// Pawn attacks for each color and square.
/// Index: [color][square]
static PAWN_ATTACKS: Lazy<[[Bitboard; 64]; 2]> = Lazy::new(|| {
    let mut attacks = [[Bitboard::EMPTY; 64]; 2];

    for sq_idx in 0..64 {
        let square = Square::new(sq_idx as u8);
        let file = square.file();
        let rank = square.rank();

        // White pawns attack diagonally up
        if rank < 7 {
            let target_rank = rank + 1;
            if file > 0 {
                let target_sq = Square::from_coords(file - 1, target_rank);
                attacks[Color::White.index()][sq_idx] =
                    attacks[Color::White.index()][sq_idx].set(target_sq);
            }
            if file < 7 {
                let target_sq = Square::from_coords(file + 1, target_rank);
                attacks[Color::White.index()][sq_idx] =
                    attacks[Color::White.index()][sq_idx].set(target_sq);
            }
        }

        // Black pawns attack diagonally down
        if rank > 0 {
            let target_rank = rank - 1;
            if file > 0 {
                let target_sq = Square::from_coords(file - 1, target_rank);
                attacks[Color::Black.index()][sq_idx] =
                    attacks[Color::Black.index()][sq_idx].set(target_sq);
            }
            if file < 7 {
                let target_sq = Square::from_coords(file + 1, target_rank);
                attacks[Color::Black.index()][sq_idx] =
                    attacks[Color::Black.index()][sq_idx].set(target_sq);
            }
        }
    }

    attacks
});

/// Returns the pawn attack bitboard for a given square and color.
///
/// # Example
/// ```
/// use engine::attacks::pawn_attacks;
/// use engine::piece::Color;
/// use engine::square::Square;
///
/// let attacks = pawn_attacks(Square::E4, Color::White);
/// assert!(attacks.contains(Square::D5));
/// assert!(attacks.contains(Square::from_coords(5, 4))); // F5
/// ```
#[inline(always)]
pub fn pawn_attacks(square: Square, color: Color) -> Bitboard {
    PAWN_ATTACKS[color.index()][square.index() as usize]
}

// =============================================================================
// KNIGHT ATTACKS
// =============================================================================

/// Knight attack deltas (file_delta, rank_delta)
const KNIGHT_DELTAS: [(i8, i8); 8] = [
    (-2, -1),
    (-2, 1),
    (-1, -2),
    (-1, 2),
    (1, -2),
    (1, 2),
    (2, -1),
    (2, 1),
];

/// Precomputed knight attacks for each square.
static KNIGHT_ATTACKS: Lazy<[Bitboard; 64]> = Lazy::new(|| {
    let mut attacks = [Bitboard::EMPTY; 64];

    for (sq_idx, attack) in attacks.iter_mut().enumerate() {
        let square = Square::new(sq_idx as u8);
        let file = square.file() as i8;
        let rank = square.rank() as i8;

        for &(df, dr) in &KNIGHT_DELTAS {
            let new_file = file + df;
            let new_rank = rank + dr;

            if (0..8).contains(&new_file) && (0..8).contains(&new_rank) {
                let target_sq = Square::from_coords(new_file as u8, new_rank as u8);
                *attack = attack.set(target_sq);
            }
        }
    }

    attacks
});

/// Returns the knight attack bitboard for a given square.
///
/// # Example
/// ```
/// use engine::attacks::knight_attacks;
/// use engine::square::Square;
///
/// let attacks = knight_attacks(Square::E4);
/// assert_eq!(attacks.count(), 8); // Knight in center has 8 moves
/// ```
#[inline(always)]
pub fn knight_attacks(square: Square) -> Bitboard {
    KNIGHT_ATTACKS[square.index() as usize]
}

// =============================================================================
// KING ATTACKS
// =============================================================================

/// King attack deltas (all 8 adjacent squares)
const KING_DELTAS: [(i8, i8); 8] = [
    (-1, -1),
    (-1, 0),
    (-1, 1),
    (0, -1),
    (0, 1),
    (1, -1),
    (1, 0),
    (1, 1),
];

/// Precomputed king attacks for each square.
static KING_ATTACKS: Lazy<[Bitboard; 64]> = Lazy::new(|| {
    let mut attacks = [Bitboard::EMPTY; 64];

    for (sq_idx, attack) in attacks.iter_mut().enumerate() {
        let square = Square::new(sq_idx as u8);
        let file = square.file() as i8;
        let rank = square.rank() as i8;

        for &(df, dr) in &KING_DELTAS {
            let new_file = file + df;
            let new_rank = rank + dr;

            if (0..8).contains(&new_file) && (0..8).contains(&new_rank) {
                let target_sq = Square::from_coords(new_file as u8, new_rank as u8);
                *attack = attack.set(target_sq);
            }
        }
    }

    attacks
});

/// Returns the king attack bitboard for a given square.
///
/// # Example
/// ```
/// use engine::attacks::king_attacks;
/// use engine::square::Square;
///
/// let attacks = king_attacks(Square::E4);
/// assert_eq!(attacks.count(), 8); // King in center has 8 moves
/// ```
#[inline(always)]
pub fn king_attacks(square: Square) -> Bitboard {
    KING_ATTACKS[square.index() as usize]
}

// =============================================================================
// SLIDING PIECE ATTACKS (Bishop, Rook, Queen)
// =============================================================================

/// Ray directions for sliding pieces.
/// (file_delta, rank_delta)
const BISHOP_DIRECTIONS: [(i8, i8); 4] = [
    (-1, -1), // Southwest
    (-1, 1),  // Northwest
    (1, -1),  // Southeast
    (1, 1),   // Northeast
];

const ROOK_DIRECTIONS: [(i8, i8); 4] = [
    (-1, 0), // West
    (1, 0),  // East
    (0, -1), // South
    (0, 1),  // North
];

/// Generate sliding attacks along given directions, stopping at blockers.
///
/// This is the classical approach (non-magic bitboards). It's simple and
/// fast enough for our purposes.
fn sliding_attacks(square: Square, occupancy: Bitboard, directions: &[(i8, i8)]) -> Bitboard {
    let mut attacks = Bitboard::EMPTY;
    let file = square.file() as i8;
    let rank = square.rank() as i8;

    for &(df, dr) in directions {
        let mut current_file = file + df;
        let mut current_rank = rank + dr;

        while (0..8).contains(&current_file) && (0..8).contains(&current_rank) {
            let target_sq = Square::from_coords(current_file as u8, current_rank as u8);
            attacks = attacks.set(target_sq);

            // Stop if we hit a blocker
            if occupancy.contains(target_sq) {
                break;
            }

            current_file += df;
            current_rank += dr;
        }
    }

    attacks
}

/// Returns bishop attack bitboard for a given square and occupancy.
///
/// # Example
/// ```
/// use engine::attacks::bishop_attacks;
/// use engine::bitboard::Bitboard;
/// use engine::square::Square;
///
/// let occupancy = Bitboard::EMPTY;
/// let attacks = bishop_attacks(Square::E4, occupancy);
/// assert!(attacks.contains(Square::A8)); // Can reach corner on empty board
/// ```
#[inline(always)]
pub fn bishop_attacks(square: Square, occupancy: Bitboard) -> Bitboard {
    sliding_attacks(square, occupancy, &BISHOP_DIRECTIONS)
}

/// Returns rook attack bitboard for a given square and occupancy.
///
/// # Example
/// ```
/// use engine::attacks::rook_attacks;
/// use engine::bitboard::Bitboard;
/// use engine::square::Square;
///
/// let occupancy = Bitboard::EMPTY;
/// let attacks = rook_attacks(Square::A1, occupancy);
/// assert!(attacks.contains(Square::A8)); // Can reach same file
/// assert!(attacks.contains(Square::H1)); // Can reach same rank
/// ```
#[inline(always)]
pub fn rook_attacks(square: Square, occupancy: Bitboard) -> Bitboard {
    sliding_attacks(square, occupancy, &ROOK_DIRECTIONS)
}

/// Returns queen attack bitboard for a given square and occupancy.
///
/// Queens combine bishop and rook moves.
///
/// # Example
/// ```
/// use engine::attacks::queen_attacks;
/// use engine::bitboard::Bitboard;
/// use engine::square::Square;
///
/// let occupancy = Bitboard::EMPTY;
/// let attacks = queen_attacks(Square::E4, occupancy);
/// assert_eq!(attacks.count(), 27); // Queen in center with empty board
/// ```
#[inline(always)]
pub fn queen_attacks(square: Square, occupancy: Bitboard) -> Bitboard {
    bishop_attacks(square, occupancy) | rook_attacks(square, occupancy)
}

// =============================================================================
// INITIALIZATION
// =============================================================================

/// Force initialization of all attack tables.
///
/// This is called automatically on first use due to `Lazy`, but can be
/// called explicitly to avoid first-use latency.
pub fn init() {
    Lazy::force(&PAWN_ATTACKS);
    Lazy::force(&KNIGHT_ATTACKS);
    Lazy::force(&KING_ATTACKS);
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pawn_attacks_white() {
        // E4 = coords(4, 3), attacks to D5=coords(3,4) and F5=coords(5,4)
        let attacks = pawn_attacks(Square::E4, Color::White);
        assert!(attacks.contains(Square::D5)); // D5 = coords(3, 4)
        assert!(attacks.contains(Square::from_coords(5, 4))); // F5
        assert_eq!(attacks.count(), 2);
    }

    #[test]
    fn test_pawn_attacks_black() {
        // E4 = coords(4, 3), black attacks down to D3=coords(3,2) and F3=coords(5,2)
        let attacks = pawn_attacks(Square::E4, Color::Black);
        assert!(attacks.contains(Square::from_coords(3, 2))); // D3
        assert!(attacks.contains(Square::from_coords(5, 2))); // F3
        assert_eq!(attacks.count(), 2);
    }

    #[test]
    fn test_pawn_attacks_edge() {
        // A2 = coords(0, 1), white attacks to B3=coords(1,2)
        let attacks = pawn_attacks(Square::A2, Color::White);
        assert_eq!(attacks.count(), 1);
        assert!(attacks.contains(Square::from_coords(1, 2))); // B3

        // H7 = coords(7, 6), black attacks to G6=coords(6,5)
        let attacks = pawn_attacks(Square::H7, Color::Black);
        assert_eq!(attacks.count(), 1);
        assert!(attacks.contains(Square::from_coords(6, 5))); // G6
    }

    #[test]
    fn test_knight_attacks_center() {
        // E4 = coords(4, 3)
        let attacks = knight_attacks(Square::E4);
        assert_eq!(attacks.count(), 8);

        // Check all 8 knight moves from E4
        assert!(attacks.contains(Square::D2)); // D2 = coords(3, 1)
        assert!(attacks.contains(Square::from_coords(5, 1))); // F2
        assert!(attacks.contains(Square::from_coords(2, 2))); // C3
        assert!(attacks.contains(Square::from_coords(6, 2))); // G3
        assert!(attacks.contains(Square::from_coords(2, 4))); // C5
        assert!(attacks.contains(Square::from_coords(6, 4))); // G5
        assert!(attacks.contains(Square::D6)); // D6 = coords(3, 5)
        assert!(attacks.contains(Square::from_coords(5, 5))); // F6
    }

    #[test]
    fn test_knight_attacks_corner() {
        // Knight in corner has only 2 moves
        let attacks = knight_attacks(Square::A1);
        assert_eq!(attacks.count(), 2);
        assert!(attacks.contains(Square::from_coords(2, 1))); // c2
        assert!(attacks.contains(Square::from_coords(1, 2))); // b3
    }

    #[test]
    fn test_king_attacks_center() {
        // E4 = coords(4, 3)
        let attacks = king_attacks(Square::E4);
        assert_eq!(attacks.count(), 8);

        // Check all 8 adjacent squares
        assert!(attacks.contains(Square::from_coords(3, 2))); // D3
        assert!(attacks.contains(Square::E3)); // E3 = coords(4, 2)
        assert!(attacks.contains(Square::from_coords(5, 2))); // F3
        assert!(attacks.contains(Square::D4)); // D4 = coords(3, 3)
        assert!(attacks.contains(Square::from_coords(5, 3))); // F4
        assert!(attacks.contains(Square::D5)); // D5 = coords(3, 4)
        assert!(attacks.contains(Square::E5)); // E5 = coords(4, 4)
        assert!(attacks.contains(Square::from_coords(5, 4))); // F5
    }

    #[test]
    fn test_king_attacks_corner() {
        // King in corner has only 3 moves
        let attacks = king_attacks(Square::A1);
        assert_eq!(attacks.count(), 3);
        assert!(attacks.contains(Square::A2));
        assert!(attacks.contains(Square::B1));
        assert!(attacks.contains(Square::from_coords(1, 1))); // b2
    }

    #[test]
    fn test_bishop_attacks_empty_board() {
        let occupancy = Bitboard::EMPTY;
        // E4 = coords(4, 3)
        let attacks = bishop_attacks(Square::E4, occupancy);

        // Should reach all diagonal squares
        assert!(attacks.contains(Square::A8)); // A8 = coords(0, 7) - NW diagonal
        assert!(attacks.contains(Square::from_coords(7, 0))); // H1 - SE diagonal
        assert!(attacks.contains(Square::D5)); // D5 = coords(3, 4) - NW
        assert!(attacks.contains(Square::from_coords(5, 2))); // F3 - SE

        // Should not reach non-diagonal squares
        assert!(!attacks.contains(Square::E5)); // Same file
        assert!(!attacks.contains(Square::D4)); // Same rank - 1 file
    }

    #[test]
    fn test_bishop_attacks_with_blocker() {
        let mut occupancy = Bitboard::EMPTY;
        occupancy = occupancy.set(Square::from_coords(5, 4)); // Blocker on F5

        // E4 = coords(4, 3)
        let attacks = bishop_attacks(Square::E4, occupancy);

        // Should include the blocker square F5
        assert!(attacks.contains(Square::from_coords(5, 4)));

        // Should not go beyond the blocker (F5 blocks to G6, H7)
        assert!(!attacks.contains(Square::from_coords(6, 5))); // G6
        assert!(!attacks.contains(Square::from_coords(7, 6))); // H7
    }

    #[test]
    fn test_rook_attacks_empty_board() {
        let occupancy = Bitboard::EMPTY;
        let attacks = rook_attacks(Square::A1, occupancy);

        // Should reach entire file and rank
        assert!(attacks.contains(Square::A8));
        assert!(attacks.contains(Square::H1));
        assert_eq!(attacks.count(), 14); // 7 squares up + 7 squares right

        // Should not reach diagonal squares
        assert!(!attacks.contains(Square::from_coords(1, 1))); // b2
    }

    #[test]
    fn test_rook_attacks_with_blocker() {
        let mut occupancy = Bitboard::EMPTY;
        occupancy = occupancy.set(Square::E4); // Blocker on e4

        let attacks = rook_attacks(Square::E1, occupancy);

        // Should include the blocker
        assert!(attacks.contains(Square::E4));

        // Should not go beyond the blocker
        assert!(!attacks.contains(Square::E5));
        assert!(!attacks.contains(Square::E8));
    }

    #[test]
    fn test_queen_attacks() {
        let occupancy = Bitboard::EMPTY;
        let attacks = queen_attacks(Square::E4, occupancy);

        // Queen should combine bishop and rook moves
        let bishop_atk = bishop_attacks(Square::E4, occupancy);
        let rook_atk = rook_attacks(Square::E4, occupancy);
        let combined = bishop_atk | rook_atk;

        assert_eq!(attacks, combined);
        assert_eq!(attacks.count(), 27); // Queen in center on empty board
    }

    #[test]
    fn test_init() {
        // Just ensure it doesn't panic
        init();
    }
}
