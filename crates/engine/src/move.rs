use crate::piece::PieceType;
use crate::square::Square;

/// A chess move packed into 16 bits.
///
/// Bit layout:
/// - Bits 0-5: from square (0-63)
/// - Bits 6-11: to square (0-63)
/// - Bits 12-15: move flags (0-15)
///
/// # Example
/// ```
/// use engine::r#move::{Move, MoveFlags};
/// use engine::square::Square;
///
/// let m = Move::new(Square::E2, Square::E4, MoveFlags::DOUBLE_PAWN_PUSH);
/// assert_eq!(m.from(), Square::E2);
/// assert_eq!(m.to(), Square::E4);
/// assert!(m.is_double_pawn_push());
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Move(u16);

/// Move flags encoded in 4 bits (0-15).
///
/// Layout:
/// - Bits 0-1: Special move type (quiet, double push, castling, etc.)
/// - Bit 2: Capture flag
/// - Bit 3: Promotion flag
///
/// This encoding allows efficient checking:
/// - `flags & 0x4` checks if capture
/// - `flags & 0x8` checks if promotion
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct MoveFlags(u8);

impl MoveFlags {
    // Quiet moves (0b00xx)
    pub const QUIET: Self = Self(0);
    pub const DOUBLE_PAWN_PUSH: Self = Self(1);
    pub const KING_CASTLE: Self = Self(2);
    pub const QUEEN_CASTLE: Self = Self(3);

    // Captures (0b01xx)
    pub const CAPTURE: Self = Self(4);
    pub const EP_CAPTURE: Self = Self(5); // En passant capture

    // Promotions - quiet (0b10xx)
    pub const KNIGHT_PROMOTION: Self = Self(8);
    pub const BISHOP_PROMOTION: Self = Self(9);
    pub const ROOK_PROMOTION: Self = Self(10);
    pub const QUEEN_PROMOTION: Self = Self(11);

    // Promotions - captures (0b11xx)
    pub const KNIGHT_PROMOTION_CAPTURE: Self = Self(12);
    pub const BISHOP_PROMOTION_CAPTURE: Self = Self(13);
    pub const ROOK_PROMOTION_CAPTURE: Self = Self(14);
    pub const QUEEN_PROMOTION_CAPTURE: Self = Self(15);

    /// Returns true if this move is a capture (including ep and promotion captures).
    #[inline(always)]
    pub fn is_capture(self) -> bool {
        (self.0 & 0x4) != 0
    }

    /// Returns true if this move is a promotion.
    #[inline(always)]
    pub fn is_promotion(self) -> bool {
        (self.0 & 0x8) != 0
    }

    /// Returns the promoted piece type, or None if not a promotion.
    pub fn promotion_piece(self) -> Option<PieceType> {
        if !self.is_promotion() {
            return None;
        }
        match self.0 & 0x3 {
            0 => Some(PieceType::Knight),
            1 => Some(PieceType::Bishop),
            2 => Some(PieceType::Rook),
            3 => Some(PieceType::Queen),
            _ => unreachable!(),
        }
    }

    /// Returns true if this is a castling move.
    #[inline(always)]
    pub fn is_castling(self) -> bool {
        matches!(self.0, 2 | 3)
    }

    /// Returns true if this is kingside castling.
    #[inline(always)]
    pub fn is_kingside_castle(self) -> bool {
        self.0 == 2
    }

    /// Returns true if this is queenside castling.
    #[inline(always)]
    pub fn is_queenside_castle(self) -> bool {
        self.0 == 3
    }

    /// Returns true if this is en passant capture.
    #[inline(always)]
    pub fn is_en_passant(self) -> bool {
        self.0 == 5
    }

    /// Returns true if this is a double pawn push.
    #[inline(always)]
    pub fn is_double_pawn_push(self) -> bool {
        self.0 == 1
    }

    /// Returns the raw flag value.
    #[inline(always)]
    pub fn value(self) -> u8 {
        self.0
    }
}

impl Move {
    /// Creates a new move from source square, destination square, and flags.
    #[inline(always)]
    pub fn new(from: Square, to: Square, flags: MoveFlags) -> Self {
        let from_bits = from.index() as u16;
        let to_bits = (to.index() as u16) << 6;
        let flag_bits = (flags.0 as u16) << 12;
        Self(from_bits | to_bits | flag_bits)
    }

    /// Creates a null move (a1a1 with no flags). Used as a sentinel value.
    #[inline(always)]
    pub fn null() -> Self {
        Self(0)
    }

    /// Returns true if this is a null move.
    #[inline(always)]
    pub fn is_null(self) -> bool {
        self.0 == 0
    }

    /// Returns the source square.
    #[inline(always)]
    pub fn from(self) -> Square {
        Square::new((self.0 & 0x3F) as u8)
    }

    /// Returns the destination square.
    #[inline(always)]
    pub fn to(self) -> Square {
        Square::new(((self.0 >> 6) & 0x3F) as u8)
    }

    /// Returns the move flags.
    #[inline(always)]
    pub fn flags(self) -> MoveFlags {
        MoveFlags((self.0 >> 12) as u8)
    }

    /// Returns true if this move is a capture.
    #[inline(always)]
    pub fn is_capture(self) -> bool {
        self.flags().is_capture()
    }

    /// Returns true if this move is a promotion.
    #[inline(always)]
    pub fn is_promotion(self) -> bool {
        self.flags().is_promotion()
    }

    /// Returns true if this is a quiet move (not capture or promotion).
    #[inline(always)]
    pub fn is_quiet(self) -> bool {
        (self.0 >> 12) < 4
    }

