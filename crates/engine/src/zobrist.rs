/// Zobrist hashing for chess positions.
///
/// Zobrist hashing is a technique for efficiently hashing chess positions by XORing
/// precomputed random numbers. This enables:
/// - Fast position comparison
/// - Transposition table implementation
/// - Position repetition detection
use crate::board::{Board, CastlingRights};
use crate::piece::{Color, Piece};
use crate::square::Square;
use once_cell::sync::Lazy;

/// Zobrist hash keys for all board elements.
pub struct ZobristKeys {
    /// Keys for each piece type, color, and square [piece_type][color][square]
    pub pieces: [[[u64; 64]; 2]; 6],
    /// Key for black to move (XOR this when it's black's turn)
    pub black_to_move: u64,
    /// Keys for castling rights [0-15] (4 bits for WK, WQ, BK, BQ)
    pub castling: [u64; 16],
    /// Keys for en passant file [0-7] (a-h files)
    pub en_passant: [u64; 8],
}

/// Generate pseudorandom 64-bit numbers using a simple LCG.
const fn prng(mut seed: u64) -> u64 {
    // LCG parameters from Numerical Recipes
    seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
    seed
}

/// Generate all Zobrist keys at compile time.
const fn generate_zobrist_keys() -> ZobristKeys {
    let mut keys = ZobristKeys {
        pieces: [[[0; 64]; 2]; 6],
        black_to_move: 0,
        castling: [0; 16],
        en_passant: [0; 8],
    };

    // Start with a fixed seed for reproducibility
    let mut seed: u64 = 0x1234_5678_9ABC_DEF0;

    // Generate piece keys
    let mut piece_type = 0;
    while piece_type < 6 {
        let mut color = 0;
        while color < 2 {
            let mut square = 0;
            while square < 64 {
                seed = prng(seed);
                keys.pieces[piece_type][color][square] = seed;
                square += 1;
            }
            color += 1;
        }
        piece_type += 1;
    }

    // Generate side to move key
    seed = prng(seed);
    keys.black_to_move = seed;

    // Generate castling keys
    let mut i = 0;
    while i < 16 {
        seed = prng(seed);
        keys.castling[i] = seed;
        i += 1;
    }

    // Generate en passant keys
    let mut i = 0;
    while i < 8 {
        seed = prng(seed);
        keys.en_passant[i] = seed;
        i += 1;
    }

    keys
}

/// Global Zobrist keys (initialized once at program start).
pub static ZOBRIST: Lazy<ZobristKeys> = Lazy::new(generate_zobrist_keys);

/// Calculate the Zobrist hash for a board position.
///
/// # Example
/// ```
/// use engine::board::Board;
/// use engine::zobrist::zobrist_hash;
///
/// let board = Board::startpos();
/// let hash = zobrist_hash(&board);
/// assert_ne!(hash, 0); // Should have a non-zero hash
/// ```
pub fn zobrist_hash(board: &Board) -> u64 {
    let mut hash: u64 = 0;

    // Hash all pieces on the board
    for square in Square::all() {
        if let Some(piece) = board.piece_at(square) {
            let piece_key = ZOBRIST.pieces[piece.piece_type.index()][piece.color.index()]
                [square.index() as usize];
            hash ^= piece_key;
        }
    }

    // Hash side to move
    if board.side_to_move() == Color::Black {
        hash ^= ZOBRIST.black_to_move;
    }

    // Hash castling rights
    let castling_bits = board.castling().bits();
    hash ^= ZOBRIST.castling[castling_bits as usize];

    // Hash en passant file
    if let Some(ep_square) = board.ep_square() {
        hash ^= ZOBRIST.en_passant[ep_square.file() as usize];
    }

    hash
}

/// Update a hash when a piece is added to a square.
#[inline(always)]
pub fn hash_piece(hash: u64, piece: Piece, square: Square) -> u64 {
    hash ^ ZOBRIST.pieces[piece.piece_type.index()][piece.color.index()][square.index() as usize]
}

/// Update a hash when side to move changes.
#[inline(always)]
pub fn hash_side_to_move(hash: u64) -> u64 {
    hash ^ ZOBRIST.black_to_move
}

