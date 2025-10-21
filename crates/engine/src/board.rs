use crate::bitboard::Bitboard;
use crate::piece::{Color, Piece, PieceType};
use crate::square::Square;

/// Castling rights for both colors
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct CastlingRights {
    bits: u8,
}

impl CastlingRights {
    const WHITE_KINGSIDE: u8 = 0b0001;
    const WHITE_QUEENSIDE: u8 = 0b0010;
    const BLACK_KINGSIDE: u8 = 0b0100;
    const BLACK_QUEENSIDE: u8 = 0b1000;

    /// Create empty castling rights (no castling allowed)
    pub const fn none() -> Self {
        CastlingRights { bits: 0 }
    }

    /// Create all castling rights (all castles allowed)
    pub const fn all() -> Self {
        CastlingRights { bits: 0b1111 }
    }

    /// Check if white can castle kingside
    #[inline]
    pub const fn white_kingside(self) -> bool {
        (self.bits & Self::WHITE_KINGSIDE) != 0
    }

    /// Check if white can castle queenside
    #[inline]
    pub const fn white_queenside(self) -> bool {
        (self.bits & Self::WHITE_QUEENSIDE) != 0
    }

    /// Check if black can castle kingside
    #[inline]
    pub const fn black_kingside(self) -> bool {
        (self.bits & Self::BLACK_KINGSIDE) != 0
    }

    /// Check if black can castle queenside
    #[inline]
    pub const fn black_queenside(self) -> bool {
        (self.bits & Self::BLACK_QUEENSIDE) != 0
    }

    /// Set white kingside castling
    #[inline]
    pub const fn set_white_kingside(mut self) -> Self {
        self.bits |= Self::WHITE_KINGSIDE;
        self
    }

    /// Set white queenside castling
    #[inline]
    pub const fn set_white_queenside(mut self) -> Self {
        self.bits |= Self::WHITE_QUEENSIDE;
        self
    }

    /// Set black kingside castling
    #[inline]
    pub const fn set_black_kingside(mut self) -> Self {
        self.bits |= Self::BLACK_KINGSIDE;
        self
    }

    /// Set black queenside castling
    #[inline]
    pub const fn set_black_queenside(mut self) -> Self {
        self.bits |= Self::BLACK_QUEENSIDE;
        self
    }

    /// Remove white kingside castling
    #[inline]
    pub const fn remove_white_kingside(mut self) -> Self {
        self.bits &= !Self::WHITE_KINGSIDE;
        self
    }

    /// Remove white queenside castling
    #[inline]
    pub const fn remove_white_queenside(mut self) -> Self {
        self.bits &= !Self::WHITE_QUEENSIDE;
        self
    }

    /// Remove black kingside castling
    #[inline]
    pub const fn remove_black_kingside(mut self) -> Self {
        self.bits &= !Self::BLACK_KINGSIDE;
        self
    }

    /// Remove black queenside castling
    #[inline]
    pub const fn remove_black_queenside(mut self) -> Self {
        self.bits &= !Self::BLACK_QUEENSIDE;
        self
    }

    /// Get raw bits
    #[inline]
    pub const fn bits(self) -> u8 {
        self.bits
    }

    /// Create from raw bits
    #[inline]
    pub const fn from_bits(bits: u8) -> Self {
        CastlingRights {
            bits: bits & 0b1111,
        }
    }
}

impl Default for CastlingRights {
    fn default() -> Self {
        Self::all()
    }
}

/// Chess board represented with bitboards
#[derive(Clone, PartialEq, Eq)]
pub struct Board {
    /// Bitboards for each piece type and color
    /// Index: [color][piece_type]
    pieces: [[Bitboard; 6]; 2],

    /// Occupied squares by color
    occupied_by_color: [Bitboard; 2],

    /// All occupied squares
    occupied: Bitboard,

    /// Side to move
    side_to_move: Color,

    /// Castling rights
    castling: CastlingRights,

    /// En passant target square (if any)
    ep_square: Option<Square>,

