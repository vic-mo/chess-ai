/// FEN (Forsyth-Edwards Notation) parsing and serialization.
///
/// FEN format: `rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1`
///
/// Components:
/// 1. Piece placement (rank 8 to rank 1, separated by /)
/// 2. Side to move (w or b)
/// 3. Castling rights (KQkq or - for none)
/// 4. En passant target square (e.g., e3 or - for none)
/// 5. Halfmove clock (50-move rule)
/// 6. Fullmove number
use crate::board::{Board, CastlingRights};
use crate::movegen::generate_moves;
use crate::piece::{Color, Piece, PieceType};
use crate::square::Square;

/// The starting position FEN string.
pub const STARTPOS_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

/// Error type for FEN parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FenError {
    /// FEN string has wrong number of components
    InvalidFormat(String),
    /// Invalid piece placement section
    InvalidPiecePlacement(String),
    /// Invalid side to move
    InvalidSideToMove(String),
    /// Invalid castling rights
    InvalidCastlingRights(String),
    /// Invalid en passant square
    InvalidEnPassant(String),
    /// Invalid halfmove clock
    InvalidHalfmoveClock(String),
    /// Invalid fullmove number
    InvalidFullmoveNumber(String),
}

impl std::fmt::Display for FenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FenError::InvalidFormat(s) => write!(f, "Invalid FEN format: {}", s),
            FenError::InvalidPiecePlacement(s) => write!(f, "Invalid piece placement: {}", s),
            FenError::InvalidSideToMove(s) => write!(f, "Invalid side to move: {}", s),
            FenError::InvalidCastlingRights(s) => write!(f, "Invalid castling rights: {}", s),
            FenError::InvalidEnPassant(s) => write!(f, "Invalid en passant square: {}", s),
            FenError::InvalidHalfmoveClock(s) => write!(f, "Invalid halfmove clock: {}", s),
            FenError::InvalidFullmoveNumber(s) => write!(f, "Invalid fullmove number: {}", s),
        }
    }
}

impl std::error::Error for FenError {}

/// Parse a FEN string into a Board.
///
/// # Example
/// ```
/// use engine::io::{parse_fen, ToFen, STARTPOS_FEN};
///
/// let board = parse_fen(STARTPOS_FEN).unwrap();
/// assert_eq!(board.to_fen(), STARTPOS_FEN);
/// ```
pub fn parse_fen(fen: &str) -> Result<Board, FenError> {
    let parts: Vec<&str> = fen.split_whitespace().collect();

    if parts.len() != 6 {
        return Err(FenError::InvalidFormat(format!(
            "Expected 6 components, got {}",
            parts.len()
        )));
    }

    let mut board = Board::empty();

    // 1. Parse piece placement
    parse_piece_placement(&mut board, parts[0])?;

    // 2. Parse side to move
    let side_to_move = parse_side_to_move(parts[1])?;
    board.set_side_to_move(side_to_move);

    // 3. Parse castling rights
    let castling_rights = parse_castling_rights(parts[2])?;
    board.set_castling(castling_rights);

    // 4. Parse en passant square
    let ep_square = parse_en_passant(parts[3])?;
    board.set_ep_square(ep_square);

    // 5. Parse halfmove clock
    let halfmove_clock = parts[4]
        .parse::<u32>()
        .map_err(|_| FenError::InvalidHalfmoveClock(parts[4].to_string()))?;
    board.set_halfmove_clock(halfmove_clock);

    // 6. Parse fullmove number
    let fullmove_number = parts[5]
        .parse::<u32>()
        .map_err(|_| FenError::InvalidFullmoveNumber(parts[5].to_string()))?;
    board.set_fullmove_number(fullmove_number);

    // Compute and set the Zobrist hash
    use crate::zobrist::zobrist_hash;
    let hash = zobrist_hash(&board);
    board.set_hash(hash);

    Ok(board)
}

