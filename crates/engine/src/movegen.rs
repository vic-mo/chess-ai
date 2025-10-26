use crate::attacks::{
    bishop_attacks, king_attacks, knight_attacks, pawn_attacks, queen_attacks, rook_attacks,
};
use crate::bitboard::Bitboard;
use crate::board::Board;
use crate::movelist::MoveList;
use crate::piece::{Color, PieceType};
use crate::r#move::{Move, MoveFlags};
use crate::square::Square;

/// Generate all pseudo-legal moves for the current position.
///
/// Pseudo-legal means the moves follow piece movement rules but may leave
/// the king in check. Legality checking is done separately.
///
/// # Example
/// ```
/// use engine::board::Board;
/// use engine::movegen::generate_moves;
///
/// let board = Board::startpos();
/// let moves = generate_moves(&board);
/// assert_eq!(moves.len(), 20); // 16 pawn moves + 4 knight moves
/// ```
pub fn generate_moves(board: &Board) -> MoveList {
    let mut moves = MoveList::new();

    let us = board.side_to_move();
    let them = us.opponent();

    let our_pieces = board.color_bb(us);
    let their_pieces = board.color_bb(them);
    let occupied = board.occupied();
    let empty = board.empty_squares();

    // Generate moves for each piece type
    generate_pawn_moves(board, &mut moves, us, our_pieces, their_pieces, empty);
    generate_knight_moves(board, &mut moves, us, our_pieces, occupied);
    generate_bishop_moves(board, &mut moves, us, our_pieces, occupied);
    generate_rook_moves(board, &mut moves, us, our_pieces, occupied);
    generate_queen_moves(board, &mut moves, us, our_pieces, occupied);
    generate_king_moves(board, &mut moves, us, our_pieces, occupied);

    moves
}

// =============================================================================
// PAWN MOVES
// =============================================================================

/// Generate pawn moves (pushes and captures).
///
/// Note: Promotions, en passant, and double pushes will be completed in Session 9-10.
fn generate_pawn_moves(
    board: &Board,
    moves: &mut MoveList,
    us: Color,
    _our_pieces: Bitboard,
    their_pieces: Bitboard,
    empty: Bitboard,
) {
    let pawns = board.piece_bb(PieceType::Pawn, us);

    // Single pushes
    let push_direction = if us == Color::White { 8 } else { -8 };

    for from_sq in pawns {
        let from_idx = from_sq.index() as i8;
        let to_idx = from_idx + push_direction;

        if (0..64).contains(&to_idx) {
            let to_sq = Square::new(to_idx as u8);

            // Check for promotion rank (will be fully implemented in Session 9-10)
            let is_promotion_rank = (us == Color::White && to_sq.rank() == 7)
                || (us == Color::Black && to_sq.rank() == 0);

            // Single push
            if empty.contains(to_sq) {
                if is_promotion_rank {
                    // TODO: Generate all 4 promotion moves (Q, R, B, N)
                    // For now, just generate queen promotion as placeholder
                    moves.push(Move::new(from_sq, to_sq, MoveFlags::QUEEN_PROMOTION));
                } else {
                    moves.push(Move::new(from_sq, to_sq, MoveFlags::QUIET));
                }
            }

            // Double push
            if !is_promotion_rank && empty.contains(to_sq) {
                let start_rank = if us == Color::White { 1 } else { 6 };
                if from_sq.rank() == start_rank {
                    let double_to_idx = from_idx + push_direction * 2;
                    if (0..64).contains(&double_to_idx) {
                        let double_to_sq = Square::new(double_to_idx as u8);
                        if empty.contains(double_to_sq) {
                            moves.push(Move::new(
                                from_sq,
                                double_to_sq,
                                MoveFlags::DOUBLE_PAWN_PUSH,
                            ));
                        }
                    }
                }
            }
        }

        // Captures
        let attacks = pawn_attacks(from_sq, us);
        let capture_targets = attacks & their_pieces;

        for to_sq in capture_targets {
            let is_promotion_rank = (us == Color::White && to_sq.rank() == 7)
                || (us == Color::Black && to_sq.rank() == 0);

            if is_promotion_rank {
                // TODO: Generate all 4 promotion captures (Q, R, B, N)
                moves.push(Move::new(
                    from_sq,
                    to_sq,
                    MoveFlags::QUEEN_PROMOTION_CAPTURE,
                ));
            } else {
                moves.push(Move::new(from_sq, to_sq, MoveFlags::CAPTURE));
            }
        }

        // TODO: En passant captures (Session 9-10)
    }
}

