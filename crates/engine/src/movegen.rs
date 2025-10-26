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

/// Generate pawn moves (pushes, captures, promotions, en passant).
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

            // Check for promotion rank
            let is_promotion_rank = (us == Color::White && to_sq.rank() == 7)
                || (us == Color::Black && to_sq.rank() == 0);

            // Single push
            if empty.contains(to_sq) {
                if is_promotion_rank {
                    // Generate all 4 promotion types
                    moves.push(Move::new(from_sq, to_sq, MoveFlags::QUEEN_PROMOTION));
                    moves.push(Move::new(from_sq, to_sq, MoveFlags::ROOK_PROMOTION));
                    moves.push(Move::new(from_sq, to_sq, MoveFlags::BISHOP_PROMOTION));
                    moves.push(Move::new(from_sq, to_sq, MoveFlags::KNIGHT_PROMOTION));
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
                // Generate all 4 promotion capture types
                moves.push(Move::new(
                    from_sq,
                    to_sq,
                    MoveFlags::QUEEN_PROMOTION_CAPTURE,
                ));
                moves.push(Move::new(from_sq, to_sq, MoveFlags::ROOK_PROMOTION_CAPTURE));
                moves.push(Move::new(
                    from_sq,
                    to_sq,
                    MoveFlags::BISHOP_PROMOTION_CAPTURE,
                ));
                moves.push(Move::new(
                    from_sq,
                    to_sq,
                    MoveFlags::KNIGHT_PROMOTION_CAPTURE,
                ));
            } else {
                moves.push(Move::new(from_sq, to_sq, MoveFlags::CAPTURE));
            }
        }

        // En passant captures
        if let Some(ep_square) = board.ep_square() {
            let attacks = pawn_attacks(from_sq, us);
            if attacks.contains(ep_square) {
                moves.push(Move::new(from_sq, ep_square, MoveFlags::EP_CAPTURE));
            }
        }
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

/// Generate king moves (including castling).
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

        // Castling
        generate_castling_moves(board, moves, us, from_sq, occupied);
    }
}