    /// Returns true if this is a double pawn push.
    #[inline(always)]
    pub fn is_double_pawn_push(self) -> bool {
        self.flags().is_double_pawn_push()
    }

    /// Returns true if this is a castling move.
    #[inline(always)]
    pub fn is_castling(self) -> bool {
        self.flags().is_castling()
    }

    /// Returns true if this is kingside castling.
    #[inline(always)]
    pub fn is_kingside_castle(self) -> bool {
        self.flags().is_kingside_castle()
    }

    /// Returns true if this is queenside castling.
    #[inline(always)]
    pub fn is_queenside_castle(self) -> bool {
        self.flags().is_queenside_castle()
    }

    /// Returns true if this is an en passant capture.
    #[inline(always)]
    pub fn is_en_passant(self) -> bool {
        self.flags().is_en_passant()
    }

    /// Returns the promoted piece type, or None if not a promotion.
    #[inline(always)]
    pub fn promotion_piece(self) -> Option<PieceType> {
        self.flags().promotion_piece()
    }

    /// Returns a UCI-style move string (e.g., "e2e4", "e7e8q").
    pub fn to_uci(self) -> String {
        let from = self.from().to_algebraic();
        let to = self.to().to_algebraic();

        if let Some(piece) = self.promotion_piece() {
            let promo_char = match piece {
                PieceType::Knight => 'n',
                PieceType::Bishop => 'b',
                PieceType::Rook => 'r',
                PieceType::Queen => 'q',
                _ => unreachable!(),
            };
            format!("{}{}{}", from, to, promo_char)
        } else {
            format!("{}{}", from, to)
        }
    }
}

impl std::fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_uci())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_packing_quiet() {
        let m = Move::new(Square::E2, Square::E4, MoveFlags::QUIET);
        assert_eq!(m.from(), Square::E2);
        assert_eq!(m.to(), Square::E4);
        assert_eq!(m.flags(), MoveFlags::QUIET);
        assert!(m.is_quiet());
        assert!(!m.is_capture());
    }

    #[test]
    fn move_packing_double_pawn_push() {
        let m = Move::new(Square::E2, Square::E4, MoveFlags::DOUBLE_PAWN_PUSH);
        assert_eq!(m.from(), Square::E2);
        assert_eq!(m.to(), Square::E4);
        assert!(m.is_double_pawn_push());
        assert!(!m.is_capture());
    }

    #[test]
    fn move_packing_capture() {
        let m = Move::new(Square::E4, Square::D5, MoveFlags::CAPTURE);
        assert_eq!(m.from(), Square::E4);
        assert_eq!(m.to(), Square::D5);
        assert!(m.is_capture());
        assert!(!m.is_promotion());
    }

    #[test]
    fn move_packing_castling() {
        let kingside = Move::new(Square::E1, Square::G1, MoveFlags::KING_CASTLE);
        assert!(kingside.is_castling());
        assert!(!kingside.is_capture());

        let queenside = Move::new(Square::E1, Square::C1, MoveFlags::QUEEN_CASTLE);
        assert!(queenside.is_castling());
    }

    #[test]
    fn move_packing_en_passant() {
        let m = Move::new(Square::E5, Square::D6, MoveFlags::EP_CAPTURE);
        assert!(m.is_en_passant());
        assert!(m.is_capture());
    }

    #[test]
    fn move_packing_promotion_quiet() {
        let m = Move::new(Square::E7, Square::E8, MoveFlags::QUEEN_PROMOTION);
        assert!(m.is_promotion());
        assert!(!m.is_capture());
        assert_eq!(m.promotion_piece(), Some(PieceType::Queen));
    }

    #[test]
    fn move_packing_promotion_capture() {
        let m = Move::new(Square::E7, Square::D8, MoveFlags::KNIGHT_PROMOTION_CAPTURE);
        assert!(m.is_promotion());
        assert!(m.is_capture());
        assert_eq!(m.promotion_piece(), Some(PieceType::Knight));
    }

    #[test]
    fn move_all_promotions() {
        let promotions = [
            (MoveFlags::KNIGHT_PROMOTION, PieceType::Knight),
            (MoveFlags::BISHOP_PROMOTION, PieceType::Bishop),
            (MoveFlags::ROOK_PROMOTION, PieceType::Rook),
            (MoveFlags::QUEEN_PROMOTION, PieceType::Queen),
        ];

        for (flag, expected_piece) in promotions {
            let m = Move::new(Square::A7, Square::A8, flag);
            assert_eq!(m.promotion_piece(), Some(expected_piece));
        }
    }

    #[test]
    fn move_null() {
        let m = Move::null();
        assert!(m.is_null());
        assert_eq!(m.from(), Square::A1);
        assert_eq!(m.to(), Square::A1);
    }

    #[test]
    fn move_to_uci() {
        assert_eq!(
            Move::new(Square::E2, Square::E4, MoveFlags::QUIET).to_uci(),
            "e2e4"
        );
        assert_eq!(
            Move::new(Square::E7, Square::E8, MoveFlags::QUEEN_PROMOTION).to_uci(),
            "e7e8q"
        );
        assert_eq!(
            Move::new(Square::G7, Square::H8, MoveFlags::KNIGHT_PROMOTION_CAPTURE).to_uci(),
            "g7h8n"
        );
    }

    #[test]
    fn move_size() {
        // Ensure Move is exactly 2 bytes
        assert_eq!(std::mem::size_of::<Move>(), 2);
    }
}