/// Parse the piece placement component of a FEN string.
fn parse_piece_placement(board: &mut Board, placement: &str) -> Result<(), FenError> {
    let ranks: Vec<&str> = placement.split('/').collect();

    if ranks.len() != 8 {
        return Err(FenError::InvalidPiecePlacement(format!(
            "Expected 8 ranks, got {}",
            ranks.len()
        )));
    }

    // Ranks are listed from 8 to 1 in FEN
    for (rank_idx, rank_str) in ranks.iter().enumerate() {
        let rank = 7 - rank_idx; // Convert FEN rank (8..1) to internal rank (7..0)
        let mut file = 0;

        for ch in rank_str.chars() {
            if ch.is_ascii_digit() {
                // Empty squares
                let empty_count = ch
                    .to_digit(10)
                    .ok_or_else(|| FenError::InvalidPiecePlacement(rank_str.to_string()))?;
                file += empty_count as u8;
            } else {
                // Piece
                let piece = parse_piece(ch).ok_or_else(|| {
                    FenError::InvalidPiecePlacement(format!("Invalid piece character: {}", ch))
                })?;

                if file >= 8 {
                    return Err(FenError::InvalidPiecePlacement(format!(
                        "Too many pieces in rank: {}",
                        rank_str
                    )));
                }

                let square = Square::from_coords(file, rank as u8);
                board.set_piece(square, piece);
                file += 1;
            }
        }

        if file != 8 {
            return Err(FenError::InvalidPiecePlacement(format!(
                "Rank {} has {} files instead of 8",
                rank_idx + 1,
                file
            )));
        }
    }

    Ok(())
}

/// Parse a piece character from FEN notation.
fn parse_piece(ch: char) -> Option<Piece> {
    let (piece_type, color) = match ch {
        'P' => (PieceType::Pawn, Color::White),
        'N' => (PieceType::Knight, Color::White),
        'B' => (PieceType::Bishop, Color::White),
        'R' => (PieceType::Rook, Color::White),
        'Q' => (PieceType::Queen, Color::White),
        'K' => (PieceType::King, Color::White),
        'p' => (PieceType::Pawn, Color::Black),
        'n' => (PieceType::Knight, Color::Black),
        'b' => (PieceType::Bishop, Color::Black),
        'r' => (PieceType::Rook, Color::Black),
        'q' => (PieceType::Queen, Color::Black),
        'k' => (PieceType::King, Color::Black),
        _ => return None,
    };
    Some(Piece::new(piece_type, color))
}

/// Parse the side to move component.
fn parse_side_to_move(s: &str) -> Result<Color, FenError> {
    match s {
        "w" => Ok(Color::White),
        "b" => Ok(Color::Black),
        _ => Err(FenError::InvalidSideToMove(s.to_string())),
    }
}

/// Parse the castling rights component.
fn parse_castling_rights(s: &str) -> Result<CastlingRights, FenError> {
    if s == "-" {
        return Ok(CastlingRights::none());
    }

    let mut rights = CastlingRights::none();

    for ch in s.chars() {
        rights = match ch {
            'K' => rights.set_white_kingside(),
            'Q' => rights.set_white_queenside(),
            'k' => rights.set_black_kingside(),
            'q' => rights.set_black_queenside(),
            _ => {
                return Err(FenError::InvalidCastlingRights(format!(
                    "Invalid character: {}",
                    ch
                )))
            }
        };
    }

    Ok(rights)
}

/// Parse the en passant square component.
fn parse_en_passant(s: &str) -> Result<Option<Square>, FenError> {
    if s == "-" {
        return Ok(None);
    }

    Square::from_algebraic(s)
        .ok_or_else(|| FenError::InvalidEnPassant(s.to_string()))
        .map(Some)
}

/// Check if a FEN string is valid (simple check).
pub fn is_valid_fen(fen: &str) -> bool {
    parse_fen(fen).is_ok()
}

/// Extension trait for Board to support FEN serialization.
pub trait ToFen {
    /// Convert the board to a FEN string.
    fn to_fen(&self) -> String;
}