    /// Halfmove clock for fifty-move rule
    halfmove_clock: u32,

    /// Fullmove number (starts at 1, increments after black's move)
    fullmove_number: u32,
}

impl Board {
    /// Create an empty board
    pub fn empty() -> Self {
        Board {
            pieces: [[Bitboard::EMPTY; 6]; 2],
            occupied_by_color: [Bitboard::EMPTY; 2],
            occupied: Bitboard::EMPTY,
            side_to_move: Color::White,
            castling: CastlingRights::none(),
            ep_square: None,
            halfmove_clock: 0,
            fullmove_number: 1,
        }
    }

    /// Create the standard chess starting position
    pub fn startpos() -> Self {
        let mut board = Self::empty();

        // White pieces
        board.set_piece(Square::A1, Piece::new(PieceType::Rook, Color::White));
        board.set_piece(Square::B1, Piece::new(PieceType::Knight, Color::White));
        board.set_piece(Square::C1, Piece::new(PieceType::Bishop, Color::White));
        board.set_piece(Square::D1, Piece::new(PieceType::Queen, Color::White));
        board.set_piece(Square::E1, Piece::new(PieceType::King, Color::White));
        board.set_piece(Square::F1, Piece::new(PieceType::Bishop, Color::White));
        board.set_piece(Square::G1, Piece::new(PieceType::Knight, Color::White));
        board.set_piece(Square::H1, Piece::new(PieceType::Rook, Color::White));

        for file in 0..8 {
            board.set_piece(
                Square::from_coords(file, 1),
                Piece::new(PieceType::Pawn, Color::White),
            );
        }

        // Black pieces
        board.set_piece(Square::A8, Piece::new(PieceType::Rook, Color::Black));
        board.set_piece(Square::B8, Piece::new(PieceType::Knight, Color::Black));
        board.set_piece(Square::C8, Piece::new(PieceType::Bishop, Color::Black));
        board.set_piece(Square::D8, Piece::new(PieceType::Queen, Color::Black));
        board.set_piece(Square::E8, Piece::new(PieceType::King, Color::Black));
        board.set_piece(Square::F8, Piece::new(PieceType::Bishop, Color::Black));
        board.set_piece(Square::G8, Piece::new(PieceType::Knight, Color::Black));
        board.set_piece(Square::H8, Piece::new(PieceType::Rook, Color::Black));

        for file in 0..8 {
            board.set_piece(
                Square::from_coords(file, 6),
                Piece::new(PieceType::Pawn, Color::Black),
            );
        }

        board.side_to_move = Color::White;
        board.castling = CastlingRights::all();
        board.ep_square = None;
        board.halfmove_clock = 0;
        board.fullmove_number = 1;

        board
    }

    /// Get the piece at a square, if any
    #[inline]
    pub fn piece_at(&self, square: Square) -> Option<Piece> {
        for color in [Color::White, Color::Black] {
            for piece_type in PieceType::all() {
                if self.pieces[color.index()][piece_type.index()].contains(square) {
                    return Some(Piece::new(piece_type, color));
                }
            }
        }
        None
    }

    /// Set a piece at a square (overwrites existing piece)
    pub fn set_piece(&mut self, square: Square, piece: Piece) {
        // Remove any existing piece at this square
        self.remove_piece(square);

        // Add the new piece
        self.pieces[piece.color.index()][piece.piece_type.index()] =
            self.pieces[piece.color.index()][piece.piece_type.index()].set(square);

        // Update occupied bitboards
        self.update_occupied();
    }

    /// Remove a piece from a square
    pub fn remove_piece(&mut self, square: Square) {
        for color in [Color::White, Color::Black] {
            for piece_type in PieceType::all() {
                self.pieces[color.index()][piece_type.index()] =
                    self.pieces[color.index()][piece_type.index()].clear(square);
            }
        }

        // Update occupied bitboards
        self.update_occupied();
    }