// =============================================================================
// KNIGHT MOVES
// =============================================================================

/// Generate knight moves.
fn generate_knight_moves(
    board: &Board,
    moves: &mut MoveList,
    us: Color,
    our_pieces: Bitboard,
    occupied: Bitboard,
) {
    let knights = board.piece_bb(PieceType::Knight, us);

    for from_sq in knights {
        let attacks = knight_attacks(from_sq);

        // Quiet moves (to empty squares)
        let quiet_targets = attacks & !occupied;
        for to_sq in quiet_targets {
            moves.push(Move::new(from_sq, to_sq, MoveFlags::QUIET));
        }

        // Captures (to enemy pieces)
        let capture_targets = attacks & !our_pieces & occupied;
        for to_sq in capture_targets {
            moves.push(Move::new(from_sq, to_sq, MoveFlags::CAPTURE));
        }
    }
}

// =============================================================================
// BISHOP MOVES
// =============================================================================

/// Generate bishop moves.
fn generate_bishop_moves(
    board: &Board,
    moves: &mut MoveList,
    us: Color,
    our_pieces: Bitboard,
    occupied: Bitboard,
) {
    let bishops = board.piece_bb(PieceType::Bishop, us);

    for from_sq in bishops {
        let attacks = bishop_attacks(from_sq, occupied);

        // Quiet moves
        let quiet_targets = attacks & !occupied;
        for to_sq in quiet_targets {
            moves.push(Move::new(from_sq, to_sq, MoveFlags::QUIET));
        }

        // Captures
        let capture_targets = attacks & !our_pieces & occupied;
        for to_sq in capture_targets {
            moves.push(Move::new(from_sq, to_sq, MoveFlags::CAPTURE));
        }
    }
}

// =============================================================================
// ROOK MOVES
// =============================================================================

/// Generate rook moves.
fn generate_rook_moves(
    board: &Board,
    moves: &mut MoveList,
    us: Color,
    our_pieces: Bitboard,
    occupied: Bitboard,
) {
    let rooks = board.piece_bb(PieceType::Rook, us);

    for from_sq in rooks {
        let attacks = rook_attacks(from_sq, occupied);

        // Quiet moves
        let quiet_targets = attacks & !occupied;
        for to_sq in quiet_targets {
            moves.push(Move::new(from_sq, to_sq, MoveFlags::QUIET));
        }

        // Captures
        let capture_targets = attacks & !our_pieces & occupied;
        for to_sq in capture_targets {
            moves.push(Move::new(from_sq, to_sq, MoveFlags::CAPTURE));
        }
    }
}

// =============================================================================
// QUEEN MOVES
// =============================================================================

/// Generate queen moves.
fn generate_queen_moves(
    board: &Board,
    moves: &mut MoveList,
    us: Color,
    our_pieces: Bitboard,
    occupied: Bitboard,
) {
    let queens = board.piece_bb(PieceType::Queen, us);

    for from_sq in queens {
        let attacks = queen_attacks(from_sq, occupied);

        // Quiet moves
        let quiet_targets = attacks & !occupied;
        for to_sq in quiet_targets {
            moves.push(Move::new(from_sq, to_sq, MoveFlags::QUIET));
        }

        // Captures
        let capture_targets = attacks & !our_pieces & occupied;
        for to_sq in capture_targets {
            moves.push(Move::new(from_sq, to_sq, MoveFlags::CAPTURE));
        }
    }
}

// =============================================================================
// KING MOVES
// =============================================================================

