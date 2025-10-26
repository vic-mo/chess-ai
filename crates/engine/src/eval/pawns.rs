//! Pawn structure evaluation with hash table caching.
//!
//! Evaluates:
//! - Doubled pawns (penalty -10 to -20 cp)
//! - Isolated pawns (penalty -15 to -25 cp)
//! - Backward pawns (penalty -10 to -15 cp)
//! - Passed pawns (bonus +20 to +150 cp by rank)
//! - Pawn chains (bonus +5 cp per protected pawn)
//! - Pawn islands (penalty -10 cp per island beyond 1)

use crate::bitboard::Bitboard;
use crate::board::Board;
use crate::piece::{Color, PieceType};
use crate::square::Square;

/// Pawn evaluation penalties and bonuses (in centipawns).
mod values {
    /// Doubled pawn penalty by file (central files worse)
    pub const DOUBLED_PAWN: [i32; 8] = [-15, -12, -18, -20, -20, -18, -12, -15];

    /// Isolated pawn penalty [mg, eg]
    pub const ISOLATED_PAWN: [i32; 2] = [-15, -20];

    /// Backward pawn penalty [mg, eg]
    pub const BACKWARD_PAWN: [i32; 2] = [-10, -15];

    /// Passed pawn bonus by rank (from white's perspective) [mg, eg]
    /// Indexed by rank: [1..=6] (no bonuses for 0 or 7)
    pub const PASSED_PAWN_BONUS: [[i32; 2]; 8] = [
        [0, 0],    // Rank 1 (impossible)
        [0, 0],    // Rank 2
        [10, 15],  // Rank 3
        [15, 25],  // Rank 4
        [30, 50],  // Rank 5
        [50, 90],  // Rank 6
        [80, 150], // Rank 7
        [0, 0],    // Rank 8 (promotion)
    ];

    /// Protected pawn (pawn chain) bonus [mg, eg]
    pub const PROTECTED_PAWN: [i32; 2] = [5, 10];

    /// Pawn island penalty (per island beyond 1) [mg, eg]
    pub const PAWN_ISLAND: [i32; 2] = [-10, -15];
}

/// Entry in the pawn hash table.
#[derive(Debug, Clone, Copy, Default)]
struct PawnEntry {
    /// Zobrist key for pawn positions only
    key: u64,
    /// Middlegame score
    mg_score: i16,
    /// Endgame score
    eg_score: i16,
}

/// Pawn hash table for caching pawn structure evaluations.
#[derive(Debug)]
pub struct PawnHashTable {
    entries: Vec<PawnEntry>,
    size: usize,
}

impl PawnHashTable {
    /// Create a new pawn hash table with given number of entries.
    ///
    /// Default size is 16384 entries (~512KB memory).
    pub fn new(size: usize) -> Self {
        let size = size.next_power_of_two();
        Self {
            entries: vec![PawnEntry::default(); size],
            size,
        }
    }

    /// Probe the hash table for a pawn structure evaluation.
    ///
    /// Returns (mg_score, eg_score) if found, None otherwise.
    pub fn probe(&self, key: u64) -> Option<(i32, i32)> {
        let index = (key as usize) & (self.size - 1);
        let entry = self.entries[index];

        if entry.key == key {
            Some((entry.mg_score as i32, entry.eg_score as i32))
        } else {
            None
        }
    }

    /// Store a pawn structure evaluation in the hash table.
    pub fn store(&mut self, key: u64, mg_score: i32, eg_score: i32) {
        let index = (key as usize) & (self.size - 1);
        self.entries[index] = PawnEntry {
            key,
            mg_score: mg_score as i16,
            eg_score: eg_score as i16,
        };
    }

    /// Clear the hash table.
    pub fn clear(&mut self) {
        self.entries.fill(PawnEntry::default());
    }
}

impl Default for PawnHashTable {
    fn default() -> Self {
        Self::new(16384) // 16K entries
    }
}

/// Calculate a Zobrist key for pawn positions only.
fn pawn_hash_key(board: &Board) -> u64 {
    use crate::zobrist::ZOBRIST;

    let mut key = 0u64;

    for color in [Color::White, Color::Black] {
        let pawns = board.piece_bb(PieceType::Pawn, color);
        for sq in pawns {
            key ^= ZOBRIST.pieces[PieceType::Pawn.index()][color.index()][sq.index() as usize];
        }
    }

    key
}