    /// Update the occupied bitboards based on piece positions
    fn update_occupied(&mut self) {
        self.occupied_by_color[Color::White.index()] = Bitboard::EMPTY;
        self.occupied_by_color[Color::Black.index()] = Bitboard::EMPTY;

        for piece_type in PieceType::all() {
            self.occupied_by_color[Color::White.index()] = self.occupied_by_color
                [Color::White.index()]
                | self.pieces[Color::White.index()][piece_type.index()];
            self.occupied_by_color[Color::Black.index()] = self.occupied_by_color
                [Color::Black.index()]
                | self.pieces[Color::Black.index()][piece_type.index()];
        }

        self.occupied = self.occupied_by_color[Color::White.index()]
            | self.occupied_by_color[Color::Black.index()];
    }

    /// Get bitboard for a specific piece type and color
    #[inline]
    pub fn piece_bb(&self, piece_type: PieceType, color: Color) -> Bitboard {
        self.pieces[color.index()][piece_type.index()]
    }

    /// Get bitboard for all pieces of a color
    #[inline]
    pub fn color_bb(&self, color: Color) -> Bitboard {
        self.occupied_by_color[color.index()]
    }

    /// Get bitboard for all occupied squares
    #[inline]
    pub fn occupied(&self) -> Bitboard {
        self.occupied
    }

    /// Get bitboard for all empty squares
    #[inline]
    pub fn empty_squares(&self) -> Bitboard {
        !self.occupied
    }

    /// Get the side to move
    #[inline]
    pub fn side_to_move(&self) -> Color {
        self.side_to_move
    }

    /// Set the side to move
    #[inline]
    pub fn set_side_to_move(&mut self, color: Color) {
        self.side_to_move = color;
    }

    /// Get castling rights
    #[inline]
    pub fn castling(&self) -> CastlingRights {
        self.castling
    }

    /// Set castling rights
    #[inline]
    pub fn set_castling(&mut self, castling: CastlingRights) {
        self.castling = castling;
    }

    /// Get en passant square
    #[inline]
    pub fn ep_square(&self) -> Option<Square> {
        self.ep_square
    }

    /// Set en passant square
    #[inline]
    pub fn set_ep_square(&mut self, square: Option<Square>) {
        self.ep_square = square;
    }

    /// Get halfmove clock
    #[inline]
    pub fn halfmove_clock(&self) -> u32 {
        self.halfmove_clock
    }

    /// Set halfmove clock
    #[inline]
    pub fn set_halfmove_clock(&mut self, clock: u32) {
        self.halfmove_clock = clock;
    }

    /// Get fullmove number
    #[inline]
    pub fn fullmove_number(&self) -> u32 {
        self.fullmove_number
    }

    /// Set fullmove number
    #[inline]
    pub fn set_fullmove_number(&mut self, number: u32) {
        self.fullmove_number = number;
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::startpos()
    }
}

impl std::fmt::Debug for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f)?;
        for rank in (0..8).rev() {
            write!(f, "{} ", rank + 1)?;
            for file in 0..8 {
                let sq = Square::from_coords(file, rank);
                let c = match self.piece_at(sq) {
                    Some(piece) => piece.to_char(),
                    None => '.',
                };
                write!(f, "{} ", c)?;
            }
            writeln!(f)?;
        }
        writeln!(f, "  a b c d e f g h")?;
        writeln!(f, "Side to move: {}", self.side_to_move)?;
        writeln!(f, "Castling: {}", format_castling(self.castling))?;
        writeln!(f, "EP square: {:?}", self.ep_square)?;
        writeln!(f, "Halfmove: {}", self.halfmove_clock)?;
        writeln!(f, "Fullmove: {}", self.fullmove_number)?;
        Ok(())
    }
}