impl ToFen for Board {
    fn to_fen(&self) -> String {
        let mut fen = String::new();

        // 1. Piece placement (rank 8 to rank 1)
        for rank in (0..8).rev() {
            let mut empty_count = 0;

            for file in 0..8 {
                let square = Square::from_coords(file, rank);

                if let Some(piece) = self.piece_at(square) {
                    // Output any accumulated empty squares
                    if empty_count > 0 {
                        fen.push_str(&empty_count.to_string());
                        empty_count = 0;
                    }

                    // Output the piece
                    fen.push(piece_to_char(piece));
                } else {
                    empty_count += 1;
                }
            }

            // Output any remaining empty squares
            if empty_count > 0 {
                fen.push_str(&empty_count.to_string());
            }

            // Add rank separator (except after rank 1)
            if rank > 0 {
                fen.push('/');
            }
        }

        // 2. Side to move
        fen.push(' ');
        fen.push(match self.side_to_move() {
            Color::White => 'w',
            Color::Black => 'b',
        });

        // 3. Castling rights
        fen.push(' ');
        let castling = self.castling();
        if castling.bits() == 0 {
            fen.push('-');
        } else {
            if castling.white_kingside() {
                fen.push('K');
            }
            if castling.white_queenside() {
                fen.push('Q');
            }
            if castling.black_kingside() {
                fen.push('k');
            }
            if castling.black_queenside() {
                fen.push('q');
            }
        }

        // 4. En passant square
        fen.push(' ');
        if let Some(ep_square) = self.ep_square() {
            fen.push_str(&ep_square.to_algebraic());
        } else {
            fen.push('-');
        }

        // 5. Halfmove clock
        fen.push(' ');
        fen.push_str(&self.halfmove_clock().to_string());

        // 6. Fullmove number
        fen.push(' ');
        fen.push_str(&self.fullmove_number().to_string());

        fen
    }
}