/// Evaluate pawn structure for a given color.
///
/// Returns (mg_score, eg_score) tuple.
fn evaluate_pawn_structure(board: &Board, color: Color) -> (i32, i32) {
    let mut mg_score = 0;
    let mut eg_score = 0;

    let our_pawns = board.piece_bb(PieceType::Pawn, color);
    let their_pawns = board.piece_bb(PieceType::Pawn, color.opponent());

    // Evaluate each pawn
    for sq in our_pawns {
        let file = sq.file();
        let rank = sq.rank();

        // Get file masks
        let file_mask = file_bitboard(file);
        let adjacent_files_mask = adjacent_files_bitboard(file);

        // Count pawns on this file
        let pawns_on_file = (our_pawns & file_mask).count();

        // 1. Doubled pawns
        if pawns_on_file > 1 {
            mg_score += values::DOUBLED_PAWN[file as usize];
            eg_score += values::DOUBLED_PAWN[file as usize];
        }

        // 2. Isolated pawns (no friendly pawns on adjacent files)
        let has_support = !(our_pawns & adjacent_files_mask).is_empty();
        if !has_support {
            mg_score += values::ISOLATED_PAWN[0];
            eg_score += values::ISOLATED_PAWN[1];
        }

        // 3. Backward pawns
        if !has_support && is_backward(sq, color, our_pawns, their_pawns) {
            mg_score += values::BACKWARD_PAWN[0];
            eg_score += values::BACKWARD_PAWN[1];
        }

        // 4. Passed pawns
        if is_passed(sq, color, their_pawns) {
            let bonus_rank = if color == Color::White {
                rank as usize
            } else {
                (7 - rank) as usize
            };
            mg_score += values::PASSED_PAWN_BONUS[bonus_rank][0];
            eg_score += values::PASSED_PAWN_BONUS[bonus_rank][1];
        }

        // 5. Protected pawns (pawn chains)
        if is_protected_by_pawn(sq, color, our_pawns) {
            mg_score += values::PROTECTED_PAWN[0];
            eg_score += values::PROTECTED_PAWN[1];
        }
    }

    // 6. Pawn islands
    let islands = count_pawn_islands(our_pawns);
    if islands > 1 {
        let penalty = (islands - 1) as i32;
        mg_score += penalty * values::PAWN_ISLAND[0];
        eg_score += penalty * values::PAWN_ISLAND[1];
    }

    (mg_score, eg_score)
}

/// Check if a pawn is passed (no enemy pawns in front on same or adjacent files).
fn is_passed(sq: Square, color: Color, enemy_pawns: Bitboard) -> bool {
    let file = sq.file();
    let rank = sq.rank();

    // Create mask for squares in front
    let front_mask = if color == Color::White {
        // Ranks above current rank
        passed_pawn_mask_white(file, rank)
    } else {
        // Ranks below current rank
        passed_pawn_mask_black(file, rank)
    };

    // Check if any enemy pawns in the way
    (enemy_pawns & front_mask).is_empty()
}

/// Check if a pawn is backward.
///
/// A pawn is backward if:
/// - It has no support from adjacent pawns
/// - It cannot safely advance
/// - The square in front is weak
fn is_backward(sq: Square, color: Color, our_pawns: Bitboard, enemy_pawns: Bitboard) -> bool {
    let file = sq.file();
    let rank = sq.rank();

    // Get the square in front
    let front_sq = if color == Color::White {
        if rank >= 7 {
            return false;
        }
        Square::from_coords(file, rank + 1)
    } else {
        if rank == 0 {
            return false;
        }
        Square::from_coords(file, rank - 1)
    };

    // Check if the front square is controlled by enemy pawns
    let enemy_control = pawn_attacks(enemy_pawns, color.opponent());
    if enemy_control.contains(front_sq) {
        // Check if we have supporting pawns that could advance
        let adjacent_files = adjacent_files_bitboard(file);
        let support_mask = if color == Color::White {
            // Pawns behind us on adjacent files
            let behind_mask = Bitboard::new(!0u64 << (rank * 8));
            our_pawns & adjacent_files & behind_mask
        } else {
            let behind_mask = Bitboard::new(!0u64 >> ((7 - rank) * 8));
            our_pawns & adjacent_files & behind_mask
        };

        return support_mask.is_empty();
    }

    false
}

/// Check if a pawn is protected by another friendly pawn.
fn is_protected_by_pawn(sq: Square, color: Color, our_pawns: Bitboard) -> bool {
    let protectors = pawn_attacks(our_pawns, color.opponent());
    protectors.contains(sq)
}

/// Calculate pawn attacks for all pawns of a given color.
fn pawn_attacks(pawns: Bitboard, color: Color) -> Bitboard {
    use crate::attacks::pawn_attacks as get_pawn_attacks;

    let mut attacks = Bitboard::EMPTY;
    for sq in pawns {
        attacks |= get_pawn_attacks(sq, color);
    }
    attacks
}

