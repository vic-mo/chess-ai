/// Represents a square on the chessboard (0-63)
///
/// Layout (little-endian rank-file mapping):
/// ```text
/// 56 57 58 59 60 61 62 63  (rank 8)
/// 48 49 50 51 52 53 54 55  (rank 7)
/// ...
///  8  9 10 11 12 13 14 15  (rank 2)
///  0  1  2  3  4  5  6  7  (rank 1)
///  a  b  c  d  e  f  g  h
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Square(u8);

impl Square {
    /// Create a square from an index (0-63)
    #[inline]
    pub const fn new(index: u8) -> Self {
        debug_assert!(index < 64, "Square index must be 0-63");
        Square(index)
    }

    /// Create a square from file (0-7) and rank (0-7)
    #[inline]
    pub const fn from_coords(file: u8, rank: u8) -> Self {
        debug_assert!(file < 8, "File must be 0-7");
        debug_assert!(rank < 8, "Rank must be 0-7");
        Square(rank * 8 + file)
    }

    /// Get the index (0-63)
    #[inline]
    pub const fn index(self) -> u8 {
        self.0
    }

    /// Get the file (0-7, where 0=a, 7=h)
    #[inline]
    pub const fn file(self) -> u8 {
        self.0 & 7
    }

    /// Get the rank (0-7, where 0=rank 1, 7=rank 8)
    #[inline]
    pub const fn rank(self) -> u8 {
        self.0 >> 3
    }

    /// Convert to algebraic notation (e.g., "e4")
    pub fn to_algebraic(self) -> String {
        let file_char = (b'a' + self.file()) as char;
        let rank_char = (b'1' + self.rank()) as char;
        format!("{}{}", file_char, rank_char)
    }

    /// Parse from algebraic notation (e.g., "e4")
    pub fn from_algebraic(s: &str) -> Option<Self> {
        let bytes = s.as_bytes();
        if bytes.len() != 2 {
            return None;
        }

        let file = bytes[0].checked_sub(b'a')?;
        let rank = bytes[1].checked_sub(b'1')?;

        if file < 8 && rank < 8 {
            Some(Square::from_coords(file, rank))
        } else {
            None
        }
    }

    /// Get all 64 squares
    pub const fn all() -> [Square; 64] {
        let mut squares = [Square(0); 64];
        let mut i = 0;
        while i < 64 {
            squares[i] = Square(i as u8);
            i += 1;
        }
        squares
    }
}

// Common square constants
#[allow(dead_code)]
impl Square {
    pub const A1: Square = Square(0);
    pub const B1: Square = Square(1);
    pub const C1: Square = Square(2);
    pub const D1: Square = Square(3);
    pub const E1: Square = Square(4);
    pub const F1: Square = Square(5);
    pub const G1: Square = Square(6);
    pub const H1: Square = Square(7);

    pub const A8: Square = Square(56);
    pub const B8: Square = Square(57);
    pub const C8: Square = Square(58);
    pub const D8: Square = Square(59);
    pub const E8: Square = Square(60);
    pub const F8: Square = Square(61);
    pub const G8: Square = Square(62);
    pub const H8: Square = Square(63);

    pub const E4: Square = Square(28);
    pub const E5: Square = Square(36);
    pub const D4: Square = Square(27);
}

impl std::fmt::Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_algebraic())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn square_creation() {
        let sq = Square::new(0);
        assert_eq!(sq.index(), 0);
        assert_eq!(sq.file(), 0);
        assert_eq!(sq.rank(), 0);
    }

    #[test]
    fn square_from_coords() {
        let sq = Square::from_coords(4, 3); // e4
        assert_eq!(sq.index(), 28);
        assert_eq!(sq.file(), 4);
        assert_eq!(sq.rank(), 3);
    }

    #[test]
    fn square_algebraic() {
        let sq = Square::from_coords(4, 3); // e4
        assert_eq!(sq.to_algebraic(), "e4");

        assert_eq!(Square::A1.to_algebraic(), "a1");
        assert_eq!(Square::H8.to_algebraic(), "h8");
    }

    #[test]
    fn square_from_algebraic() {
        assert_eq!(
            Square::from_algebraic("e4"),
            Some(Square::from_coords(4, 3))
        );
        assert_eq!(Square::from_algebraic("a1"), Some(Square::A1));
        assert_eq!(Square::from_algebraic("h8"), Some(Square::H8));

        assert_eq!(Square::from_algebraic("i1"), None);
        assert_eq!(Square::from_algebraic("a9"), None);
        assert_eq!(Square::from_algebraic("e"), None);
    }

    #[test]
    fn square_constants() {
        assert_eq!(Square::E1.index(), 4);
        assert_eq!(Square::E4.index(), 28);
        assert_eq!(Square::A1.index(), 0);
        assert_eq!(Square::H8.index(), 63);
    }

    #[test]
    fn square_all() {
        let all_squares = Square::all();
        assert_eq!(all_squares.len(), 64);
        assert_eq!(all_squares[0], Square::A1);
        assert_eq!(all_squares[63], Square::H8);
    }
}