/// Convert a piece to its FEN character representation.
fn piece_to_char(piece: Piece) -> char {
    let ch = match piece.piece_type {
        PieceType::Pawn => 'p',
        PieceType::Knight => 'n',
        PieceType::Bishop => 'b',
        PieceType::Rook => 'r',
        PieceType::Queen => 'q',
        PieceType::King => 'k',
    };

    if piece.color == Color::White {
        ch.to_ascii_uppercase()
    } else {
        ch
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_startpos() {
        let board = parse_fen(STARTPOS_FEN).unwrap();

        // Check piece placement
        assert_eq!(
            board.piece_at(Square::E1),
            Some(Piece::new(PieceType::King, Color::White))
        );
        assert_eq!(
            board.piece_at(Square::E8),
            Some(Piece::new(PieceType::King, Color::Black))
        );
        assert_eq!(
            board.piece_at(Square::A1),
            Some(Piece::new(PieceType::Rook, Color::White))
        );
        assert_eq!(
            board.piece_at(Square::H8),
            Some(Piece::new(PieceType::Rook, Color::Black))
        );
        assert_eq!(
            board.piece_at(Square::E2),
            Some(Piece::new(PieceType::Pawn, Color::White))
        );

        // Check metadata
        assert_eq!(board.side_to_move(), Color::White);
        assert!(board.castling().white_kingside());
        assert!(board.castling().white_queenside());
        assert!(board.castling().black_kingside());
        assert!(board.castling().black_queenside());
        assert_eq!(board.ep_square(), None);
        assert_eq!(board.halfmove_clock(), 0);
        assert_eq!(board.fullmove_number(), 1);
    }

    #[test]
    fn test_fen_roundtrip() {
        let board = parse_fen(STARTPOS_FEN).unwrap();
        assert_eq!(board.to_fen(), STARTPOS_FEN);
    }

    #[test]
    fn test_parse_fen_with_en_passant() {
        let fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
        let board = parse_fen(fen).unwrap();

        assert_eq!(board.ep_square(), Some(Square::E3));
        assert_eq!(board.to_fen(), fen);
    }

    #[test]
    fn test_parse_fen_no_castling() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w - - 0 1";
        let board = parse_fen(fen).unwrap();

        assert_eq!(board.castling().bits(), 0);
        assert_eq!(board.to_fen(), fen);
    }

    #[test]
    fn test_parse_fen_partial_castling() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w Kq - 0 1";
        let board = parse_fen(fen).unwrap();

        assert!(board.castling().white_kingside());
        assert!(!board.castling().white_queenside());
        assert!(!board.castling().black_kingside());
        assert!(board.castling().black_queenside());
        assert_eq!(board.to_fen(), fen);
    }

    #[test]
    fn test_parse_fen_black_to_move() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1";
        let board = parse_fen(fen).unwrap();

        assert_eq!(board.side_to_move(), Color::Black);
        assert_eq!(board.to_fen(), fen);
    }

    #[test]
    fn test_parse_complex_position() {
        let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
        let board = parse_fen(fen).unwrap();

        // Verify some key pieces
        assert_eq!(
            board.piece_at(Square::E1),
            Some(Piece::new(PieceType::King, Color::White))
        );
        assert_eq!(
            board.piece_at(Square::E8),
            Some(Piece::new(PieceType::King, Color::Black))
        );
        assert_eq!(
            board.piece_at(Square::F3),
            Some(Piece::new(PieceType::Queen, Color::White))
        );
        assert_eq!(
            board.piece_at(Square::E7),
            Some(Piece::new(PieceType::Queen, Color::Black))
        );

        assert_eq!(board.to_fen(), fen);
    }

    #[test]
    fn test_parse_fen_with_clocks() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 5 10";
        let board = parse_fen(fen).unwrap();

        assert_eq!(board.halfmove_clock(), 5);
        assert_eq!(board.fullmove_number(), 10);
        assert_eq!(board.to_fen(), fen);
    }

    #[test]
    fn test_parse_empty_board() {
        let fen = "8/8/8/8/8/8/8/8 w - - 0 1";
        let board = parse_fen(fen).unwrap();

        // All squares should be empty
        for sq in Square::all() {
            assert_eq!(board.piece_at(sq), None);
        }

        assert_eq!(board.to_fen(), fen);
    }

    #[test]
    fn test_parse_invalid_piece_count() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP w KQkq - 0 1"; // Missing rank
        assert!(parse_fen(fen).is_err());
    }

    #[test]
    fn test_parse_invalid_side() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR x KQkq - 0 1";
        assert!(parse_fen(fen).is_err());
    }

    #[test]
    fn test_parse_invalid_castling() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQx - 0 1";
        assert!(parse_fen(fen).is_err());
    }

    #[test]
    fn test_parse_invalid_en_passant() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq z9 0 1";
        assert!(parse_fen(fen).is_err());
    }

    #[test]
    fn test_is_valid_fen() {
        assert!(is_valid_fen(STARTPOS_FEN));
        assert!(is_valid_fen(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1"
        ));
        assert!(!is_valid_fen("invalid"));
        assert!(!is_valid_fen(""));
    }
}

// =============================================================================
// EPD (Extended Position Description) PARSING
// =============================================================================

/// EPD test position containing board state and expected best moves.
///
/// EPD format extends FEN with additional operations:
/// `FEN; bm MOVE1 MOVE2; id "Test Name"; c0 "Comment"`
///
/// # Example
/// ```
/// use engine::io::parse_epd;
///
/// let epd = r#"r1bqkb1r/pppp1ppp/2n2n2/4p2Q/2B1P3/8/PPPP1PPP/RNB1K1NR w KQkq - 0 1; bm Qxf7#; id "Scholar's Mate"; c0 "Checkmate in 1""#;
/// let test_pos = parse_epd(epd).unwrap();
/// assert_eq!(test_pos.id, "Scholar's Mate");
/// assert!(test_pos.best_moves.contains(&"Qxf7#".to_string()));
/// ```
#[derive(Debug, Clone)]
pub struct EpdTestPosition {
    /// The board position
    pub board: Board,
    /// List of acceptable best moves (in UCI or SAN format)
    pub best_moves: Vec<String>,
    /// Test identifier/name
    pub id: String,
    /// Optional comment/description
    pub comment: Option<String>,
}