/// Generate castling moves for the given king position.
///
/// This checks if castling is pseudo-legal (has rights, squares are empty).
/// Legality checking (not in check, not moving through check) is done in Session 13.
fn generate_castling_moves(
    board: &Board,
    moves: &mut MoveList,
    us: Color,
    king_sq: Square,
    occupied: Bitboard,
) {
    let castling = board.castling();

    // Kingside castling
    if (us == Color::White && castling.white_kingside())
        || (us == Color::Black && castling.black_kingside())
    {
        // Check if squares between king and rook are empty
        let squares_to_check = if us == Color::White {
            // f1, g1
            vec![Square::F1, Square::G1]
        } else {
            // f8, g8
            vec![Square::from_coords(5, 7), Square::from_coords(6, 7)]
        };

        let all_empty = squares_to_check.iter().all(|&sq| !occupied.contains(sq));

        if all_empty {
            let to_sq = if us == Color::White {
                Square::G1
            } else {
                Square::from_coords(6, 7)
            };
            moves.push(Move::new(king_sq, to_sq, MoveFlags::KING_CASTLE));
        }
    }

    // Queenside castling
    if (us == Color::White && castling.white_queenside())
        || (us == Color::Black && castling.black_queenside())
    {
        // Check if squares between king and rook are empty
        let squares_to_check = if us == Color::White {
            // b1, c1, d1
            vec![Square::B1, Square::C1, Square::D1]
        } else {
            // b8, c8, d8
            vec![
                Square::from_coords(1, 7),
                Square::from_coords(2, 7),
                Square::from_coords(3, 7),
            ]
        };

        let all_empty = squares_to_check.iter().all(|&sq| !occupied.contains(sq));

        if all_empty {
            let to_sq = if us == Color::White {
                Square::C1
            } else {
                Square::from_coords(2, 7)
            };
            moves.push(Move::new(king_sq, to_sq, MoveFlags::QUEEN_CASTLE));
        }
    }
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

    #[test]
    fn test_en_passant_generation() {
        let mut board = Board::empty();
        board.set_piece(Square::E5, Piece::new(PieceType::Pawn, Color::White));
        board.set_piece(Square::D5, Piece::new(PieceType::Pawn, Color::Black));
        board.set_side_to_move(Color::White);
        board.set_ep_square(Some(Square::D6));

        let moves = generate_moves(&board);

        // Should have regular pawn push + en passant capture
        let ep_moves = moves.iter().filter(|m| m.is_en_passant()).count();
        assert_eq!(ep_moves, 1);

        let ep_move = moves.iter().find(|m| m.is_en_passant()).unwrap();
        assert_eq!(ep_move.from(), Square::E5);
        assert_eq!(ep_move.to(), Square::D6);
    }

    #[test]
    fn test_all_promotion_types() {
        let mut board = Board::empty();
        board.set_piece(Square::E7, Piece::new(PieceType::Pawn, Color::White));
        board.set_side_to_move(Color::White);

        let moves = generate_moves(&board);

        // Should generate 4 promotion moves (Q, R, B, N)
        assert_eq!(moves.len(), 4);
        assert!(moves.iter().all(|m| m.is_promotion()));

        // Check all 4 promotion types are present
        assert!(moves
            .iter()
            .any(|m| m.promotion_piece() == Some(PieceType::Queen)));
        assert!(moves
            .iter()
            .any(|m| m.promotion_piece() == Some(PieceType::Rook)));
        assert!(moves
            .iter()
            .any(|m| m.promotion_piece() == Some(PieceType::Bishop)));
        assert!(moves
            .iter()
            .any(|m| m.promotion_piece() == Some(PieceType::Knight)));
    }

    #[test]
    fn test_promotion_captures() {
        let mut board = Board::empty();
        board.set_piece(Square::E7, Piece::new(PieceType::Pawn, Color::White));
        board.set_piece(Square::D8, Piece::new(PieceType::Rook, Color::Black));
        board.set_side_to_move(Color::White);

        let moves = generate_moves(&board);

        // Should generate 4 quiet promotions + 4 capture promotions = 8 total
        assert_eq!(moves.len(), 8);

        let promotion_captures = moves
            .iter()
            .filter(|m| m.is_promotion() && m.is_capture())
            .count();
        assert_eq!(promotion_captures, 4);
    }

    #[test]
    fn test_castling_kingside_white() {
        use crate::board::CastlingRights;

        let mut board = Board::empty();
        board.set_piece(Square::E1, Piece::new(PieceType::King, Color::White));
        board.set_piece(Square::H1, Piece::new(PieceType::Rook, Color::White));
        board.set_side_to_move(Color::White);
        board.set_castling(CastlingRights::none().set_white_kingside());

        let moves = generate_moves(&board);

        // Should have king moves + castling
        let castling_moves = moves.iter().filter(|m| m.is_castling()).count();
        assert_eq!(castling_moves, 1);

        let castle_move = moves.iter().find(|m| m.is_castling()).unwrap();
        assert_eq!(castle_move.from(), Square::E1);
        assert_eq!(castle_move.to(), Square::G1);
        assert!(castle_move.is_kingside_castle());
    }

    #[test]
    fn test_castling_queenside_white() {
        use crate::board::CastlingRights;

        let mut board = Board::empty();
        board.set_piece(Square::E1, Piece::new(PieceType::King, Color::White));
        board.set_piece(Square::A1, Piece::new(PieceType::Rook, Color::White));
        board.set_side_to_move(Color::White);
        board.set_castling(CastlingRights::none().set_white_queenside());

        let moves = generate_moves(&board);

        // Should have king moves + castling
        let castling_moves = moves.iter().filter(|m| m.is_castling()).count();
        assert_eq!(castling_moves, 1);

        let castle_move = moves.iter().find(|m| m.is_castling()).unwrap();
        assert_eq!(castle_move.from(), Square::E1);
        assert_eq!(castle_move.to(), Square::C1);
        assert!(castle_move.is_queenside_castle());
    }

    #[test]
    fn test_castling_blocked() {
        use crate::board::CastlingRights;

        let mut board = Board::empty();
        board.set_piece(Square::E1, Piece::new(PieceType::King, Color::White));
        board.set_piece(Square::H1, Piece::new(PieceType::Rook, Color::White));
        board.set_piece(Square::F1, Piece::new(PieceType::Bishop, Color::White)); // Blocker
        board.set_side_to_move(Color::White);
        board.set_castling(CastlingRights::none().set_white_kingside());

        let moves = generate_moves(&board);

        // Should not generate castling move due to blocker
        let castling_moves = moves.iter().filter(|m| m.is_castling()).count();
        assert_eq!(castling_moves, 0);
    }

    #[test]
    fn test_castling_no_rights() {
        let mut board = Board::empty();
        board.set_piece(Square::E1, Piece::new(PieceType::King, Color::White));
        board.set_piece(Square::H1, Piece::new(PieceType::Rook, Color::White));
        board.set_side_to_move(Color::White);
        // No castling rights set

        let moves = generate_moves(&board);

        // Should not generate castling move without rights
        let castling_moves = moves.iter().filter(|m| m.is_castling()).count();
        assert_eq!(castling_moves, 0);
    }
}
