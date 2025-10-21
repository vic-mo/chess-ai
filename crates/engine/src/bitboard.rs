use crate::square::Square;

/// A bitboard representing a set of squares on the chessboard
///
/// Each bit represents a square (0 = empty, 1 = occupied)
/// Bit 0 = a1, Bit 1 = b1, ..., Bit 63 = h8
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, Hash)]
pub struct Bitboard(pub u64);

impl Bitboard {
    /// Empty bitboard
    pub const EMPTY: Bitboard = Bitboard(0);

    /// Full bitboard (all squares set)
    pub const FULL: Bitboard = Bitboard(u64::MAX);

    /// Create a bitboard from a u64
    #[inline]
    pub const fn new(value: u64) -> Self {
        Bitboard(value)
    }

    /// Create a bitboard with a single square set
    #[inline]
    pub const fn from_square(square: Square) -> Self {
        Bitboard(1u64 << square.index())
    }

    /// Check if a square is set
    #[inline]
    pub const fn contains(self, square: Square) -> bool {
        (self.0 & (1u64 << square.index())) != 0
    }

    /// Set a square
    #[inline]
    pub const fn set(self, square: Square) -> Self {
        Bitboard(self.0 | (1u64 << square.index()))
    }

    /// Clear a square
    #[inline]
    pub const fn clear(self, square: Square) -> Self {
        Bitboard(self.0 & !(1u64 << square.index()))
    }

    /// Toggle a square
    #[inline]
    pub const fn toggle(self, square: Square) -> Self {
        Bitboard(self.0 ^ (1u64 << square.index()))
    }

    /// Check if the bitboard is empty
    #[inline]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Check if the bitboard is not empty
    #[inline]
    pub const fn is_not_empty(self) -> bool {
        self.0 != 0
    }

    /// Count the number of set bits (population count)
    #[inline]
    pub const fn count(self) -> u32 {
        self.0.count_ones()
    }

    /// Get the index of the least significant bit (LSB)
    /// Returns None if the bitboard is empty
    #[inline]
    pub const fn lsb(self) -> Option<Square> {
        if self.0 == 0 {
            None
        } else {
            Some(Square::new(self.0.trailing_zeros() as u8))
        }
    }

    /// Get the index of the most significant bit (MSB)
    /// Returns None if the bitboard is empty
    #[inline]
    pub const fn msb(self) -> Option<Square> {
        if self.0 == 0 {
            None
        } else {
            Some(Square::new(63 - self.0.leading_zeros() as u8))
        }
    }

    /// Pop the least significant bit and return it
    #[inline]
    pub fn pop_lsb(&mut self) -> Option<Square> {
        let sq = self.lsb()?;
        self.0 &= self.0 - 1; // Clear LSB
        Some(sq)
    }

    /// Iterate over all set squares
    pub fn iter(self) -> BitboardIterator {
        BitboardIterator { bb: self }
    }

    /// Shift the bitboard north (towards rank 8)
    #[inline]
    pub const fn shift_north(self) -> Self {
        Bitboard(self.0 << 8)
    }

    /// Shift the bitboard south (towards rank 1)
    #[inline]
    pub const fn shift_south(self) -> Self {
        Bitboard(self.0 >> 8)
    }

    /// Shift the bitboard east (towards h-file)
    #[inline]
    pub const fn shift_east(self) -> Self {
        Bitboard((self.0 << 1) & !FILE_A)
    }

    /// Shift the bitboard west (towards a-file)
    #[inline]
    pub const fn shift_west(self) -> Self {
        Bitboard((self.0 >> 1) & !FILE_H)
    }

    /// Shift the bitboard north-east
    #[inline]
    pub const fn shift_north_east(self) -> Self {
        Bitboard((self.0 << 9) & !FILE_A)
    }

    /// Shift the bitboard north-west
    #[inline]
    pub const fn shift_north_west(self) -> Self {
        Bitboard((self.0 << 7) & !FILE_H)
    }

    /// Shift the bitboard south-east
    #[inline]
    pub const fn shift_south_east(self) -> Self {
        Bitboard((self.0 >> 7) & !FILE_A)
    }

    /// Shift the bitboard south-west
    #[inline]
    pub const fn shift_south_west(self) -> Self {
        Bitboard((self.0 >> 9) & !FILE_H)
    }
}

// Bitwise operations
impl std::ops::BitAnd for Bitboard {
    type Output = Self;
    #[inline]
    fn bitand(self, rhs: Self) -> Self {
        Bitboard(self.0 & rhs.0)
    }
}

impl std::ops::BitOr for Bitboard {
    type Output = Self;
    #[inline]
    fn bitor(self, rhs: Self) -> Self {
        Bitboard(self.0 | rhs.0)
    }
}

impl std::ops::BitXor for Bitboard {
    type Output = Self;
    #[inline]
    fn bitxor(self, rhs: Self) -> Self {
        Bitboard(self.0 ^ rhs.0)
    }
}

impl std::ops::Not for Bitboard {
    type Output = Self;
    #[inline]
    fn not(self) -> Self {
        Bitboard(!self.0)
    }
}

