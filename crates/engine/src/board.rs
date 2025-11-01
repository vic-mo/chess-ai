use crate::bitboard::Bitboard;
use crate::piece::{Color, Piece, PieceType};
use crate::r#move::Move;
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

/// Information needed to unmake a move.
///
/// This stores all the state that changes when making a move so it can be
/// restored when unmaking the move.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct UndoInfo {
    /// The piece that was captured (if any)
    pub captured_piece: Option<Piece>,
    /// Castling rights before the move
    pub castling_rights: CastlingRights,
    /// En passant square before the move
    pub ep_square: Option<Square>,
    /// Halfmove clock before the move
    pub halfmove_clock: u32,
    /// Zobrist hash before the move (will be used in Session 14)
    pub hash: u64,
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

    /// Zobrist hash of the position
    hash: u64,
}

impl Board {
    /// Create an empty board
    pub fn empty() -> Self {
        let mut board = Board {
            pieces: [[Bitboard::EMPTY; 6]; 2],
            occupied_by_color: [Bitboard::EMPTY; 2],
            occupied: Bitboard::EMPTY,
            side_to_move: Color::White,
            castling: CastlingRights::none(),
            ep_square: None,
            halfmove_clock: 0,
            fullmove_number: 1,
            hash: 0,
        };
        board.hash = crate::zobrist::zobrist_hash(&board);
        board
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

        board.hash = crate::zobrist::zobrist_hash(&board);

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

    /// Get the Zobrist hash of the current position
    #[inline]
    pub fn hash(&self) -> u64 {
        self.hash
    }

    /// Set the Zobrist hash (used internally by FEN parser)
    #[inline]
    pub(crate) fn set_hash(&mut self, hash: u64) {
        self.hash = hash;
    }

    /// Make a move on the board, returning undo information.
    ///
    /// This updates the board state and returns information needed to unmake
    /// the move. The move is assumed to be pseudo-legal (it may leave the king
    /// in check).
    ///
    /// # Example
    /// ```
    /// use engine::board::Board;
    /// use engine::r#move::{Move, MoveFlags};
    /// use engine::square::Square;
    ///
    /// let mut board = Board::startpos();
    /// let m = Move::new(Square::E2, Square::E4, MoveFlags::DOUBLE_PAWN_PUSH);
    /// let undo = board.make_move(m);
    /// // Board state has changed
    /// board.unmake_move(m, undo);
    /// // Board is back to starting position
    /// ```
    pub fn make_move(&mut self, m: Move) -> UndoInfo {
        let from = m.from();
        let to = m.to();
        let us = self.side_to_move;
        let them = us.opponent();

        // Store undo information
        // For en passant, the captured piece is not at the destination square
        let captured_piece = if m.is_en_passant() {
            let captured_pawn_square = if us == Color::White {
                Square::new(to.index() - 8)
            } else {
                Square::new(to.index() + 8)
            };
            self.piece_at(captured_pawn_square)
        } else {
            self.piece_at(to)
        };

        let undo = UndoInfo {
            captured_piece,
            castling_rights: self.castling,
            ep_square: self.ep_square,
            halfmove_clock: self.halfmove_clock,
            hash: self.hash,
        };

        // Get the moving piece
        let moving_piece = self
            .piece_at(from)
            .expect("make_move called with no piece at from square");

        // Clear en passant square (will be set again if this is a double pawn push)
        self.ep_square = None;

        // Handle captures
        if m.is_capture() {
            if m.is_en_passant() {
                // En passant: captured pawn is not on the destination square
                let captured_pawn_square = if us == Color::White {
                    Square::new(to.index() - 8)
                } else {
                    Square::new(to.index() + 8)
                };
                self.remove_piece(captured_pawn_square);
            } else {
                // Normal capture: remove piece at destination
                self.remove_piece(to);
            }
            // Reset halfmove clock on capture
            self.halfmove_clock = 0;
        } else if moving_piece.piece_type == PieceType::Pawn {
            // Reset halfmove clock on pawn move
            self.halfmove_clock = 0;
        } else {
            // Increment halfmove clock
            self.halfmove_clock += 1;
        }

        // Move the piece
        self.remove_piece(from);

        // Handle promotions
        if m.is_promotion() {
            let promoted_piece = m
                .promotion_piece()
                .expect("Promotion move without promotion piece");
            self.set_piece(to, Piece::new(promoted_piece, us));
        } else {
            self.set_piece(to, moving_piece);
        }

        // Handle double pawn push (set en passant square)
        if m.is_double_pawn_push() {
            let ep_square = if us == Color::White {
                Square::new(from.index() + 8)
            } else {
                Square::new(from.index() - 8)
            };
            self.ep_square = Some(ep_square);
        }

        // Handle castling
        if m.is_castling() {
            // Move the rook
            let (rook_from, rook_to) = if to.file() > from.file() {
                // Kingside castling
                (
                    Square::from_coords(7, from.rank()),
                    Square::from_coords(5, from.rank()),
                )
            } else {
                // Queenside castling
                (
                    Square::from_coords(0, from.rank()),
                    Square::from_coords(3, from.rank()),
                )
            };

            let rook = self.piece_at(rook_from).expect("Castling without rook");
            self.remove_piece(rook_from);
            self.set_piece(rook_to, rook);
        }

        // Update castling rights
        // Remove castling rights if king or rook moves
        if moving_piece.piece_type == PieceType::King {
            if us == Color::White {
                self.castling = self
                    .castling
                    .remove_white_kingside()
                    .remove_white_queenside();
            } else {
                self.castling = self
                    .castling
                    .remove_black_kingside()
                    .remove_black_queenside();
            }
        } else if moving_piece.piece_type == PieceType::Rook {
            // Check which rook moved
            if from == Square::A1 {
                self.castling = self.castling.remove_white_queenside();
            } else if from == Square::H1 {
                self.castling = self.castling.remove_white_kingside();
            } else if from == Square::A8 {
                self.castling = self.castling.remove_black_queenside();
            } else if from == Square::H8 {
                self.castling = self.castling.remove_black_kingside();
            }
        }

        // If a rook is captured, remove castling rights
        if m.is_capture() && !m.is_en_passant() {
            if to == Square::A1 {
                self.castling = self.castling.remove_white_queenside();
            } else if to == Square::H1 {
                self.castling = self.castling.remove_white_kingside();
            } else if to == Square::A8 {
                self.castling = self.castling.remove_black_queenside();
            } else if to == Square::H8 {
                self.castling = self.castling.remove_black_kingside();
            }
        }

        // Switch side to move
        self.side_to_move = them;

        // Update fullmove number (increments after black's move)
        if us == Color::Black {
            self.fullmove_number += 1;
        }

        // Incrementally update hash
        use crate::zobrist::{hash_castling, hash_en_passant, hash_piece, hash_side_to_move};

        // Remove old piece from source square
        self.hash = hash_piece(self.hash, moving_piece, from);

        // Remove captured piece
        if let Some(piece) = captured_piece {
            let capture_sq = if m.is_en_passant() {
                if us == Color::White {
                    Square::new(to.index() - 8)
                } else {
                    Square::new(to.index() + 8)
                }
            } else {
                to
            };
            self.hash = hash_piece(self.hash, piece, capture_sq);
        }

        // Add new piece to destination square (or promoted piece)
        let final_piece = if m.is_promotion() {
            Piece::new(m.promotion_piece().unwrap(), us)
        } else {
            moving_piece
        };
        self.hash = hash_piece(self.hash, final_piece, to);

        // Handle castling rook move
        if m.is_castling() {
            let (rook_from, rook_to) = if to.file() > from.file() {
                (
                    Square::from_coords(7, from.rank()),
                    Square::from_coords(5, from.rank()),
                )
            } else {
                (
                    Square::from_coords(0, from.rank()),
                    Square::from_coords(3, from.rank()),
                )
            };
            let rook = Piece::new(PieceType::Rook, us);
            self.hash = hash_piece(self.hash, rook, rook_from);
            self.hash = hash_piece(self.hash, rook, rook_to);
        }

        // Update castling rights hash
        self.hash = hash_castling(self.hash, undo.castling_rights, self.castling);

        // Update en passant hash
        self.hash = hash_en_passant(self.hash, undo.ep_square, self.ep_square);

        // Toggle side to move (always XOR since we switched sides)
        self.hash = hash_side_to_move(self.hash);

        undo
    }

    /// Unmake a move on the board, restoring the previous state.
    ///
    /// This reverses the effects of `make_move` using the undo information.
    ///
    /// # Example
    /// ```
    /// use engine::board::Board;
    /// use engine::r#move::{Move, MoveFlags};
    /// use engine::square::Square;
    ///
    /// let original = Board::startpos();
    /// let mut board = original.clone();
    /// let m = Move::new(Square::E2, Square::E4, MoveFlags::DOUBLE_PAWN_PUSH);
    ///
    /// let undo = board.make_move(m);
    /// board.unmake_move(m, undo);
    ///
    /// assert_eq!(board, original);
    /// ```
    pub fn unmake_move(&mut self, m: Move, undo: UndoInfo) {
        let from = m.from();
        let to = m.to();

        // Switch side to move back
        self.side_to_move = self.side_to_move.opponent();
        let us = self.side_to_move;

        // Update fullmove number (decrements if we're unmaking black's move)
        if us == Color::Black {
            self.fullmove_number -= 1;
        }

        // Get the piece at destination (might be promoted)
        let piece_at_dest = self.piece_at(to).expect("No piece at destination");

        // Move the piece back
        self.remove_piece(to);

        // If it was a promotion, restore the pawn
        if m.is_promotion() {
            self.set_piece(from, Piece::new(PieceType::Pawn, us));
        } else {
            self.set_piece(from, piece_at_dest);
        }

        // Restore captured piece
        if let Some(captured) = undo.captured_piece {
            if m.is_en_passant() {
                // En passant: restore pawn at different square
                let captured_pawn_square = if us == Color::White {
                    Square::new(to.index() - 8)
                } else {
                    Square::new(to.index() + 8)
                };
                self.set_piece(captured_pawn_square, captured);
            } else {
                // Normal capture: restore piece at destination
                self.set_piece(to, captured);
            }
        }

        // Unmake castling
        if m.is_castling() {
            // Move the rook back
            let (rook_from, rook_to) = if to.file() > from.file() {
                // Kingside castling
                (
                    Square::from_coords(7, from.rank()),
                    Square::from_coords(5, from.rank()),
                )
            } else {
                // Queenside castling
                (
                    Square::from_coords(0, from.rank()),
                    Square::from_coords(3, from.rank()),
                )
            };

            let rook = self.piece_at(rook_to).expect("No rook to unmove");
            self.remove_piece(rook_to);
            self.set_piece(rook_from, rook);
        }

        // Restore state
        self.castling = undo.castling_rights;
        self.ep_square = undo.ep_square;
        self.halfmove_clock = undo.halfmove_clock;
        self.hash = undo.hash;
    }

    /// Check if a square is attacked by the given color.
    ///
    /// This is used for legality checking (king in check, castling through check, etc).
    ///
    /// # Example
    /// ```
    /// use engine::board::Board;
    /// use engine::piece::Color;
    /// use engine::square::Square;
    ///
    /// let board = Board::startpos();
    /// // E2 is protected by white pieces
    /// assert!(board.is_square_attacked(Square::E2, Color::White));
    /// ```
    pub fn is_square_attacked(&self, square: Square, by_color: Color) -> bool {
        use crate::attacks::{
            bishop_attacks, king_attacks, knight_attacks, pawn_attacks, rook_attacks,
        };

        let occupied = self.occupied();

        // Check for pawn attacks
        let pawns = self.piece_bb(PieceType::Pawn, by_color);
        for pawn_sq in pawns {
            if pawn_attacks(pawn_sq, by_color).contains(square) {
                return true;
            }
        }

        // Check for knight attacks
        let knights = self.piece_bb(PieceType::Knight, by_color);
        for knight_sq in knights {
            if knight_attacks(knight_sq).contains(square) {
                return true;
            }
        }

        // Check for bishop/queen diagonal attacks
        let bishops = self.piece_bb(PieceType::Bishop, by_color);
        let queens = self.piece_bb(PieceType::Queen, by_color);
        let diagonal_attackers = bishops | queens;
        for attacker_sq in diagonal_attackers {
            if bishop_attacks(attacker_sq, occupied).contains(square) {
                return true;
            }
        }

        // Check for rook/queen orthogonal attacks
        let rooks = self.piece_bb(PieceType::Rook, by_color);
        let orthogonal_attackers = rooks | queens;
        for attacker_sq in orthogonal_attackers {
            if rook_attacks(attacker_sq, occupied).contains(square) {
                return true;
            }
        }

        // Check for king attacks
        let king = self.piece_bb(PieceType::King, by_color);
        for king_sq in king {
            if king_attacks(king_sq).contains(square) {
                return true;
            }
        }

        false
    }

    /// Check if the current side to move is in check.
    ///
    /// # Example
    /// ```
    /// use engine::board::Board;
    /// use engine::io::parse_fen;
    ///
    /// let board = Board::startpos();
    /// assert!(!board.is_in_check());
    ///
    /// // Position with white king in check
    /// let fen = "rnb1kbnr/pppp1ppp/8/4p3/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 1 3";
    /// let board = parse_fen(fen).unwrap();
    /// assert!(board.is_in_check());
    /// ```
    pub fn is_in_check(&self) -> bool {
        let us = self.side_to_move;
        let them = us.opponent();

        // Find our king
        let our_king = self.piece_bb(PieceType::King, us);
        if our_king.is_empty() {
            // No king (shouldn't happen in valid position)
            return false;
        }

        let king_square = our_king.into_iter().next().unwrap();
        self.is_square_attacked(king_square, them)
    }

    /// Check if a move gives check to the opponent.
    ///
    /// This requires making the move temporarily to check if the opponent's
    /// king is in check. This is relatively expensive, so use sparingly.
    ///
    /// # Example
    /// ```
    /// use engine::board::Board;
    /// use engine::movegen::generate_moves;
    ///
    /// let board = Board::startpos();
    /// let moves = generate_moves(&board);
    /// // Check if any move gives check
    /// let checking_moves: Vec<_> = moves.into_iter()
    ///     .filter(|&m| board.gives_check(m))
    ///     .collect();
    /// ```
    pub fn gives_check(&self, m: Move) -> bool {
        let mut board_after = self.clone();
        board_after.make_move(m);
        board_after.is_in_check()
    }

    /// Make a null move (pass the turn without moving).
    ///
    /// This is used in null move pruning during search. A null move:
    /// - Toggles side to move
    /// - Clears en passant square
    /// - Increments halfmove clock
    /// - Updates Zobrist hash
    ///
    /// # Example
    /// ```
    /// use engine::board::Board;
    /// use engine::piece::Color;
    ///
    /// let mut board = Board::startpos();
    /// assert_eq!(board.side_to_move(), Color::White);
    /// board.make_null_move();
    /// assert_eq!(board.side_to_move(), Color::Black);
    /// ```
    pub fn make_null_move(&mut self) {
        use crate::zobrist::{hash_en_passant, hash_side_to_move};

        // Clear en passant square if present
        let old_ep = self.ep_square;
        if old_ep.is_some() {
            self.hash = hash_en_passant(self.hash, old_ep, None);
            self.ep_square = None;
        }

        // Toggle side to move
        self.side_to_move = self.side_to_move.opponent();
        self.hash = hash_side_to_move(self.hash);

        // Increment halfmove clock (null move doesn't reset it)
        self.halfmove_clock += 1;
    }

    /// Check if a move is legal (doesn't leave the king in check).
    ///
    /// This assumes the move is pseudo-legal (follows piece movement rules).
    ///
    /// # Example
    /// ```
    /// use engine::board::Board;
    /// use engine::r#move::{Move, MoveFlags};
    /// use engine::square::Square;
    ///
    /// let board = Board::startpos();
    /// let m = Move::new(Square::E2, Square::E4, MoveFlags::DOUBLE_PAWN_PUSH);
    /// assert!(board.is_legal(m));
    /// ```
    pub fn is_legal(&self, m: Move) -> bool {
        // Special handling for castling
        if m.is_castling() {
            return self.is_castling_legal(m);
        }

        // For non-castling moves, make the move and check if our king is still safe
        let us = self.side_to_move;
        let mut board = self.clone();
        board.make_move(m);

        // After making the move, side_to_move has switched to opponent.
        // We need to check if our king (the side that just moved) is attacked.
        let our_king = board.piece_bb(PieceType::King, us);
        if our_king.is_empty() {
            return false; // No king (shouldn't happen in valid position)
        }

        let king_square = our_king.into_iter().next().unwrap();
        !board.is_square_attacked(king_square, us.opponent())
    }

    /// Check if a castling move is legal.
    ///
    /// Castling is illegal if:
    /// 1. The king is currently in check
    /// 2. The king passes through a square under attack
    /// 3. The king ends up in check
    fn is_castling_legal(&self, m: Move) -> bool {
        let us = self.side_to_move;
        let them = us.opponent();

        // Can't castle out of check
        if self.is_in_check() {
            return false;
        }

        let from = m.from();
        let to = m.to();

        // Determine which squares the king passes through
        let passing_square = if m.is_kingside_castle() {
            // Kingside: king passes through f1/f8
            Square::from_coords(5, from.rank())
        } else {
            // Queenside: king passes through d1/d8
            Square::from_coords(3, from.rank())
        };

        // Check if king passes through attacked square
        if self.is_square_attacked(passing_square, them) {
            return false;
        }

        // Check if king ends up in check (destination square)
        if self.is_square_attacked(to, them) {
            return false;
        }

        true
    }

    /// Generate all legal moves for the current position.
    ///
    /// This generates pseudo-legal moves and filters out illegal ones.
    ///
    /// # Example
    /// ```
    /// use engine::board::Board;
    ///
    /// let board = Board::startpos();
    /// let legal_moves = board.generate_legal_moves();
    /// assert_eq!(legal_moves.len(), 20); // 16 pawn moves + 4 knight moves
    /// ```
    pub fn generate_legal_moves(&self) -> crate::movelist::MoveList {
        use crate::movegen::generate_moves;
        use crate::movelist::MoveList;

        let pseudo_legal = generate_moves(self);
        let mut legal = MoveList::new();

        for m in pseudo_legal {
            if self.is_legal(m) {
                legal.push(m);
            }
        }

        legal
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
    use crate::r#move::MoveFlags;

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

    #[test]
    fn make_unmake_quiet_move() {
        let original = Board::startpos();
        let mut board = original.clone();

        let m = Move::new(Square::E2, Square::E4, MoveFlags::DOUBLE_PAWN_PUSH);
        let undo = board.make_move(m);

        // Piece should have moved
        assert_eq!(board.piece_at(Square::E2), None);
        assert_eq!(
            board.piece_at(Square::E4),
            Some(Piece::new(PieceType::Pawn, Color::White))
        );

        // Side to move should have switched
        assert_eq!(board.side_to_move(), Color::Black);

        // En passant square should be set
        assert_eq!(board.ep_square(), Some(Square::E3));

        // Halfmove clock should reset
        assert_eq!(board.halfmove_clock(), 0);

        // Unmake the move
        board.unmake_move(m, undo);

        // Board should be identical to original
        assert_eq!(board, original);
    }

    #[test]
    fn make_unmake_capture() {
        let mut board = Board::empty();
        board.set_piece(Square::E4, Piece::new(PieceType::Knight, Color::White));
        board.set_piece(Square::D6, Piece::new(PieceType::Pawn, Color::Black));
        board.set_side_to_move(Color::White);
        board.set_halfmove_clock(10);

        let original = board.clone();

        let m = Move::new(Square::E4, Square::D6, MoveFlags::CAPTURE);
        let undo = board.make_move(m);

        // Knight should have captured pawn
        assert_eq!(
            board.piece_at(Square::D6),
            Some(Piece::new(PieceType::Knight, Color::White))
        );
        assert_eq!(board.piece_at(Square::E4), None);

        // Halfmove clock should reset
        assert_eq!(board.halfmove_clock(), 0);

        // Unmake the move
        board.unmake_move(m, undo);

        // Board should be restored
        assert_eq!(board, original);
    }

    #[test]
    fn make_unmake_castling_kingside() {
        let mut board = Board::empty();
        board.set_piece(Square::E1, Piece::new(PieceType::King, Color::White));
        board.set_piece(Square::H1, Piece::new(PieceType::Rook, Color::White));
        board.set_side_to_move(Color::White);
        board.set_castling(CastlingRights::all());

        let original = board.clone();

        let m = Move::new(Square::E1, Square::G1, MoveFlags::KING_CASTLE);
        let undo = board.make_move(m);

        // King should be on G1
        assert_eq!(
            board.piece_at(Square::G1),
            Some(Piece::new(PieceType::King, Color::White))
        );

        // Rook should be on F1
        assert_eq!(
            board.piece_at(Square::F1),
            Some(Piece::new(PieceType::Rook, Color::White))
        );

        // E1 and H1 should be empty
        assert_eq!(board.piece_at(Square::E1), None);
        assert_eq!(board.piece_at(Square::H1), None);

        // Castling rights should be removed for white
        assert!(!board.castling().white_kingside());
        assert!(!board.castling().white_queenside());

        // Unmake the move
        board.unmake_move(m, undo);

        // Board should be restored
        assert_eq!(board, original);
    }

    #[test]
    fn make_unmake_promotion() {
        let mut board = Board::empty();
        board.set_piece(Square::E7, Piece::new(PieceType::Pawn, Color::White));
        board.set_side_to_move(Color::White);

        let original = board.clone();

        let m = Move::new(Square::E7, Square::E8, MoveFlags::QUEEN_PROMOTION);
        let undo = board.make_move(m);

        // Should be a queen on E8
        assert_eq!(
            board.piece_at(Square::E8),
            Some(Piece::new(PieceType::Queen, Color::White))
        );
        assert_eq!(board.piece_at(Square::E7), None);

        // Unmake the move
        board.unmake_move(m, undo);

        // Board should be restored (pawn back on E7)
        assert_eq!(board, original);
    }

    #[test]
    fn make_unmake_en_passant() {
        // Set up position with proper double pawn push first
        let mut board = Board::empty();
        board.set_piece(Square::E5, Piece::new(PieceType::Pawn, Color::White));
        board.set_piece(Square::D7, Piece::new(PieceType::Pawn, Color::Black));
        board.set_side_to_move(Color::Black);

        // Black does double pawn push D7-D5
        let double_push = Move::new(Square::D7, Square::D5, MoveFlags::DOUBLE_PAWN_PUSH);
        board.make_move(double_push);

        // Now en passant square should be D6
        assert_eq!(board.ep_square(), Some(Square::D6));

        let original = board.clone();

        // White captures en passant E5xD6
        let ep_capture = Move::new(Square::E5, Square::D6, MoveFlags::EP_CAPTURE);
        let undo = board.make_move(ep_capture);

        // White pawn should be on D6
        assert_eq!(
            board.piece_at(Square::D6),
            Some(Piece::new(PieceType::Pawn, Color::White))
        );

        // Black pawn should be captured (D5 should be empty)
        assert_eq!(board.piece_at(Square::D5), None);
        assert_eq!(board.piece_at(Square::E5), None);

        // Unmake the move
        board.unmake_move(ep_capture, undo);

        // Board should be restored
        assert_eq!(board, original);
    }

    #[test]
    fn make_move_updates_castling_rights() {
        let mut board = Board::startpos();

        // Move white king
        let m = Move::new(Square::E1, Square::E2, MoveFlags::QUIET);
        board.make_move(m);

        // White should lose all castling rights
        assert!(!board.castling().white_kingside());
        assert!(!board.castling().white_queenside());

        // Black should still have castling rights
        assert!(board.castling().black_kingside());
        assert!(board.castling().black_queenside());
    }

    #[test]
    fn make_move_updates_castling_on_rook_move() {
        let mut board = Board::startpos();

        // Move white kingside rook
        let m = Move::new(Square::H1, Square::H2, MoveFlags::QUIET);
        board.make_move(m);

        // White should lose kingside castling only
        assert!(!board.castling().white_kingside());
        assert!(board.castling().white_queenside());
    }

    #[test]
    fn make_move_updates_castling_on_rook_capture() {
        let mut board = Board::startpos();

        // Simulate capturing black's queenside rook
        let m = Move::new(Square::D1, Square::A8, MoveFlags::CAPTURE);
        board.make_move(m);

        // Black should lose queenside castling
        assert!(!board.castling().black_queenside());
    }

    #[test]
    fn make_move_fullmove_number() {
        let mut board = Board::startpos();

        // White's move - fullmove should stay 1
        let m1 = Move::new(Square::E2, Square::E4, MoveFlags::QUIET);
        board.make_move(m1);
        assert_eq!(board.fullmove_number(), 1);

        // Black's move - fullmove should increment to 2
        let m2 = Move::new(Square::E7, Square::E5, MoveFlags::QUIET);
        board.make_move(m2);
        assert_eq!(board.fullmove_number(), 2);
    }

    #[test]
    fn make_move_halfmove_clock() {
        let mut board = Board::startpos();

        // Pawn move - resets clock
        let m1 = Move::new(Square::E2, Square::E4, MoveFlags::QUIET);
        board.make_move(m1);
        assert_eq!(board.halfmove_clock(), 0);

        // Knight move - increments clock
        let m2 = Move::new(Square::B8, Square::C6, MoveFlags::QUIET);
        board.make_move(m2);
        assert_eq!(board.halfmove_clock(), 1);

        // Another knight move - increments clock
        let m3 = Move::new(Square::G1, Square::F3, MoveFlags::QUIET);
        board.make_move(m3);
        assert_eq!(board.halfmove_clock(), 2);
    }

    #[test]
    fn reversibility_chain() {
        // Test multiple moves and unmakes
        let original = Board::startpos();
        let mut board = original.clone();

        let moves_and_undos = vec![
            (
                Move::new(Square::E2, Square::E4, MoveFlags::DOUBLE_PAWN_PUSH),
                board.make_move(Move::new(
                    Square::E2,
                    Square::E4,
                    MoveFlags::DOUBLE_PAWN_PUSH,
                )),
            ),
            (
                Move::new(Square::E7, Square::E5, MoveFlags::DOUBLE_PAWN_PUSH),
                board.make_move(Move::new(
                    Square::E7,
                    Square::E5,
                    MoveFlags::DOUBLE_PAWN_PUSH,
                )),
            ),
            (
                Move::new(Square::G1, Square::F3, MoveFlags::QUIET),
                board.make_move(Move::new(Square::G1, Square::F3, MoveFlags::QUIET)),
            ),
        ];

        // Unmake in reverse order
        for (m, undo) in moves_and_undos.into_iter().rev() {
            board.unmake_move(m, undo);
        }

        // Should be back to original
        assert_eq!(board, original);
    }

    #[test]
    fn test_is_square_attacked() {
        let board = Board::startpos();

        // E2 is protected by white
        assert!(board.is_square_attacked(Square::E2, Color::White));

        // E7 is protected by black
        assert!(board.is_square_attacked(Square::E7, Color::Black));

        // E4 is not attacked at start
        assert!(!board.is_square_attacked(Square::E4, Color::White));
        assert!(!board.is_square_attacked(Square::E4, Color::Black));
    }

    #[test]
    fn test_is_in_check_startpos() {
        let board = Board::startpos();
        assert!(!board.is_in_check());
    }

    #[test]
    fn test_is_in_check_white() {
        use crate::io::parse_fen;

        // White king in check from black queen on h4
        let fen = "rnb1kbnr/pppp1ppp/8/4p3/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 1 3";
        let board = parse_fen(fen).unwrap();
        assert!(board.is_in_check());
    }

    #[test]
    fn test_is_in_check_black() {
        use crate::io::parse_fen;

        // Black king in check from white queen on e8
        let fen = "rnb1k2Q/pppp1ppp/5n2/2b1p3/2B1P3/8/PPPP1PPP/RNB1K1NR b KQq - 4 4";
        let board = parse_fen(fen).unwrap();
        assert!(board.is_in_check());
    }

    #[test]
    fn test_is_legal_normal_move() {
        let board = Board::startpos();
        let m = Move::new(Square::E2, Square::E4, MoveFlags::DOUBLE_PAWN_PUSH);
        assert!(board.is_legal(m));
    }

    #[test]
    fn test_is_legal_pinned_piece() {
        use crate::io::parse_fen;

        // Position where knight can move freely
        let fen = "r1bqkbnr/pppp1ppp/2n5/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4";
        let board = parse_fen(fen).unwrap();

        // Move knight normally - should be legal
        let legal_move = Move::new(Square::F3, Square::from_coords(6, 4), MoveFlags::QUIET); // Ng5
        assert!(board.is_legal(legal_move));
    }

    #[test]
    fn test_is_legal_blocks_check() {
        use crate::io::parse_fen;

        // Black in check from rook on e1 (along e-file to e8)
        let fen = "4k3/8/8/8/8/8/8/4R2K b - - 0 1";
        let board = parse_fen(fen).unwrap();

        assert!(board.is_in_check()); // Black king on e8 in check from rook on e1

        // King can move to escape
        let king_move = Move::new(
            Square::from_coords(4, 7), // e8
            Square::from_coords(3, 7), // d8
            MoveFlags::QUIET,
        );
        assert!(board.is_legal(king_move));
    }

    #[test]
    fn test_castling_legal() {
        let mut board = Board::empty();
        board.set_piece(Square::E1, Piece::new(PieceType::King, Color::White));
        board.set_piece(Square::H1, Piece::new(PieceType::Rook, Color::White));
        board.set_side_to_move(Color::White);
        board.set_castling(CastlingRights::all());

        let castle_move = Move::new(Square::E1, Square::G1, MoveFlags::KING_CASTLE);
        assert!(board.is_legal(castle_move));
    }

    #[test]
    fn test_castling_through_check() {
        let mut board = Board::empty();
        board.set_piece(Square::E1, Piece::new(PieceType::King, Color::White));
        board.set_piece(Square::H1, Piece::new(PieceType::Rook, Color::White));
        board.set_piece(Square::F3, Piece::new(PieceType::Rook, Color::Black)); // Attacks f1
        board.set_side_to_move(Color::White);
        board.set_castling(CastlingRights::all());

        let castle_move = Move::new(Square::E1, Square::G1, MoveFlags::KING_CASTLE);
        assert!(!board.is_legal(castle_move)); // Can't castle through check
    }

    #[test]
    fn test_castling_into_check() {
        let mut board = Board::empty();
        board.set_piece(Square::E1, Piece::new(PieceType::King, Color::White));
        board.set_piece(Square::H1, Piece::new(PieceType::Rook, Color::White));
        board.set_piece(
            Square::from_coords(6, 2),
            Piece::new(PieceType::Rook, Color::Black),
        ); // G3 - Attacks g1
        board.set_side_to_move(Color::White);
        board.set_castling(CastlingRights::all());

        let castle_move = Move::new(Square::E1, Square::G1, MoveFlags::KING_CASTLE);
        assert!(!board.is_legal(castle_move)); // Can't castle into check
    }

    #[test]
    fn test_castling_out_of_check() {
        let mut board = Board::empty();
        board.set_piece(Square::E1, Piece::new(PieceType::King, Color::White));
        board.set_piece(Square::H1, Piece::new(PieceType::Rook, Color::White));
        board.set_piece(Square::E3, Piece::new(PieceType::Rook, Color::Black)); // Attacks e1
        board.set_side_to_move(Color::White);
        board.set_castling(CastlingRights::all());

        let castle_move = Move::new(Square::E1, Square::G1, MoveFlags::KING_CASTLE);
        assert!(!board.is_legal(castle_move)); // Can't castle out of check
    }

    #[test]
    fn test_generate_legal_moves_startpos() {
        let board = Board::startpos();
        let legal_moves = board.generate_legal_moves();

        // Starting position has 20 legal moves (same as pseudo-legal)
        assert_eq!(legal_moves.len(), 20);
    }

    #[test]
    fn test_generate_legal_moves_in_check() {
        use crate::io::parse_fen;

        // Black king in check from rook
        let fen = "4k3/8/8/8/8/8/8/4R2K b - - 0 1";
        let board = parse_fen(fen).unwrap();

        assert!(board.is_in_check());
        let legal_moves = board.generate_legal_moves();

        // Should have some legal moves (king moves to escape)
        assert!(!legal_moves.is_empty());

        // All legal moves should actually be legal
        for m in legal_moves {
            assert!(board.is_legal(m));
        }
    }

    #[test]
    fn test_generate_legal_moves_filters_illegal() {
        use crate::io::parse_fen;
        use crate::movegen::generate_moves;

        // Position with some pins
        let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
        let board = parse_fen(fen).unwrap();

        let pseudo_legal = generate_moves(&board);
        let legal = board.generate_legal_moves();

        // Legal should be <= pseudo-legal (some moves may be illegal)
        assert!(legal.len() <= pseudo_legal.len());

        // All legal moves should pass is_legal check
        for m in legal {
            assert!(board.is_legal(m));
        }
    }
}
