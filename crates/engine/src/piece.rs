/// Color of a chess piece
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Color {
    White = 0,
    Black = 1,
}

impl Color {
    /// Get the opposite color
    #[inline]
    pub const fn opponent(self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }

    /// Convert to index (0 or 1)
    #[inline]
    pub const fn index(self) -> usize {
        self as usize
    }

    /// Convert from index
    #[inline]
    pub const fn from_index(index: usize) -> Self {
        match index {
            0 => Color::White,
            1 => Color::Black,
            _ => panic!("Invalid color index"),
        }
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Color::White => write!(f, "white"),
            Color::Black => write!(f, "black"),
        }
    }
}

/// Type of chess piece
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum PieceType {
    Pawn = 0,
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
    King = 5,
}

impl PieceType {
    /// Get all piece types
    pub const fn all() -> [PieceType; 6] {
        [
            PieceType::Pawn,
            PieceType::Knight,
            PieceType::Bishop,
            PieceType::Rook,
            PieceType::Queen,
            PieceType::King,
        ]
    }

    /// Convert to index (0-5)
    #[inline]
    pub const fn index(self) -> usize {
        self as usize
    }

    /// Convert from index
    #[inline]
    pub const fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(PieceType::Pawn),
            1 => Some(PieceType::Knight),
            2 => Some(PieceType::Bishop),
            3 => Some(PieceType::Rook),
            4 => Some(PieceType::Queen),
            5 => Some(PieceType::King),
            _ => None,
        }
    }

    /// Get FEN character for this piece type (lowercase)
    pub const fn to_char(self) -> char {
        match self {
            PieceType::Pawn => 'p',
            PieceType::Knight => 'n',
            PieceType::Bishop => 'b',
            PieceType::Rook => 'r',
            PieceType::Queen => 'q',
            PieceType::King => 'k',
        }
    }

    /// Parse from FEN character (case insensitive)
    pub fn from_char(c: char) -> Option<Self> {
        match c.to_ascii_lowercase() {
            'p' => Some(PieceType::Pawn),
            'n' => Some(PieceType::Knight),
            'b' => Some(PieceType::Bishop),
            'r' => Some(PieceType::Rook),
            'q' => Some(PieceType::Queen),
            'k' => Some(PieceType::King),
            _ => None,
        }
    }
}

impl std::fmt::Display for PieceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_char())
    }
}

/// A colored chess piece
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Piece {
    pub piece_type: PieceType,
    pub color: Color,
}

impl Piece {
    /// Create a new piece
    #[inline]
    pub const fn new(piece_type: PieceType, color: Color) -> Self {
        Piece { piece_type, color }
    }

    /// Get FEN character for this piece (uppercase for white, lowercase for black)
    pub fn to_char(self) -> char {
        let c = self.piece_type.to_char();
        match self.color {
            Color::White => c.to_ascii_uppercase(),
            Color::Black => c,
        }
    }

    /// Parse from FEN character
    pub fn from_char(c: char) -> Option<Self> {
        let piece_type = PieceType::from_char(c)?;
        let color = if c.is_ascii_uppercase() {
            Color::White
        } else {
            Color::Black
        };
        Some(Piece::new(piece_type, color))
    }
}

impl std::fmt::Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_char())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_opponent() {
        assert_eq!(Color::White.opponent(), Color::Black);
        assert_eq!(Color::Black.opponent(), Color::White);
    }

    #[test]
    fn color_index() {
        assert_eq!(Color::White.index(), 0);
        assert_eq!(Color::Black.index(), 1);
        assert_eq!(Color::from_index(0), Color::White);
        assert_eq!(Color::from_index(1), Color::Black);
    }

    #[test]
    fn piece_type_index() {
        assert_eq!(PieceType::Pawn.index(), 0);
        assert_eq!(PieceType::King.index(), 5);
        assert_eq!(PieceType::from_index(0), Some(PieceType::Pawn));
        assert_eq!(PieceType::from_index(5), Some(PieceType::King));
        assert_eq!(PieceType::from_index(6), None);
    }

    #[test]
    fn piece_type_char() {
        assert_eq!(PieceType::Pawn.to_char(), 'p');
        assert_eq!(PieceType::Knight.to_char(), 'n');
        assert_eq!(PieceType::King.to_char(), 'k');

        assert_eq!(PieceType::from_char('p'), Some(PieceType::Pawn));
        assert_eq!(PieceType::from_char('P'), Some(PieceType::Pawn));
        assert_eq!(PieceType::from_char('n'), Some(PieceType::Knight));
        assert_eq!(PieceType::from_char('x'), None);
    }

    #[test]
    fn piece_char() {
        let white_pawn = Piece::new(PieceType::Pawn, Color::White);
        let black_king = Piece::new(PieceType::King, Color::Black);

        assert_eq!(white_pawn.to_char(), 'P');
        assert_eq!(black_king.to_char(), 'k');

        assert_eq!(Piece::from_char('P'), Some(white_pawn));
        assert_eq!(Piece::from_char('k'), Some(black_king));
        assert_eq!(Piece::from_char('x'), None);
    }

    #[test]
    fn piece_all_types() {
        let all = PieceType::all();
        assert_eq!(all.len(), 6);
        assert_eq!(all[0], PieceType::Pawn);
        assert_eq!(all[5], PieceType::King);
    }
}