/// Count the number of pawn islands.
///
/// A pawn island is a group of connected pawns on adjacent files.
fn count_pawn_islands(pawns: Bitboard) -> u32 {
    let mut islands = 0;
    let mut in_island = false;

    for file in 0..8 {
        let file_mask = file_bitboard(file);
        let has_pawn = !(pawns & file_mask).is_empty();

        if has_pawn && !in_island {
            islands += 1;
            in_island = true;
        } else if !has_pawn {
            in_island = false;
        }
    }

    islands
}

/// Get a bitboard mask for a file.
fn file_bitboard(file: u8) -> Bitboard {
    Bitboard::new(0x0101010101010101u64 << file)
}

/// Get a bitboard mask for adjacent files.
fn adjacent_files_bitboard(file: u8) -> Bitboard {
    let mut mask = Bitboard::EMPTY;
    if file > 0 {
        mask |= file_bitboard(file - 1);
    }
    if file < 7 {
        mask |= file_bitboard(file + 1);
    }
    mask
}

/// Get passed pawn mask for white (squares in front on same and adjacent files).
fn passed_pawn_mask_white(file: u8, rank: u8) -> Bitboard {
    let file_mask = file_bitboard(file);
    let adjacent_mask = adjacent_files_bitboard(file);
    let all_files = file_mask | adjacent_mask;

    // Mask ranks above current rank
    let rank_mask = Bitboard::new(!0u64 << ((rank + 1) * 8));

    all_files & rank_mask
}

/// Get passed pawn mask for black (squares in front on same and adjacent files).
fn passed_pawn_mask_black(file: u8, rank: u8) -> Bitboard {
    let file_mask = file_bitboard(file);
    let adjacent_mask = adjacent_files_bitboard(file);
    let all_files = file_mask | adjacent_mask;

    // Mask ranks below current rank
    let rank_mask = if rank > 0 {
        Bitboard::new((1u64 << (rank * 8)) - 1)
    } else {
        Bitboard::EMPTY
    };

    all_files & rank_mask
}