impl std::ops::BitAndAssign for Bitboard {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl std::ops::BitOrAssign for Bitboard {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl std::ops::BitXorAssign for Bitboard {
    #[inline]
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}

/// Iterator over set bits in a bitboard
pub struct BitboardIterator {
    bb: Bitboard,
}

impl Iterator for BitboardIterator {
    type Item = Square;

    fn next(&mut self) -> Option<Self::Item> {
        self.bb.pop_lsb()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let count = self.bb.count() as usize;
        (count, Some(count))
    }
}

impl ExactSizeIterator for BitboardIterator {}

impl IntoIterator for Bitboard {
    type Item = Square;
    type IntoIter = BitboardIterator;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

// File and rank constants
const FILE_A: u64 = 0x0101010101010101;
const FILE_H: u64 = 0x8080808080808080;

#[allow(dead_code)]
const RANK_1: u64 = 0x00000000000000FF;
#[allow(dead_code)]
const RANK_8: u64 = 0xFF00000000000000;

impl std::fmt::Display for Bitboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f)?;
        for rank in (0..8).rev() {
            for file in 0..8 {
                let sq = Square::from_coords(file, rank);
                let c = if self.contains(sq) { '1' } else { '.' };
                write!(f, "{} ", c)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bitboard_empty() {
        let bb = Bitboard::EMPTY;
        assert!(bb.is_empty());
        assert_eq!(bb.count(), 0);
    }

    #[test]
    fn bitboard_from_square() {
        let bb = Bitboard::from_square(Square::E4);
        assert!(!bb.is_empty());
        assert_eq!(bb.count(), 1);
        assert!(bb.contains(Square::E4));
        assert!(!bb.contains(Square::E5));
    }

    #[test]
    fn bitboard_set_clear() {
        let mut bb = Bitboard::EMPTY;
        bb = bb.set(Square::E4);
        assert!(bb.contains(Square::E4));
        assert_eq!(bb.count(), 1);

        bb = bb.set(Square::E5);
        assert!(bb.contains(Square::E5));
        assert_eq!(bb.count(), 2);

        bb = bb.clear(Square::E4);
        assert!(!bb.contains(Square::E4));
        assert!(bb.contains(Square::E5));
        assert_eq!(bb.count(), 1);
    }

    #[test]
    fn bitboard_toggle() {
        let mut bb = Bitboard::EMPTY;
        bb = bb.toggle(Square::E4);
        assert!(bb.contains(Square::E4));

        bb = bb.toggle(Square::E4);
        assert!(!bb.contains(Square::E4));
    }

    #[test]
    fn bitboard_lsb_msb() {
        let bb = Bitboard::from_square(Square::A1)
            .set(Square::H8)
            .set(Square::E4);

        assert_eq!(bb.lsb(), Some(Square::A1));
        assert_eq!(bb.msb(), Some(Square::H8));

        assert_eq!(Bitboard::EMPTY.lsb(), None);
        assert_eq!(Bitboard::EMPTY.msb(), None);
    }

    #[test]
    fn bitboard_pop_lsb() {
        let mut bb = Bitboard::from_square(Square::A1)
            .set(Square::E4)
            .set(Square::H8);

        assert_eq!(bb.pop_lsb(), Some(Square::A1));
        assert_eq!(bb.count(), 2);
        assert_eq!(bb.pop_lsb(), Some(Square::E4));
        assert_eq!(bb.count(), 1);
        assert_eq!(bb.pop_lsb(), Some(Square::H8));
        assert_eq!(bb.count(), 0);
        assert_eq!(bb.pop_lsb(), None);
    }

    #[test]
    fn bitboard_iterator() {
        let bb = Bitboard::from_square(Square::A1)
            .set(Square::E4)
            .set(Square::H8);

        let squares: Vec<Square> = bb.into_iter().collect();
        assert_eq!(squares.len(), 3);
        assert_eq!(squares[0], Square::A1);
        assert_eq!(squares[1], Square::E4);
        assert_eq!(squares[2], Square::H8);
    }

    #[test]
    fn bitboard_bitwise_ops() {
        let bb1 = Bitboard::from_square(Square::E4).set(Square::E5);
        let bb2 = Bitboard::from_square(Square::E5).set(Square::D4);

        let and = bb1 & bb2;
        assert_eq!(and.count(), 1);
        assert!(and.contains(Square::E5));

        let or = bb1 | bb2;
        assert_eq!(or.count(), 3);
        assert!(or.contains(Square::E4));
        assert!(or.contains(Square::E5));
        assert!(or.contains(Square::D4));

        let xor = bb1 ^ bb2;
        assert_eq!(xor.count(), 2);
        assert!(xor.contains(Square::E4));
        assert!(xor.contains(Square::D4));
        assert!(!xor.contains(Square::E5));
    }

    #[test]
    fn bitboard_shifts() {
        let bb = Bitboard::from_square(Square::E4);

        let north = bb.shift_north();
        assert!(north.contains(Square::E5));

        let south = bb.shift_south();
        assert!(south.contains(Square::from_coords(4, 2)));

        let east = bb.shift_east();
        assert!(east.contains(Square::from_coords(5, 3)));

        let west = bb.shift_west();
        assert!(west.contains(Square::from_coords(3, 3)));
    }
}