fn format_castling(castling: CastlingRights) -> String {
    if castling.bits() == 0 {
        return "-".to_string();
    }
    let mut s = String::new();
    if castling.white_kingside() {
        s.push('K');
    }
    if castling.white_queenside() {
        s.push('Q');
    }
    if castling.black_kingside() {
        s.push('k');
    }
    if castling.black_queenside() {
        s.push('q');
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn castling_rights_all() {
        let rights = CastlingRights::all();
        assert!(rights.white_kingside());
        assert!(rights.white_queenside());
        assert!(rights.black_kingside());
        assert!(rights.black_queenside());
    }

    #[test]
    fn castling_rights_none() {
        let rights = CastlingRights::none();
        assert!(!rights.white_kingside());
        assert!(!rights.white_queenside());
        assert!(!rights.black_kingside());
        assert!(!rights.black_queenside());
    }

    #[test]
    fn castling_rights_partial() {
        let rights = CastlingRights::none()
            .set_white_kingside()
            .set_black_queenside();
        assert!(rights.white_kingside());
        assert!(!rights.white_queenside());
        assert!(!rights.black_kingside());
        assert!(rights.black_queenside());
    }

    #[test]
    fn empty_board() {
        let board = Board::empty();
        assert!(board.occupied().is_empty());
        assert_eq!(board.side_to_move(), Color::White);
        assert_eq!(board.piece_at(Square::E4), None);
    }

    #[test]
    fn startpos_board() {
        let board = Board::startpos();

        // Check white pieces
        assert_eq!(
            board.piece_at(Square::E1),
            Some(Piece::new(PieceType::King, Color::White))
        );
        assert_eq!(
            board.piece_at(Square::E2),
            Some(Piece::new(PieceType::Pawn, Color::White))
        );

        // Check black pieces
        assert_eq!(
            board.piece_at(Square::E8),
            Some(Piece::new(PieceType::King, Color::Black))
        );
        assert_eq!(
            board.piece_at(Square::E7),
            Some(Piece::new(PieceType::Pawn, Color::Black))
        );

        // Check empty squares
        assert_eq!(board.piece_at(Square::E4), None);
        assert_eq!(board.piece_at(Square::E5), None);

        // Check occupied count
        assert_eq!(board.occupied().count(), 32);
    }

    #[test]
    fn set_and_remove_piece() {
        let mut board = Board::empty();

        board.set_piece(Square::E4, Piece::new(PieceType::Pawn, Color::White));
        assert_eq!(
            board.piece_at(Square::E4),
            Some(Piece::new(PieceType::Pawn, Color::White))
        );
        assert_eq!(board.occupied().count(), 1);

        board.remove_piece(Square::E4);
        assert_eq!(board.piece_at(Square::E4), None);
        assert!(board.occupied().is_empty());
    }

    #[test]
    fn piece_bitboards() {
        let board = Board::startpos();

        // White pawns
        let white_pawns = board.piece_bb(PieceType::Pawn, Color::White);
        assert_eq!(white_pawns.count(), 8);

        // Black knights
        let black_knights = board.piece_bb(PieceType::Knight, Color::Black);
        assert_eq!(black_knights.count(), 2);
        assert!(black_knights.contains(Square::B8));
        assert!(black_knights.contains(Square::G8));
    }

    #[test]
    fn color_bitboards() {
        let board = Board::startpos();

        let white_pieces = board.color_bb(Color::White);
        let black_pieces = board.color_bb(Color::Black);

        assert_eq!(white_pieces.count(), 16);
        assert_eq!(black_pieces.count(), 16);
        assert_eq!((white_pieces | black_pieces).count(), 32);
    }

    #[test]
    fn board_state() {
        let mut board = Board::startpos();

        assert_eq!(board.side_to_move(), Color::White);
        assert_eq!(board.halfmove_clock(), 0);
        assert_eq!(board.fullmove_number(), 1);
        assert_eq!(board.ep_square(), None);

        board.set_side_to_move(Color::Black);
        board.set_halfmove_clock(5);
        board.set_fullmove_number(10);
        board.set_ep_square(Some(Square::E3));

        assert_eq!(board.side_to_move(), Color::Black);
        assert_eq!(board.halfmove_clock(), 5);
        assert_eq!(board.fullmove_number(), 10);
        assert_eq!(board.ep_square(), Some(Square::E3));
    }
}