/// Error type for EPD parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EpdError {
    /// Invalid FEN portion
    InvalidFen(String),
    /// Missing or invalid best move operation
    MissingBestMoves,
    /// Invalid EPD format
    InvalidFormat(String),
    /// Best move is not legal in the position
    IllegalMove(String),
}

impl std::fmt::Display for EpdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EpdError::InvalidFen(s) => write!(f, "Invalid FEN in EPD: {}", s),
            EpdError::MissingBestMoves => write!(f, "EPD missing best move (bm) operation"),
            EpdError::InvalidFormat(s) => write!(f, "Invalid EPD format: {}", s),
            EpdError::IllegalMove(s) => write!(f, "Best move is not legal: {}", s),
        }
    }
}

impl std::error::Error for EpdError {}

/// Parse an EPD (Extended Position Description) line.
///
/// EPD format: `FEN; operation1 value1; operation2 value2; ...`
///
/// Common operations:
/// - `bm` (best move): Expected best move(s)
/// - `id`: Test identifier/name
/// - `c0` through `c9`: Comments
/// - `am` (avoid move): Moves to avoid
///
/// # Example
/// ```
/// use engine::io::parse_epd;
///
/// let epd = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1; bm e2e4 d2d4; id \"Opening\"";
/// let test_pos = parse_epd(epd).unwrap();
/// assert_eq!(test_pos.best_moves.len(), 2);
/// ```
pub fn parse_epd(epd_line: &str) -> Result<EpdTestPosition, EpdError> {
    let epd_line = epd_line.trim();
    if epd_line.is_empty() {
        return Err(EpdError::InvalidFormat("Empty EPD line".to_string()));
    }

    // EPD format: FEN (4 fields) followed by operations
    // Example: rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - bm e2e4; id "test";
    let tokens: Vec<&str> = epd_line.split_whitespace().collect();

    if tokens.len() < 4 {
        return Err(EpdError::InvalidFormat(format!(
            "EPD requires at least 4 FEN fields, got {}",
            tokens.len()
        )));
    }

    // First 4 tokens are the FEN fields
    let full_fen = format!("{} {} {} {} 0 1",
        tokens[0], // position
        tokens[1], // side to move
        tokens[2], // castling
        tokens[3]  // en passant
    );

    let board = parse_fen(&full_fen)
        .map_err(|e| EpdError::InvalidFen(e.to_string()))?;

    // Parse operations starting from token 4
    // Operations are terminated by semicolons
    let operations_str = tokens[4..].join(" ");
    let operations: Vec<&str> = operations_str.split(';').collect();

    let mut best_moves = Vec::new();
    let mut id = String::new();
    let mut comment = None;

    for operation in operations {
        let operation = operation.trim();
        if operation.is_empty() {
            continue;
        }

        let op_tokens: Vec<&str> = operation.split_whitespace().collect();
        if op_tokens.is_empty() {
            continue;
        }

        let op_name = op_tokens[0];
        let op_values = &op_tokens[1..];

        match op_name {
            "bm" => {
                // Best moves - can be multiple
                for value in op_values {
                    let move_str = value.trim_matches('"').trim_matches(',');
                    if !move_str.is_empty() {
                        best_moves.push(move_str.to_string());
                    }
                }
            }
            "id" => {
                // Test ID - join remaining tokens and remove quotes
                id = op_values.join(" ")
                    .trim_matches('"')
                    .to_string();
            }
            "c0" | "c1" | "c2" | "c3" | "c4" | "c5" | "c6" | "c7" | "c8" | "c9" => {
                // Comment - join remaining tokens and remove quotes
                comment = Some(op_values.join(" ")
                    .trim_matches('"')
                    .to_string());
            }
            _ => {
                // Ignore unknown operations
            }
        }
    }

    if best_moves.is_empty() {
        return Err(EpdError::MissingBestMoves);
    }

    Ok(EpdTestPosition {
        board,
        best_moves,
        id,
        comment,
    })
}