/// Evaluate pawn structure using the hash table.
pub fn evaluate_pawns_cached(
    board: &Board,
    pawn_table: &mut PawnHashTable,
) -> (i32, i32, i32, i32) {
    let key = pawn_hash_key(board);

    // Try to probe the hash table
    if let Some((_white_mg, _white_eg)) = pawn_table.probe(key) {
        // Note: We store the evaluation relative to white
        // Need to evaluate both colors separately
        // For now, we'll re-evaluate if we miss
        // TODO: Implement proper caching that returns both colors
    }

    // Evaluate both colors
    let (white_mg, white_eg) = evaluate_pawn_structure(board, Color::White);
    let (black_mg, black_eg) = evaluate_pawn_structure(board, Color::Black);

    // Store in hash table (storing white's perspective)
    pawn_table.store(key, white_mg - black_mg, white_eg - black_eg);

    (white_mg, white_eg, black_mg, black_eg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::parse_fen;

    #[test]
    fn test_doubled_pawns() {
        // White has doubled pawns on e-file
        let board = parse_fen("8/8/8/4p3/4P3/4P3/8/8 w - - 0 1").unwrap();
        let (mg, eg) = evaluate_pawn_structure(&board, Color::White);

        // Should have penalty for doubled pawns
        assert!(mg < 0, "Doubled pawns should have penalty");
        assert!(eg < 0, "Doubled pawns should have penalty in endgame");
    }

    #[test]
    fn test_isolated_pawns() {
        // White has isolated pawn on e4, with enemy pawn on e6 (so not passed)
        let board = parse_fen("8/8/4p3/8/4P3/8/8/8 w - - 0 1").unwrap();
        let (mg, eg) = evaluate_pawn_structure(&board, Color::White);

        // Should have penalty for isolated pawn
        // Note: This pawn is isolated AND blocked, so it should be negative
        assert!(mg < 0, "Isolated pawn should have penalty, got mg={}", mg);
        assert!(
            eg < 0,
            "Isolated pawn should have penalty in endgame, got eg={}",
            eg
        );
    }

    #[test]
    fn test_passed_pawns() {
        // White has passed pawn on e6
        let board = parse_fen("8/8/4P3/8/8/8/p7/8 w - - 0 1").unwrap();
        let (mg, eg) = evaluate_pawn_structure(&board, Color::White);

        // Should have bonus for passed pawn
        assert!(mg > 0, "Passed pawn should have bonus");
        assert!(eg > 0, "Passed pawn should have bonus in endgame");
    }

    #[test]
    fn test_protected_pawns() {
        // White has pawn chain d4-e5
        let board = parse_fen("8/8/8/4P3/3P4/8/8/8 w - - 0 1").unwrap();
        let (mg, _eg) = evaluate_pawn_structure(&board, Color::White);

        // Should have bonus for protected pawns
        assert!(mg > 0, "Protected pawns should have bonus");
    }

    #[test]
    fn test_pawn_islands() {
        // White has 3 pawn islands: a2, c4-d4, g5
        let board = parse_fen("8/8/8/6P1/2PP4/8/P7/8 w - - 0 1").unwrap();
        let islands = count_pawn_islands(board.piece_bb(PieceType::Pawn, Color::White));

        assert_eq!(islands, 3, "Should count 3 pawn islands");
    }

    #[test]
    fn test_pawn_hash_table() {
        let mut table = PawnHashTable::new(16);
        let board = Board::startpos();
        let key = pawn_hash_key(&board);

        // Store evaluation
        table.store(key, 100, 150);

        // Probe should return same values
        let result = table.probe(key);
        assert_eq!(result, Some((100, 150)));

        // Different key should miss
        let result = table.probe(key + 1);
        assert_eq!(result, None);
    }

    #[test]
    fn test_pawn_hash_key_stability() {
        let board = Board::startpos();
        let key1 = pawn_hash_key(&board);
        let key2 = pawn_hash_key(&board);

        assert_eq!(key1, key2, "Pawn hash key should be stable");
    }

    #[test]
    fn test_file_bitboard() {
        let file_e = file_bitboard(4); // e-file
        assert_eq!(file_e.count(), 8, "File should have 8 squares");

        // Check that e1, e2, ..., e8 are set
        for rank in 0..8 {
            let sq = Square::from_coords(4, rank);
            assert!(
                file_e.contains(sq),
                "File bitboard should contain square {}",
                sq.to_algebraic()
            );
        }
    }

    #[test]
    fn test_adjacent_files() {
        let adjacent = adjacent_files_bitboard(4); // d and f files
        assert_eq!(
            adjacent.count(),
            16,
            "Adjacent files should have 16 squares"
        );

        // Check d-file
        for rank in 0..8 {
            let sq = Square::from_coords(3, rank);
            assert!(
                adjacent.contains(sq),
                "Adjacent files should contain d-file"
            );
        }

        // Check f-file
        for rank in 0..8 {
            let sq = Square::from_coords(5, rank);
            assert!(
                adjacent.contains(sq),
                "Adjacent files should contain f-file"
            );
        }
    }

    #[test]
    fn test_passed_pawn_detection_white() {
        // White pawn on e5, no black pawns blocking
        let board = parse_fen("8/8/8/4P3/8/8/8/8 w - - 0 1").unwrap();
        let black_pawns = board.piece_bb(PieceType::Pawn, Color::Black);

        let pawn_sq = Square::from_algebraic("e5").unwrap();
        assert!(is_passed(pawn_sq, Color::White, black_pawns));
    }

    #[test]
    fn test_not_passed_pawn() {
        // White pawn on e4, black pawn on e6 blocks
        let board = parse_fen("8/8/4p3/8/4P3/8/8/8 w - - 0 1").unwrap();
        let black_pawns = board.piece_bb(PieceType::Pawn, Color::Black);

        let pawn_sq = Square::from_algebraic("e4").unwrap();
        assert!(!is_passed(pawn_sq, Color::White, black_pawns));
    }

    #[test]
    fn test_pawn_islands_connected() {
        // White has 1 island: e4-f4-g4
        let board = parse_fen("8/8/8/8/4PPP1/8/8/8 w - - 0 1").unwrap();
        let islands = count_pawn_islands(board.piece_bb(PieceType::Pawn, Color::White));

        assert_eq!(islands, 1, "Connected pawns should be 1 island");
    }

    #[test]
    fn test_startpos_no_weaknesses() {
        let board = Board::startpos();
        let (white_mg, white_eg) = evaluate_pawn_structure(&board, Color::White);

        // Starting position should have neutral/zero pawn structure
        // No doubled, isolated, or passed pawns
        // No pawn chains either (all on same rank)
        assert_eq!(
            white_mg, 0,
            "Startpos should have neutral pawn structure score, got mg={}",
            white_mg
        );
        assert_eq!(
            white_eg, 0,
            "Startpos should have neutral pawn structure score in endgame, got eg={}",
            white_eg
        );
    }
}
