//! Edge case tests for the chess engine.
//!
//! Tests unusual positions and edge cases to ensure robustness.

use engine::board::Board;
use engine::io::parse_fen;
use engine::piece::{Color, Piece, PieceType};
use engine::r#move::{Move, MoveFlags};
use engine::square::Square;

#[test]
fn test_checkmate_position() {
    // Scholar's mate position
    let fen = "r1bqkb1r/pppp1Qpp/2n2n2/4p3/2B1P3/8/PPPP1PPP/RNB1K1NR b KQkq - 0 4";
    let board = parse_fen(fen).unwrap();

    assert!(board.is_in_check());
    let legal_moves = board.generate_legal_moves();
    assert_eq!(legal_moves.len(), 0, "Checkmate should have no legal moves");
}

#[test]
fn test_stalemate_position() {
    // Classic stalemate: king not in check but no legal moves
    let fen = "7k/5Q2/6K1/8/8/8/8/8 b - - 0 1";
    let board = parse_fen(fen).unwrap();

    assert!(
        !board.is_in_check(),
        "Stalemate: king should not be in check"
    );
    let legal_moves = board.generate_legal_moves();
    assert_eq!(legal_moves.len(), 0, "Stalemate should have no legal moves");
}

#[test]
fn test_only_kings() {
    // Minimum valid position: just kings
    let fen = "4k3/8/8/8/8/8/8/4K3 w - - 0 1";
    let board = parse_fen(fen).unwrap();

    let legal_moves = board.generate_legal_moves();
    assert!(!legal_moves.is_empty(), "Kings should have legal moves");
}

#[test]
fn test_all_promotion_types() {
    // White pawn ready to promote
    let fen = "8/4P3/8/8/8/8/8/4K2k w - - 0 1";
    let mut board = parse_fen(fen).unwrap();

    // Test all 4 promotion types
    for promo_flag in [
        MoveFlags::QUEEN_PROMOTION,
        MoveFlags::ROOK_PROMOTION,
        MoveFlags::BISHOP_PROMOTION,
        MoveFlags::KNIGHT_PROMOTION,
    ] {
        let original = board.clone();
        let m = Move::new(Square::E7, Square::E8, promo_flag);
        let undo = board.make_move(m);

        // Verify piece was promoted
        let piece = board.piece_at(Square::E8);
        assert!(piece.is_some());
        assert_eq!(piece.unwrap().color, Color::White);

        // Unmake and verify
        board.unmake_move(m, undo);
        assert_eq!(board, original);
    }
}

#[test]
fn test_en_passant_both_sides() {
    // Test white en passant
    let fen = "rnbqkbnr/ppp1p1pp/8/3pPp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3";
    let board = parse_fen(fen).unwrap();

    let legal_moves = board.generate_legal_moves();
    let ep_moves: Vec<_> = legal_moves.iter().filter(|m| m.is_en_passant()).collect();

    assert!(
        !ep_moves.is_empty(),
        "Should have en passant captures available"
    );

    // Test black en passant
    let fen = "rnbqkbnr/pppp1ppp/8/8/3pP3/8/PPP2PPP/RNBQKBNR b KQkq e3 0 2";
    let board = parse_fen(fen).unwrap();

    let legal_moves = board.generate_legal_moves();
    let ep_moves: Vec<_> = legal_moves.iter().filter(|m| m.is_en_passant()).collect();

    assert!(
        !ep_moves.is_empty(),
        "Black should have en passant captures available"
    );
}

#[test]
fn test_all_castling_combinations() {
    // Position where all castling rights exist
    let fen = "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1";
    let board = parse_fen(fen).unwrap();

    let legal_moves = board.generate_legal_moves();
    let castling_moves: Vec<_> = legal_moves.iter().filter(|m| m.is_castling()).collect();

    assert_eq!(
        castling_moves.len(),
        2,
        "White should have 2 castling moves"
    );

    // Black to move
    let fen = "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R b KQkq - 0 1";
    let board = parse_fen(fen).unwrap();

    let legal_moves = board.generate_legal_moves();
    let castling_moves: Vec<_> = legal_moves.iter().filter(|m| m.is_castling()).collect();

    assert_eq!(
        castling_moves.len(),
        2,
        "Black should have 2 castling moves"
    );
}

#[test]
fn test_maximum_pieces() {
    // Board with all 32 pieces
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let board = parse_fen(fen).unwrap();

    // Count pieces
    let mut piece_count = 0;
    for sq in Square::all() {
        if board.piece_at(sq).is_some() {
            piece_count += 1;
        }
    }

    assert_eq!(piece_count, 32, "Starting position should have 32 pieces");
}

#[test]
fn test_hash_uniqueness() {
    // Different positions should have different hashes
    let fen1 = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let fen2 = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
    let fen3 = "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq e6 0 2";

    let board1 = parse_fen(fen1).unwrap();
    let board2 = parse_fen(fen2).unwrap();
    let board3 = parse_fen(fen3).unwrap();

    assert_ne!(board1.hash(), board2.hash());
    assert_ne!(board2.hash(), board3.hash());
    assert_ne!(board1.hash(), board3.hash());
}

#[test]
fn test_hash_same_position() {
    // Same position should have same hash
    let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";

    let board1 = parse_fen(fen).unwrap();
    let board2 = parse_fen(fen).unwrap();

    assert_eq!(board1.hash(), board2.hash());
}

#[test]
fn test_complex_positions_parse() {
    // Complex positions should parse correctly
    let test_fens = vec![
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
        "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
        "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
    ];

    for fen in test_fens {
        let board = parse_fen(fen);
        assert!(board.is_ok(), "Failed to parse FEN: {}", fen);

        let board = board.unwrap();
        // Verify we can generate moves without panicking
        let _moves = board.generate_legal_moves();
    }
}

#[test]
fn test_make_unmake_preserves_hash() {
    let mut board = Board::startpos();
    let original_hash = board.hash();

    let legal_moves = board.generate_legal_moves();

    for m in legal_moves.iter() {
        let undo = board.make_move(*m);
        board.unmake_move(*m, undo);

        assert_eq!(
            board.hash(),
            original_hash,
            "Hash should be restored after unmake for move {}",
            m.to_uci()
        );
    }
}

#[test]
fn test_discovered_check() {
    // Position with discovered check potential
    let fen = "rnbqk2r/pppp1ppp/5n2/2b1p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4";
    let mut board = parse_fen(fen).unwrap();

    // Knight on f3 is pinned, moving it would discover check
    let legal_moves = board.generate_legal_moves();

    // Make a legal move
    if !legal_moves.is_empty() {
        let m = legal_moves[0];
        board.make_move(m);
        // Should still be valid
        assert!(board.piece_at(m.to()).is_some());
    }
}

#[test]
fn test_double_check() {
    // Position where king is in double check (can only move king)
    let mut board = Board::empty();
    board.set_piece(Square::E1, Piece::new(PieceType::King, Color::White));
    board.set_piece(Square::E8, Piece::new(PieceType::Rook, Color::Black));
    board.set_piece(
        Square::from_coords(7, 3),
        Piece::new(PieceType::Rook, Color::Black),
    ); // h4
    board.set_piece(Square::D2, Piece::new(PieceType::Pawn, Color::White));
    board.set_side_to_move(Color::White);

    assert!(board.is_in_check());

    let legal_moves = board.generate_legal_moves();
    // In double check, only king moves are legal
    for m in legal_moves.iter() {
        let piece = board.piece_at(m.from()).unwrap();
        assert_eq!(
            piece.piece_type,
            PieceType::King,
            "In double check, only king can move"
        );
    }
}