/// Update a hash when castling rights change.
#[inline(always)]
pub fn hash_castling(hash: u64, old_rights: CastlingRights, new_rights: CastlingRights) -> u64 {
    // XOR out old rights, XOR in new rights
    hash ^ ZOBRIST.castling[old_rights.bits() as usize]
        ^ ZOBRIST.castling[new_rights.bits() as usize]
}

/// Update a hash when en passant square changes.
#[inline(always)]
pub fn hash_en_passant(hash: u64, old_ep: Option<Square>, new_ep: Option<Square>) -> u64 {
    let mut h = hash;

    // XOR out old en passant
    if let Some(sq) = old_ep {
        h ^= ZOBRIST.en_passant[sq.file() as usize];
    }

    // XOR in new en passant
    if let Some(sq) = new_ep {
        h ^= ZOBRIST.en_passant[sq.file() as usize];
    }

    h
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::parse_fen;
    use crate::piece::PieceType;

    #[test]
    fn test_zobrist_startpos_nonzero() {
        let board = Board::startpos();
        let hash = zobrist_hash(&board);
        assert_ne!(hash, 0);
    }

    #[test]
    fn test_zobrist_different_positions() {
        let board1 = Board::startpos();
        let fen2 = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
        let board2 = parse_fen(fen2).unwrap();

        let hash1 = zobrist_hash(&board1);
        let hash2 = zobrist_hash(&board2);

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_zobrist_same_position() {
        let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
        let board1 = parse_fen(fen).unwrap();
        let board2 = parse_fen(fen).unwrap();

        assert_eq!(zobrist_hash(&board1), zobrist_hash(&board2));
    }

    #[test]
    fn test_zobrist_side_to_move() {
        let fen_white = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let fen_black = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1";

        let board_white = parse_fen(fen_white).unwrap();
        let board_black = parse_fen(fen_black).unwrap();

        let hash_white = zobrist_hash(&board_white);
        let hash_black = zobrist_hash(&board_black);

        // Only difference is side to move
        assert_ne!(hash_white, hash_black);

        // XOR with black_to_move should toggle
        assert_eq!(hash_white ^ ZOBRIST.black_to_move, hash_black);
    }

    #[test]
    fn test_zobrist_castling_rights() {
        let fen1 = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let fen2 = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w Kq - 0 1";

        let board1 = parse_fen(fen1).unwrap();
        let board2 = parse_fen(fen2).unwrap();

        assert_ne!(zobrist_hash(&board1), zobrist_hash(&board2));
    }

    #[test]
    fn test_zobrist_en_passant() {
        let fen1 = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let fen2 = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";

        let board1 = parse_fen(fen1).unwrap();
        let board2 = parse_fen(fen2).unwrap();

        assert_ne!(zobrist_hash(&board1), zobrist_hash(&board2));
    }

    #[test]
    fn test_hash_piece_toggle() {
        let board = Board::empty();
        let hash = zobrist_hash(&board);

        let piece = Piece::new(PieceType::Pawn, Color::White);
        let square = Square::E2;

        // Add piece
        let hash_with_piece = hash_piece(hash, piece, square);
        assert_ne!(hash, hash_with_piece);

        // Remove piece (XOR again)
        let hash_removed = hash_piece(hash_with_piece, piece, square);
        assert_eq!(hash, hash_removed);
    }

    #[test]
    fn test_incremental_hash() {
        use crate::r#move::{Move, MoveFlags};

        let mut board = Board::startpos();
        let initial_hash = zobrist_hash(&board);

        // Make a move
        let m = Move::new(Square::E2, Square::E4, MoveFlags::DOUBLE_PAWN_PUSH);
        board.make_move(m);

        // Hash should have changed
        let new_hash = zobrist_hash(&board);
        assert_ne!(initial_hash, new_hash);

        // The board's internal hash should match calculated hash
        assert_eq!(board.hash(), new_hash);
    }

    #[test]
    fn test_make_unmake_hash() {
        use crate::r#move::{Move, MoveFlags};

        let mut board = Board::startpos();
        let initial_hash = board.hash();

        // Make and unmake a move
        let m = Move::new(Square::E2, Square::E4, MoveFlags::DOUBLE_PAWN_PUSH);
        let undo = board.make_move(m);
        board.unmake_move(m, undo);

        // Hash should be back to initial
        assert_eq!(board.hash(), initial_hash);
    }
}