/// Validate that all best moves in an EPD position are legal.
///
/// This is crucial to ensure test positions are valid.
/// Returns the first illegal move found, or None if all moves are legal.
pub fn validate_epd_moves(epd: &EpdTestPosition) -> Option<String> {
    let legal_moves = generate_moves(&epd.board);

    for best_move_str in &epd.best_moves {
        let mut found_legal = false;

        // Try to match the move string against legal moves
        // Support both UCI format (e2e4) and SAN format (e4, Nf3, O-O, etc.)
        for i in 0..legal_moves.len() {
            let legal_move = legal_moves[i];
            let uci = legal_move.to_uci();

            // Direct UCI match
            if uci == *best_move_str || uci == best_move_str.trim_end_matches(|c| c == '+' || c == '#' || c == '!' || c == '?') {
                found_legal = true;
                break;
            }

            // Also check algebraic notation variations
            let to_sq = legal_move.to().to_algebraic();

            // Handle variations like "Nf3" vs "nf3", case-insensitive for simple moves
            if best_move_str.to_lowercase().ends_with(&to_sq) {
                // For now, we'll be lenient and accept moves that end with the target square
                // Full SAN parsing would require more complex logic
                found_legal = true;
                break;
            }
        }

        if !found_legal {
            return Some(best_move_str.clone());
        }
    }

    None
}

/// Load and parse multiple EPD positions from a file.
///
/// Returns a vector of successfully parsed positions.
/// Skips lines that fail to parse (with optional error reporting).
pub fn load_epd_file(file_path: &str) -> Result<Vec<EpdTestPosition>, std::io::Error> {
    use std::fs;
    use std::io::{BufRead, BufReader};

    let file = fs::File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut positions = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        match parse_epd(line) {
            Ok(pos) => positions.push(pos),
            Err(e) => {
                eprintln!("Warning: Failed to parse EPD at line {}: {}", line_num + 1, e);
                eprintln!("  Line: {}", line);
            }
        }
    }

    Ok(positions)
}

#[cfg(test)]
mod epd_tests {
    use super::*;

    #[test]
    fn test_parse_simple_epd() {
        let epd = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1; bm e2e4; id \"Test\"";
        let result = parse_epd(epd).unwrap();

        assert_eq!(result.best_moves.len(), 1);
        assert_eq!(result.best_moves[0], "e2e4");
        assert_eq!(result.id, "Test");
    }

    #[test]
    fn test_parse_epd_multiple_best_moves() {
        let epd = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq -; bm e2e4 d2d4; id \"Opening\"";
        let result = parse_epd(epd).unwrap();

        assert_eq!(result.best_moves.len(), 2);
        assert!(result.best_moves.contains(&"e2e4".to_string()));
        assert!(result.best_moves.contains(&"d2d4".to_string()));
    }

    #[test]
    fn test_parse_epd_with_comment() {
        let epd = r#"rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3; bm e7e5; id "Test"; c0 "King's pawn opening""#;
        let result = parse_epd(epd).unwrap();

        assert_eq!(result.best_moves[0], "e7e5");
        assert_eq!(result.comment.as_ref().unwrap(), "King's pawn opening");
    }

    #[test]
    fn test_parse_epd_no_best_move() {
        let epd = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1; id \"Test\"";
        let result = parse_epd(epd);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), EpdError::MissingBestMoves));
    }

    #[test]
    fn test_parse_epd_minimal_fen() {
        // EPD with only 4 FEN fields (standard EPD format)
        let epd = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq -; bm e2e4";
        let result = parse_epd(epd).unwrap();

        assert_eq!(result.board.side_to_move(), Color::White);
        assert_eq!(result.best_moves[0], "e2e4");
    }
}