/// Generate king moves (non-castling).
///
/// Note: Castling will be added in Session 9-10.
fn generate_king_moves(
    board: &Board,
    moves: &mut MoveList,
    us: Color,
    our_pieces: Bitboard,
    occupied: Bitboard,
) {
    let kings = board.piece_bb(PieceType::King, us);

    for from_sq in kings {
        let attacks = king_attacks(from_sq);

        // Quiet moves
        let quiet_targets = attacks & !occupied;
        for to_sq in quiet_targets {
            moves.push(Move::new(from_sq, to_sq, MoveFlags::QUIET));
        }

        // Captures
        let capture_targets = attacks & !our_pieces & occupied;
        for to_sq in capture_targets {
            moves.push(Move::new(from_sq, to_sq, MoveFlags::CAPTURE));
        }
    }

    // TODO: Castling (Session 9-10)
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::piece::Piece;

    #[test]
    fn test_startpos_move_count() {
        let board = Board::startpos();
        let moves = generate_moves(&board);

        // Starting position has 20 legal moves:
        // 16 pawn moves (8 single pushes + 8 double pushes)
        // 4 knight moves (2 knights * 2 moves each)
        assert_eq!(moves.len(), 20);
    }

    #[test]
    fn test_startpos_pawn_moves() {
        let board = Board::startpos();
        let moves = generate_moves(&board);

        // Count pawn moves
        let pawn_moves = moves
            .iter()
            .filter(|m| {
                board
                    .piece_at(m.from())
                    .is_some_and(|p| p.piece_type == PieceType::Pawn)
            })
            .count();

        assert_eq!(pawn_moves, 16); // 8 single + 8 double pushes
    }

    #[test]
    fn test_startpos_knight_moves() {
        let board = Board::startpos();
        let moves = generate_moves(&board);

        // Count knight moves
        let knight_moves = moves
            .iter()
            .filter(|m| {
                board
                    .piece_at(m.from())
                    .is_some_and(|p| p.piece_type == PieceType::Knight)
            })
            .count();

        assert_eq!(knight_moves, 4); // 2 knights * 2 squares each
    }

    #[test]
    fn test_knight_center_moves() {
        let mut board = Board::empty();
        board.set_piece(Square::E4, Piece::new(PieceType::Knight, Color::White));
        board.set_side_to_move(Color::White);

        let moves = generate_moves(&board);

        // Knight on E4 has 8 possible moves on empty board
        assert_eq!(moves.len(), 8);

        // All should be quiet moves
        assert!(moves.iter().all(|m| !m.is_capture()));
    }

    #[test]
    fn test_knight_captures() {
        let mut board = Board::empty();
        board.set_piece(Square::E4, Piece::new(PieceType::Knight, Color::White));
        board.set_piece(
            Square::from_coords(5, 5),
            Piece::new(PieceType::Pawn, Color::Black),
        ); // F6
        board.set_piece(
            Square::from_coords(3, 5),
            Piece::new(PieceType::Pawn, Color::Black),
        ); // D6
        board.set_side_to_move(Color::White);

        let moves = generate_moves(&board);

        // 6 quiet moves + 2 captures = 8 total
        assert_eq!(moves.len(), 8);

        let captures = moves.iter().filter(|m| m.is_capture()).count();
        assert_eq!(captures, 2);
    }

    #[test]
    fn test_bishop_diagonal_moves() {
        let mut board = Board::empty();
        board.set_piece(Square::E4, Piece::new(PieceType::Bishop, Color::White));
        board.set_side_to_move(Color::White);

        let moves = generate_moves(&board);

        // Bishop on E4: 13 diagonal squares (4+3+3+3 in each direction)
        assert_eq!(moves.len(), 13);
    }

    #[test]
    fn test_rook_rank_file_moves() {
        let mut board = Board::empty();
        board.set_piece(Square::E4, Piece::new(PieceType::Rook, Color::White));
        board.set_side_to_move(Color::White);

        let moves = generate_moves(&board);

        // Rook on E4: 14 squares (7 on file + 7 on rank)
        assert_eq!(moves.len(), 14);
    }

    #[test]
    fn test_queen_all_directions() {
        let mut board = Board::empty();
        board.set_piece(Square::E4, Piece::new(PieceType::Queen, Color::White));
        board.set_side_to_move(Color::White);

        let moves = generate_moves(&board);

        // Queen on E4: 27 squares (13 diagonal + 14 rank/file)
        assert_eq!(moves.len(), 27);
    }

    #[test]
    fn test_king_adjacent_moves() {
        let mut board = Board::empty();
        board.set_piece(Square::E4, Piece::new(PieceType::King, Color::White));
        board.set_side_to_move(Color::White);

        let moves = generate_moves(&board);

        // King on E4: 8 adjacent squares
        assert_eq!(moves.len(), 8);
    }

    #[test]
    fn test_pawn_single_push() {
        let mut board = Board::empty();
        board.set_piece(Square::E2, Piece::new(PieceType::Pawn, Color::White));
        board.set_side_to_move(Color::White);

        let moves = generate_moves(&board);

        // Pawn on E2 can move to E3 (single) and E4 (double)
        assert_eq!(moves.len(), 2);

        // Check single push
        assert!(moves.iter().any(|m| {
            m.from() == Square::E2 && m.to() == Square::E3 && !m.is_double_pawn_push()
        }));

        // Check double push
        assert!(moves.iter().any(|m| {
            m.from() == Square::E2 && m.to() == Square::E4 && m.is_double_pawn_push()
        }));
    }

    #[test]
    fn test_pawn_captures() {
        let mut board = Board::empty();
        board.set_piece(Square::E4, Piece::new(PieceType::Pawn, Color::White));
        board.set_piece(Square::D5, Piece::new(PieceType::Pawn, Color::Black));
        board.set_piece(
            Square::from_coords(5, 4),
            Piece::new(PieceType::Pawn, Color::Black),
        ); // F5
        board.set_side_to_move(Color::White);

        let moves = generate_moves(&board);

        // 1 push + 2 captures = 3 moves
        assert_eq!(moves.len(), 3);

        let captures = moves.iter().filter(|m| m.is_capture()).count();
        assert_eq!(captures, 2);
    }

    #[test]
    fn test_pawn_blocked() {
        let mut board = Board::empty();
        board.set_piece(Square::E2, Piece::new(PieceType::Pawn, Color::White));
        board.set_piece(Square::E3, Piece::new(PieceType::Pawn, Color::Black)); // Blocker
        board.set_side_to_move(Color::White);

        let moves = generate_moves(&board);

        // Pawn is completely blocked
        assert_eq!(moves.len(), 0);
    }

    #[test]
    fn test_black_pawn_moves() {
        let mut board = Board::empty();
        board.set_piece(Square::E7, Piece::new(PieceType::Pawn, Color::Black));
        board.set_side_to_move(Color::Black);

        let moves = generate_moves(&board);

        // Black pawn on E7 can move to E6 (single) and E5 (double)
        assert_eq!(moves.len(), 2);

        // Check moves go in correct direction (down for black)
        assert!(moves
            .iter()
            .any(|m| m.from() == Square::E7 && m.to().rank() < Square::E7.rank()));
    }

    #[test]
    fn test_no_friendly_captures() {
        let mut board = Board::empty();
        board.set_piece(Square::E4, Piece::new(PieceType::Knight, Color::White));
        board.set_piece(
            Square::from_coords(5, 5),
            Piece::new(PieceType::Pawn, Color::White),
        ); // F6 - friendly
        board.set_side_to_move(Color::White);

        let moves = generate_moves(&board);

        // Knight should have 7 moves (8 - 1 blocked by friendly)
        // Pawn should have 1 move (F6 to F7)
        // Total: 8 moves
        assert_eq!(moves.len(), 8);

        // Count knight moves specifically
        let knight_moves = moves
            .iter()
            .filter(|m| {
                board
                    .piece_at(m.from())
                    .is_some_and(|p| p.piece_type == PieceType::Knight)
            })
            .count();
        assert_eq!(knight_moves, 7);

        // None should be captures
        assert_eq!(moves.iter().filter(|m| m.is_capture()).count(), 0);
    }
}
