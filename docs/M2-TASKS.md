# M2: Engine Core - Detailed Task Breakdown

**Milestone:** M2 — Engine Core: Rules and Move Generation

**Duration:** 3 weeks (15 working days)

**Goal:** Production-ready board representation, move generation, and validation

---

## Quick Reference

### Success Criteria (DoD)

- [ ] Perft depth 1-6 matches canonical values
- [ ] Performance: ≥3M nodes/s single-threaded
- [ ] No panics under fuzz testing
- [ ] All tests passing in CI
- [ ] Code documented and lint-clean

### Key Deliverables

1. Bitboard-based board representation
2. Complete move generation (all pieces + special moves)
3. FEN parser and serializer
4. Perft test harness
5. Zobrist hashing for position uniqueness

---

## Week 1: Foundation (Days 1-5)

### Day 1: Project Setup & Basic Types

**Goal:** Set up module structure and basic data types

#### Tasks

**1.1 Create module structure** (30 min)

```bash
# Create new files
touch crates/engine/src/bitboard.rs
touch crates/engine/src/square.rs
touch crates/engine/src/piece.rs
touch crates/engine/src/color.rs
touch crates/engine/src/castling.rs
```

**1.2 Define Square enum** (45 min)

```rust
// crates/engine/src/square.rs
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum Square {
    A1, B1, C1, D1, E1, F1, G1, H1,
    A2, B2, C2, D2, E2, F2, G2, H2,
    // ... up to H8
}

impl Square {
    pub const fn index(self) -> usize { self as usize }
    pub const fn rank(self) -> u8 { self as u8 / 8 }
    pub const fn file(self) -> u8 { self as u8 % 8 }
    pub const fn from_coords(file: u8, rank: u8) -> Self { /* ... */ }
}
```

**1.3 Define Piece and Color enums** (30 min)

```rust
// crates/engine/src/piece.rs
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Piece {
    Pawn, Knight, Bishop, Rook, Queen, King,
}

// crates/engine/src/color.rs
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Color {
    White, Black,
}

impl Color {
    pub fn opposite(self) -> Self { /* ... */ }
}
```

**1.4 Define Bitboard wrapper** (1 hour)

```rust
// crates/engine/src/bitboard.rs
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Bitboard(pub u64);

impl Bitboard {
    pub const EMPTY: Self = Bitboard(0);
    pub const ALL: Self = Bitboard(0xFFFF_FFFF_FFFF_FFFF);

    pub fn is_empty(self) -> bool { self.0 == 0 }
    pub fn is_set(self, sq: Square) -> bool { /* ... */ }
    pub fn set(&mut self, sq: Square) { /* ... */ }
    pub fn clear(&mut self, sq: Square) { /* ... */ }
    pub fn toggle(&mut self, sq: Square) { /* ... */ }
    pub fn count(self) -> u32 { self.0.count_ones() }

    // Bit scanning
    pub fn lsb(self) -> Option<Square> { /* ... */ }
    pub fn pop_lsb(&mut self) -> Option<Square> { /* ... */ }
}

// Implement BitOr, BitAnd, BitXor, Not
```

**1.5 Write basic tests** (1 hour)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn square_to_index() {
        assert_eq!(Square::A1.index(), 0);
        assert_eq!(Square::H8.index(), 63);
    }

    #[test]
    fn bitboard_set_clear() {
        let mut bb = Bitboard::EMPTY;
        bb.set(Square::E4);
        assert!(bb.is_set(Square::E4));
        bb.clear(Square::E4);
        assert!(bb.is_empty());
    }
}
```

**Time Check:** 4 hours

---

### Day 2: Board Representation

**Goal:** Implement the main Board struct with piece placement

#### Tasks

**2.1 Define Board struct** (1 hour)

```rust
// crates/engine/src/board.rs
use crate::{Bitboard, Piece, Color, Square};

pub struct Board {
    // Piece bitboards (one per piece type per color)
    pieces: [[Bitboard; 6]; 2], // [Color][Piece]

    // Occupancy bitboards (all pieces per color)
    occupancy: [Bitboard; 2], // [Color]

    // Game state
    side_to_move: Color,
    castling_rights: CastlingRights,
    en_passant_square: Option<Square>,
    halfmove_clock: u32,
    fullmove_number: u32,

    // Zobrist hash for position uniqueness
    hash: u64,
}

impl Board {
    pub fn new() -> Self { /* empty board */ }
    pub fn startpos() -> Self { /* starting position */ }
}
```

**2.2 Implement piece placement accessors** (1.5 hours)

```rust
impl Board {
    pub fn piece_at(&self, sq: Square) -> Option<(Piece, Color)> {
        for color in [Color::White, Color::Black] {
            for piece in [Piece::Pawn, Piece::Knight, /* ... */] {
                if self.pieces[color as usize][piece as usize].is_set(sq) {
                    return Some((piece, color));
                }
            }
        }
        None
    }

    pub fn set_piece(&mut self, sq: Square, piece: Piece, color: Color) {
        self.pieces[color as usize][piece as usize].set(sq);
        self.occupancy[color as usize].set(sq);
    }

    pub fn remove_piece(&mut self, sq: Square) -> Option<(Piece, Color)> {
        let piece_color = self.piece_at(sq)?;
        // Remove from bitboards
        // ...
        Some(piece_color)
    }

    pub fn all_occupancy(&self) -> Bitboard {
        self.occupancy[0] | self.occupancy[1]
    }
}
```

**2.3 Implement starting position** (1 hour)

```rust
impl Board {
    pub fn startpos() -> Self {
        let mut board = Self::new();

        // White pieces
        board.set_piece(Square::A1, Piece::Rook, Color::White);
        board.set_piece(Square::B1, Piece::Knight, Color::White);
        // ... all 16 white pieces

        // Black pieces
        board.set_piece(Square::A8, Piece::Rook, Color::Black);
        // ... all 16 black pieces

        board.side_to_move = Color::White;
        board.castling_rights = CastlingRights::ALL;
        board.en_passant_square = None;
        board.halfmove_clock = 0;
        board.fullmove_number = 1;

        board
    }
}
```

**2.4 Write board tests** (30 min)

```rust
#[test]
fn startpos_piece_placement() {
    let board = Board::startpos();
    assert_eq!(board.piece_at(Square::E1), Some((Piece::King, Color::White)));
    assert_eq!(board.piece_at(Square::E8), Some((Piece::King, Color::Black)));
    assert_eq!(board.piece_at(Square::E4), None);
}
```

**Time Check:** 4 hours

---

### Day 3: Move Structure & Castling Rights

**Goal:** Define Move type and castling logic

#### Tasks

**3.1 Define Move struct** (1 hour)

```rust
// crates/engine/src/move.rs
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Move {
    data: u16, // Packed: from(6) | to(6) | flags(4)
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MoveFlag {
    Quiet = 0,
    DoublePawnPush = 1,
    KingCastle = 2,
    QueenCastle = 3,
    Capture = 4,
    EnPassant = 5,
    PromoteKnight = 8,
    PromoteBishop = 9,
    PromoteRook = 10,
    PromoteQueen = 11,
    // Capture promotions: 12-15
}

impl Move {
    pub fn new(from: Square, to: Square, flag: MoveFlag) -> Self { /* ... */ }
    pub fn from(self) -> Square { /* ... */ }
    pub fn to(self) -> Square { /* ... */ }
    pub fn flag(self) -> MoveFlag { /* ... */ }
    pub fn is_capture(self) -> bool { /* ... */ }
    pub fn is_promotion(self) -> bool { /* ... */ }
}
```

**3.2 Define CastlingRights** (45 min)

```rust
// crates/engine/src/castling.rs
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct CastlingRights(u8);

impl CastlingRights {
    pub const NONE: Self = CastlingRights(0);
    pub const WHITE_KING: Self = CastlingRights(1);
    pub const WHITE_QUEEN: Self = CastlingRights(2);
    pub const BLACK_KING: Self = CastlingRights(4);
    pub const BLACK_QUEEN: Self = CastlingRights(8);
    pub const ALL: Self = CastlingRights(15);

    pub fn has(self, right: Self) -> bool { self.0 & right.0 != 0 }
    pub fn add(&mut self, right: Self) { self.0 |= right.0; }
    pub fn remove(&mut self, right: Self) { self.0 &= !right.0; }
}
```

**3.3 Implement castling logic in Board** (1.5 hours)

```rust
impl Board {
    pub fn can_castle_kingside(&self, color: Color) -> bool {
        let right = match color {
            Color::White => CastlingRights::WHITE_KING,
            Color::Black => CastlingRights::BLACK_KING,
        };

        if !self.castling_rights.has(right) {
            return false;
        }

        // Check squares are empty
        // Check king not in check
        // Check path not attacked
        // ...
    }

    pub fn can_castle_queenside(&self, color: Color) -> bool {
        // Similar logic
    }
}
```

**3.4 Write Move tests** (30 min)

```rust
#[test]
fn move_packing() {
    let m = Move::new(Square::E2, Square::E4, MoveFlag::DoublePawnPush);
    assert_eq!(m.from(), Square::E2);
    assert_eq!(m.to(), Square::E4);
    assert_eq!(m.flag(), MoveFlag::DoublePawnPush);
}
```

**Time Check:** 4 hours

---

### Day 4: Attack Maps (Preparation for Move Gen)

**Goal:** Implement attack/control square calculation

#### Tasks

**4.1 Create attack map module** (30 min)

```bash
touch crates/engine/src/attacks.rs
```

**4.2 Implement pawn attacks** (45 min)

```rust
// crates/engine/src/attacks.rs
pub struct Attacks;

impl Attacks {
    // Precomputed tables (lazy_static or const arrays)
    pub fn pawn_attacks(sq: Square, color: Color) -> Bitboard {
        // Lookup from precomputed table
        PAWN_ATTACKS[color as usize][sq.index()]
    }
}

// Precompute at compile time or startup
fn init_pawn_attacks() -> [[Bitboard; 64]; 2] {
    let mut attacks = [[Bitboard::EMPTY; 64]; 2];

    for sq in 0..64 {
        let file = sq % 8;
        let rank = sq / 8;

        // White attacks
        if rank < 7 {
            if file > 0 { attacks[0][sq].set(Square::from_coords(file - 1, rank + 1)); }
            if file < 7 { attacks[0][sq].set(Square::from_coords(file + 1, rank + 1)); }
        }

        // Black attacks (similar)
    }

    attacks
}
```

**4.3 Implement knight attacks** (1 hour)

```rust
impl Attacks {
    pub fn knight_attacks(sq: Square) -> Bitboard {
        KNIGHT_ATTACKS[sq.index()]
    }
}

fn init_knight_attacks() -> [Bitboard; 64] {
    let mut attacks = [Bitboard::EMPTY; 64];

    let deltas = [
        (-2, -1), (-2, 1), (-1, -2), (-1, 2),
        (1, -2), (1, 2), (2, -1), (2, 1),
    ];

    for sq in 0..64 {
        let (file, rank) = (sq % 8, sq / 8);

        for (df, dr) in deltas {
            let new_file = file as i8 + df;
            let new_rank = rank as i8 + dr;

            if new_file >= 0 && new_file < 8 && new_rank >= 0 && new_rank < 8 {
                attacks[sq].set(Square::from_coords(new_file as u8, new_rank as u8));
            }
        }
    }

    attacks
}
```

**4.4 Implement king attacks** (30 min)

```rust
impl Attacks {
    pub fn king_attacks(sq: Square) -> Bitboard {
        KING_ATTACKS[sq.index()]
    }
}

// Similar to knight, but with 8 directions
```

**4.5 Implement sliding piece attacks (bishop/rook/queen)** (1.5 hours)

```rust
impl Attacks {
    // Magic bitboards or classical approach
    pub fn bishop_attacks(sq: Square, occupancy: Bitboard) -> Bitboard {
        // Diagonal and anti-diagonal rays
        // Stop at first blocker in each direction
        ray_attacks(sq, occupancy, BISHOP_DIRECTIONS)
    }

    pub fn rook_attacks(sq: Square, occupancy: Bitboard) -> Bitboard {
        // Horizontal and vertical rays
        ray_attacks(sq, occupancy, ROOK_DIRECTIONS)
    }

    pub fn queen_attacks(sq: Square, occupancy: Bitboard) -> Bitboard {
        bishop_attacks(sq, occupancy) | rook_attacks(sq, occupancy)
    }
}

fn ray_attacks(sq: Square, occupancy: Bitboard, directions: &[(i8, i8)]) -> Bitboard {
    let mut attacks = Bitboard::EMPTY;

    for &(df, dr) in directions {
        let mut file = sq.file() as i8;
        let mut rank = sq.rank() as i8;

        loop {
            file += df;
            rank += dr;

            if file < 0 || file > 7 || rank < 0 || rank > 7 {
                break;
            }

            let target = Square::from_coords(file as u8, rank as u8);
            attacks.set(target);

            if occupancy.is_set(target) {
                break; // Blocker found
            }
        }
    }

    attacks
}
```

**4.6 Write attack tests** (30 min)

```rust
#[test]
fn knight_attacks_center() {
    let attacks = Attacks::knight_attacks(Square::E4);
    assert_eq!(attacks.count(), 8); // Knight in center has 8 moves
}

#[test]
fn rook_attacks_blocked() {
    let mut occ = Bitboard::EMPTY;
    occ.set(Square::E6); // Blocker

    let attacks = Attacks::rook_attacks(Square::E4, occ);
    assert!(attacks.is_set(Square::E5));
    assert!(attacks.is_set(Square::E6)); // Can capture blocker
    assert!(!attacks.is_set(Square::E7)); // Blocked
}
```

**Time Check:** 4.5 hours

---

### Day 5: Move Generation - Part 1 (Quiet Moves)

**Goal:** Generate pseudo-legal quiet moves (non-captures)

#### Tasks

**5.1 Create movegen module** (30 min)

```bash
touch crates/engine/src/movegen.rs
touch crates/engine/src/movelist.rs
```

**5.2 Define MoveList (stack-allocated)** (45 min)

```rust
// crates/engine/src/movelist.rs
const MAX_MOVES: usize = 256;

pub struct MoveList {
    moves: [Move; MAX_MOVES],
    len: usize,
}

impl MoveList {
    pub fn new() -> Self {
        Self {
            moves: [Move::new(Square::A1, Square::A1, MoveFlag::Quiet); MAX_MOVES],
            len: 0,
        }
    }

    pub fn push(&mut self, m: Move) {
        self.moves[self.len] = m;
        self.len += 1;
    }

    pub fn len(&self) -> usize { self.len }
    pub fn iter(&self) -> impl Iterator<Item = &Move> {
        self.moves[..self.len].iter()
    }
}
```

**5.3 Implement pawn quiet moves** (1.5 hours)

```rust
// crates/engine/src/movegen.rs
use crate::{Board, MoveList, Move, MoveFlag, Square, Color, Bitboard, Attacks};

pub fn generate_pawn_moves(board: &Board, moves: &mut MoveList) {
    let us = board.side_to_move;
    let our_pawns = board.pieces[us as usize][Piece::Pawn as usize];
    let all_occ = board.all_occupancy();

    let (up, double_rank) = match us {
        Color::White => (8, 1),   // Up = +8, double push from rank 1
        Color::Black => (-8, 6),  // Up = -8, double push from rank 6
    };

    // Single pushes
    let mut pushable = our_pawns;
    while let Some(from) = pushable.pop_lsb() {
        let to_idx = (from.index() as i8 + up) as usize;

        if to_idx >= 64 || all_occ.is_set(Square::from_index(to_idx)) {
            continue;
        }

        let to = Square::from_index(to_idx);

        // Check for promotion
        if to.rank() == 0 || to.rank() == 7 {
            moves.push(Move::new(from, to, MoveFlag::PromoteQueen));
            moves.push(Move::new(from, to, MoveFlag::PromoteRook));
            moves.push(Move::new(from, to, MoveFlag::PromoteBishop));
            moves.push(Move::new(from, to, MoveFlag::PromoteKnight));
        } else {
            moves.push(Move::new(from, to, MoveFlag::Quiet));
        }

        // Double push
        if from.rank() == double_rank {
            let double_to_idx = (from.index() as i8 + 2 * up) as usize;
            let double_to = Square::from_index(double_to_idx);

            if !all_occ.is_set(double_to) {
                moves.push(Move::new(from, double_to, MoveFlag::DoublePawnPush));
            }
        }
    }
}
```

**5.4 Implement knight quiet moves** (45 min)

```rust
pub fn generate_knight_moves(board: &Board, moves: &mut MoveList) {
    let us = board.side_to_move;
    let our_knights = board.pieces[us as usize][Piece::Knight as usize];
    let our_occ = board.occupancy[us as usize];

    let mut knights = our_knights;
    while let Some(from) = knights.pop_lsb() {
        let attacks = Attacks::knight_attacks(from);
        let targets = attacks & !our_occ; // Can't capture our own

        let mut dests = targets;
        while let Some(to) = dests.pop_lsb() {
            let flag = if board.piece_at(to).is_some() {
                MoveFlag::Capture
            } else {
                MoveFlag::Quiet
            };
            moves.push(Move::new(from, to, flag));
        }
    }
}
```

**5.5 Implement king quiet moves** (30 min)

```rust
pub fn generate_king_moves(board: &Board, moves: &mut MoveList) {
    let us = board.side_to_move;
    let our_king = board.pieces[us as usize][Piece::King as usize];
    let our_occ = board.occupancy[us as usize];

    if let Some(from) = our_king.lsb() {
        let attacks = Attacks::king_attacks(from);
        let targets = attacks & !our_occ;

        let mut dests = targets;
        while let Some(to) = dests.pop_lsb() {
            let flag = if board.piece_at(to).is_some() {
                MoveFlag::Capture
            } else {
                MoveFlag::Quiet
            };
            moves.push(Move::new(from, to, flag));
        }
    }
}
```

**5.6 Write movegen tests** (30 min)

```rust
#[test]
fn startpos_pawn_moves() {
    let board = Board::startpos();
    let mut moves = MoveList::new();
    generate_pawn_moves(&board, &mut moves);

    // Each of 8 white pawns can move 1 or 2 squares
    assert_eq!(moves.len(), 16);
}
```

**Time Check:** 4 hours

---

## Week 2: Move Generation & FEN (Days 6-10)

### Day 6: Move Generation - Part 2 (Sliding Pieces)

**Goal:** Complete move generation for bishops, rooks, queens

#### Tasks

**6.1 Implement bishop moves** (1 hour)

```rust
pub fn generate_bishop_moves(board: &Board, moves: &mut MoveList) {
    let us = board.side_to_move;
    let our_bishops = board.pieces[us as usize][Piece::Bishop as usize];
    let our_occ = board.occupancy[us as usize];
    let all_occ = board.all_occupancy();

    let mut bishops = our_bishops;
    while let Some(from) = bishops.pop_lsb() {
        let attacks = Attacks::bishop_attacks(from, all_occ);
        let targets = attacks & !our_occ;

        let mut dests = targets;
        while let Some(to) = dests.pop_lsb() {
            let flag = if board.piece_at(to).is_some() {
                MoveFlag::Capture
            } else {
                MoveFlag::Quiet
            };
            moves.push(Move::new(from, to, flag));
        }
    }
}
```

**6.2 Implement rook and queen moves** (1 hour)

```rust
pub fn generate_rook_moves(board: &Board, moves: &mut MoveList) {
    // Similar to bishop
}

pub fn generate_queen_moves(board: &Board, moves: &mut MoveList) {
    // Similar to bishop + rook
}
```

**6.3 Add castling moves** (1.5 hours)

```rust
pub fn generate_castling_moves(board: &Board, moves: &mut MoveList) {
    let us = board.side_to_move;

    if board.can_castle_kingside(us) {
        let (from, to) = match us {
            Color::White => (Square::E1, Square::G1),
            Color::Black => (Square::E8, Square::G8),
        };
        moves.push(Move::new(from, to, MoveFlag::KingCastle));
    }

    if board.can_castle_queenside(us) {
        let (from, to) = match us {
            Color::White => (Square::E1, Square::C1),
            Color::Black => (Square::E8, Square::C8),
        };
        moves.push(Move::new(from, to, MoveFlag::QueenCastle));
    }
}
```

**6.4 Add en passant captures** (1 hour)

```rust
pub fn generate_pawn_captures(board: &Board, moves: &mut MoveList) {
    let us = board.side_to_move;
    let them = us.opposite();
    let our_pawns = board.pieces[us as usize][Piece::Pawn as usize];
    let their_occ = board.occupancy[them as usize];

    // Regular captures
    let mut pawns = our_pawns;
    while let Some(from) = pawns.pop_lsb() {
        let attacks = Attacks::pawn_attacks(from, us);
        let captures = attacks & their_occ;

        let mut targets = captures;
        while let Some(to) = targets.pop_lsb() {
            if to.rank() == 0 || to.rank() == 7 {
                // Capture promotion
                moves.push(Move::new(from, to, MoveFlag::PromoteQueen | MoveFlag::Capture));
                // ... other promotions
            } else {
                moves.push(Move::new(from, to, MoveFlag::Capture));
            }
        }
    }

    // En passant
    if let Some(ep_sq) = board.en_passant_square {
        let mut pawns = our_pawns;
        while let Some(from) = pawns.pop_lsb() {
            let attacks = Attacks::pawn_attacks(from, us);
            if attacks.is_set(ep_sq) {
                moves.push(Move::new(from, ep_sq, MoveFlag::EnPassant));
            }
        }
    }
}
```

**6.5 Unify move generation** (30 min)

```rust
pub fn generate_all_moves(board: &Board, moves: &mut MoveList) {
    generate_pawn_moves(board, moves);
    generate_pawn_captures(board, moves);
    generate_knight_moves(board, moves);
    generate_bishop_moves(board, moves);
    generate_rook_moves(board, moves);
    generate_queen_moves(board, moves);
    generate_king_moves(board, moves);
    generate_castling_moves(board, moves);
}
```

**Time Check:** 5 hours

---

### Day 7: Make/Unmake Move

**Goal:** Implement move application and reversal

#### Tasks

**7.1 Define UndoInfo struct** (30 min)

```rust
// crates/engine/src/board.rs
#[derive(Debug, Copy, Clone)]
pub struct UndoInfo {
    captured_piece: Option<Piece>,
    castling_rights: CastlingRights,
    en_passant_square: Option<Square>,
    halfmove_clock: u32,
    hash: u64,
}
```

**7.2 Implement make_move** (2 hours)

```rust
impl Board {
    pub fn make_move(&mut self, m: Move) -> UndoInfo {
        let undo = UndoInfo {
            captured_piece: None, // Will be set if capture
            castling_rights: self.castling_rights,
            en_passant_square: self.en_passant_square,
            halfmove_clock: self.halfmove_clock,
            hash: self.hash,
        };

        let from = m.from();
        let to = m.to();
        let flag = m.flag();

        let (piece, color) = self.piece_at(from).unwrap();

        // Remove piece from 'from'
        self.remove_piece(from);

        // Handle captures
        let mut undo = undo;
        if m.is_capture() && flag != MoveFlag::EnPassant {
            if let Some((captured, _)) = self.piece_at(to) {
                undo.captured_piece = Some(captured);
                self.remove_piece(to);
            }
        }

        // Handle special moves
        match flag {
            MoveFlag::DoublePawnPush => {
                // Set en passant square
                let ep_sq = Square::from_coords(from.file(), (from.rank() + to.rank()) / 2);
                self.en_passant_square = Some(ep_sq);
            },
            MoveFlag::EnPassant => {
                // Remove captured pawn
                let captured_sq = Square::from_coords(to.file(), from.rank());
                self.remove_piece(captured_sq);
                undo.captured_piece = Some(Piece::Pawn);
            },
            MoveFlag::KingCastle => {
                // Move rook
                let (rook_from, rook_to) = match color {
                    Color::White => (Square::H1, Square::F1),
                    Color::Black => (Square::H8, Square::F8),
                };
                self.remove_piece(rook_from);
                self.set_piece(rook_to, Piece::Rook, color);
            },
            MoveFlag::QueenCastle => {
                // Move rook
                let (rook_from, rook_to) = match color {
                    Color::White => (Square::A1, Square::D1),
                    Color::Black => (Square::A8, Square::D8),
                };
                self.remove_piece(rook_from);
                self.set_piece(rook_to, Piece::Rook, color);
            },
            flag if flag >= MoveFlag::PromoteKnight => {
                // Handle promotion
                let promo_piece = match flag & 0x3 {
                    0 => Piece::Knight,
                    1 => Piece::Bishop,
                    2 => Piece::Rook,
                    3 => Piece::Queen,
                    _ => unreachable!(),
                };
                self.set_piece(to, promo_piece, color);

                // Update castling rights, clocks, etc.
                self.update_state_after_move(piece, from, to);
                self.side_to_move = self.side_to_move.opposite();

                return undo;
            },
            _ => {}
        }

        // Place piece on 'to'
        self.set_piece(to, piece, color);

        // Update state
        self.update_state_after_move(piece, from, to);
        self.side_to_move = self.side_to_move.opposite();

        undo
    }

    fn update_state_after_move(&mut self, piece: Piece, from: Square, to: Square) {
        // Update castling rights
        match piece {
            Piece::King => {
                if self.side_to_move == Color::White {
                    self.castling_rights.remove(CastlingRights::WHITE_KING);
                    self.castling_rights.remove(CastlingRights::WHITE_QUEEN);
                } else {
                    self.castling_rights.remove(CastlingRights::BLACK_KING);
                    self.castling_rights.remove(CastlingRights::BLACK_QUEEN);
                }
            },
            Piece::Rook => {
                // Remove castling if rook moved from starting square
                if from == Square::A1 {
                    self.castling_rights.remove(CastlingRights::WHITE_QUEEN);
                } else if from == Square::H1 {
                    self.castling_rights.remove(CastlingRights::WHITE_KING);
                } else if from == Square::A8 {
                    self.castling_rights.remove(CastlingRights::BLACK_QUEEN);
                } else if from == Square::H8 {
                    self.castling_rights.remove(CastlingRights::BLACK_KING);
                }
            },
            _ => {}
        }

        // Update halfmove clock
        if piece == Piece::Pawn || self.piece_at(to).is_some() {
            self.halfmove_clock = 0;
        } else {
            self.halfmove_clock += 1;
        }

        // Clear en passant
        self.en_passant_square = None;

        // Update fullmove number
        if self.side_to_move == Color::Black {
            self.fullmove_number += 1;
        }
    }
}
```

**7.3 Implement unmake_move** (1 hour)

```rust
impl Board {
    pub fn unmake_move(&mut self, m: Move, undo: UndoInfo) {
        // Reverse side to move first
        self.side_to_move = self.side_to_move.opposite();

        let from = m.from();
        let to = m.to();
        let flag = m.flag();

        let (piece, color) = self.piece_at(to).unwrap_or_else(|| {
            // For promotions, we need to figure out the original piece was pawn
            (Piece::Pawn, self.side_to_move)
        });

        // Handle promotions
        let moving_piece = if flag >= MoveFlag::PromoteKnight {
            Piece::Pawn
        } else {
            piece
        };

        // Remove piece from 'to'
        self.remove_piece(to);

        // Restore piece to 'from'
        self.set_piece(from, moving_piece, color);

        // Restore captured piece
        if let Some(captured) = undo.captured_piece {
            if flag == MoveFlag::EnPassant {
                let captured_sq = Square::from_coords(to.file(), from.rank());
                self.set_piece(captured_sq, captured, color.opposite());
            } else {
                self.set_piece(to, captured, color.opposite());
            }
        }

        // Undo castling
        if flag == MoveFlag::KingCastle {
            let (rook_from, rook_to) = match color {
                Color::White => (Square::H1, Square::F1),
                Color::Black => (Square::H8, Square::F8),
            };
            self.remove_piece(rook_to);
            self.set_piece(rook_from, Piece::Rook, color);
        } else if flag == MoveFlag::QueenCastle {
            let (rook_from, rook_to) = match color {
                Color::White => (Square::A1, Square::D1),
                Color::Black => (Square::A8, Square::D8),
            };
            self.remove_piece(rook_to);
            self.set_piece(rook_from, Piece::Rook, color);
        }

        // Restore state
        self.castling_rights = undo.castling_rights;
        self.en_passant_square = undo.en_passant_square;
        self.halfmove_clock = undo.halfmove_clock;
        self.hash = undo.hash;
    }
}
```

**7.4 Write make/unmake tests** (30 min)

```rust
#[test]
fn make_unmake_e2e4() {
    let mut board = Board::startpos();
    let m = Move::new(Square::E2, Square::E4, MoveFlag::DoublePawnPush);

    let undo = board.make_move(m);

    assert_eq!(board.piece_at(Square::E2), None);
    assert_eq!(board.piece_at(Square::E4), Some((Piece::Pawn, Color::White)));
    assert_eq!(board.side_to_move, Color::Black);

    board.unmake_move(m, undo);

    assert_eq!(board.piece_at(Square::E2), Some((Piece::Pawn, Color::White)));
    assert_eq!(board.piece_at(Square::E4), None);
    assert_eq!(board.side_to_move, Color::White);
}
```

**Time Check:** 4 hours

---

### Day 8: FEN Parser

**Goal:** Parse and serialize FEN strings

#### Tasks

**8.1 Create FEN module** (30 min)

```bash
touch crates/engine/src/fen.rs
```

**8.2 Implement FEN parser** (2 hours)

```rust
// crates/engine/src/fen.rs
use crate::{Board, Piece, Color, Square, CastlingRights};
use std::str::FromStr;

#[derive(Debug)]
pub enum FenError {
    InvalidFormat,
    InvalidPiece(char),
    InvalidRank,
    InvalidSideToMove,
    InvalidCastling,
    InvalidEnPassant,
}

impl FromStr for Board {
    type Err = FenError;

    fn from_str(fen: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = fen.split_whitespace().collect();

        if parts.len() != 6 {
            return Err(FenError::InvalidFormat);
        }

        let mut board = Board::new();

        // Parse piece placement
        let ranks: Vec<&str> = parts[0].split('/').collect();
        if ranks.len() != 8 {
            return Err(FenError::InvalidRank);
        }

        for (rank_idx, rank_str) in ranks.iter().enumerate() {
            let rank = 7 - rank_idx; // FEN starts from rank 8
            let mut file = 0;

            for ch in rank_str.chars() {
                if ch.is_ascii_digit() {
                    file += ch.to_digit(10).unwrap() as usize;
                } else {
                    let (piece, color) = parse_piece(ch)?;
                    let sq = Square::from_coords(file as u8, rank as u8);
                    board.set_piece(sq, piece, color);
                    file += 1;
                }
            }
        }

        // Parse side to move
        board.side_to_move = match parts[1] {
            "w" => Color::White,
            "b" => Color::Black,
            _ => return Err(FenError::InvalidSideToMove),
        };

        // Parse castling rights
        board.castling_rights = parse_castling(parts[2])?;

        // Parse en passant
        board.en_passant_square = if parts[3] == "-" {
            None
        } else {
            Some(parse_square(parts[3])?)
        };

        // Parse clocks
        board.halfmove_clock = parts[4].parse().unwrap_or(0);
        board.fullmove_number = parts[5].parse().unwrap_or(1);

        Ok(board)
    }
}

fn parse_piece(ch: char) -> Result<(Piece, Color), FenError> {
    let piece = match ch.to_ascii_lowercase() {
        'p' => Piece::Pawn,
        'n' => Piece::Knight,
        'b' => Piece::Bishop,
        'r' => Piece::Rook,
        'q' => Piece::Queen,
        'k' => Piece::King,
        _ => return Err(FenError::InvalidPiece(ch)),
    };

    let color = if ch.is_ascii_uppercase() {
        Color::White
    } else {
        Color::Black
    };

    Ok((piece, color))
}

fn parse_castling(s: &str) -> Result<CastlingRights, FenError> {
    let mut rights = CastlingRights::NONE;

    if s == "-" {
        return Ok(rights);
    }

    for ch in s.chars() {
        match ch {
            'K' => rights.add(CastlingRights::WHITE_KING),
            'Q' => rights.add(CastlingRights::WHITE_QUEEN),
            'k' => rights.add(CastlingRights::BLACK_KING),
            'q' => rights.add(CastlingRights::BLACK_QUEEN),
            _ => return Err(FenError::InvalidCastling),
        }
    }

    Ok(rights)
}

fn parse_square(s: &str) -> Result<Square, FenError> {
    if s.len() != 2 {
        return Err(FenError::InvalidEnPassant);
    }

    let chars: Vec<char> = s.chars().collect();
    let file = (chars[0] as u8) - b'a';
    let rank = (chars[1] as u8) - b'1';

    if file > 7 || rank > 7 {
        return Err(FenError::InvalidEnPassant);
    }

    Ok(Square::from_coords(file, rank))
}
```

**8.3 Implement FEN serializer** (1 hour)

```rust
impl Board {
    pub fn to_fen(&self) -> String {
        let mut fen = String::new();

        // Piece placement
        for rank in (0..8).rev() {
            let mut empty_count = 0;

            for file in 0..8 {
                let sq = Square::from_coords(file, rank);

                if let Some((piece, color)) = self.piece_at(sq) {
                    if empty_count > 0 {
                        fen.push_str(&empty_count.to_string());
                        empty_count = 0;
                    }

                    let ch = match piece {
                        Piece::Pawn => 'p',
                        Piece::Knight => 'n',
                        Piece::Bishop => 'b',
                        Piece::Rook => 'r',
                        Piece::Queen => 'q',
                        Piece::King => 'k',
                    };

                    let ch = if color == Color::White {
                        ch.to_ascii_uppercase()
                    } else {
                        ch
                    };

                    fen.push(ch);
                } else {
                    empty_count += 1;
                }
            }

            if empty_count > 0 {
                fen.push_str(&empty_count.to_string());
            }

            if rank > 0 {
                fen.push('/');
            }
        }

        // Side to move
        fen.push(' ');
        fen.push(if self.side_to_move == Color::White { 'w' } else { 'b' });

        // Castling rights
        fen.push(' ');
        let castling = self.format_castling();
        fen.push_str(&castling);

        // En passant
        fen.push(' ');
        if let Some(sq) = self.en_passant_square {
            fen.push_str(&square_to_algebraic(sq));
        } else {
            fen.push('-');
        }

        // Clocks
        fen.push(' ');
        fen.push_str(&self.halfmove_clock.to_string());
        fen.push(' ');
        fen.push_str(&self.fullmove_number.to_string());

        fen
    }

    fn format_castling(&self) -> String {
        let mut s = String::new();

        if self.castling_rights.has(CastlingRights::WHITE_KING) { s.push('K'); }
        if self.castling_rights.has(CastlingRights::WHITE_QUEEN) { s.push('Q'); }
        if self.castling_rights.has(CastlingRights::BLACK_KING) { s.push('k'); }
        if self.castling_rights.has(CastlingRights::BLACK_QUEEN) { s.push('q'); }

        if s.is_empty() {
            s.push('-');
        }

        s
    }
}

fn square_to_algebraic(sq: Square) -> String {
    let file = (b'a' + sq.file()) as char;
    let rank = (b'1' + sq.rank()) as char;
    format!("{}{}", file, rank)
}
```

**8.4 Write FEN tests** (30 min)

```rust
#[test]
fn parse_startpos_fen() {
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let board = Board::from_str(fen).unwrap();

    assert_eq!(board.piece_at(Square::E1), Some((Piece::King, Color::White)));
    assert_eq!(board.side_to_move, Color::White);
}

#[test]
fn fen_roundtrip() {
    let original = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
    let board = Board::from_str(original).unwrap();
    let serialized = board.to_fen();

    assert_eq!(original, serialized);
}
```

**Time Check:** 4 hours

---

### Day 9: Legality Checking

**Goal:** Filter pseudo-legal moves to only legal moves

#### Tasks

**9.1 Implement is_square_attacked** (1.5 hours)

```rust
// crates/engine/src/board.rs
impl Board {
    pub fn is_square_attacked(&self, sq: Square, by: Color) -> bool {
        let their_occ = self.occupancy[by as usize];
        let all_occ = self.all_occupancy();

        // Pawn attacks
        let pawn_attacks = Attacks::pawn_attacks(sq, by.opposite());
        if !(pawn_attacks & self.pieces[by as usize][Piece::Pawn as usize]).is_empty() {
            return true;
        }

        // Knight attacks
        let knight_attacks = Attacks::knight_attacks(sq);
        if !(knight_attacks & self.pieces[by as usize][Piece::Knight as usize]).is_empty() {
            return true;
        }

        // Bishop/Queen diagonal attacks
        let bishop_attacks = Attacks::bishop_attacks(sq, all_occ);
        let diagonal_attackers = self.pieces[by as usize][Piece::Bishop as usize]
            | self.pieces[by as usize][Piece::Queen as usize];
        if !(bishop_attacks & diagonal_attackers).is_empty() {
            return true;
        }

        // Rook/Queen straight attacks
        let rook_attacks = Attacks::rook_attacks(sq, all_occ);
        let straight_attackers = self.pieces[by as usize][Piece::Rook as usize]
            | self.pieces[by as usize][Piece::Queen as usize];
        if !(rook_attacks & straight_attackers).is_empty() {
            return true;
        }

        // King attacks
        let king_attacks = Attacks::king_attacks(sq);
        if !(king_attacks & self.pieces[by as usize][Piece::King as usize]).is_empty() {
            return true;
        }

        false
    }

    pub fn is_in_check(&self, color: Color) -> bool {
        if let Some(king_sq) = self.pieces[color as usize][Piece::King as usize].lsb() {
            self.is_square_attacked(king_sq, color.opposite())
        } else {
            false
        }
    }
}
```

**9.2 Implement is_legal** (1 hour)

```rust
impl Board {
    pub fn is_legal(&mut self, m: Move) -> bool {
        let us = self.side_to_move;

        // Make the move
        let undo = self.make_move(m);

        // Check if our king is in check (illegal)
        let legal = !self.is_in_check(us);

        // Unmake the move
        self.unmake_move(m, undo);

        legal
    }
}
```

**9.3 Implement generate_legal_moves** (1 hour)

```rust
pub fn generate_legal_moves(board: &mut Board, moves: &mut MoveList) {
    let mut pseudo_legal = MoveList::new();
    generate_all_moves(board, &mut pseudo_legal);

    for &m in pseudo_legal.iter() {
        if board.is_legal(m) {
            moves.push(m);
        }
    }
}
```

**9.4 Write legality tests** (30 min)

```rust
#[test]
fn illegal_move_into_check() {
    // Position where moving king into check is illegal
    let fen = "4k3/8/8/8/8/8/8/4K2R w K - 0 1"; // King on e1, Rook on h1
    let mut board = Board::from_str(fen).unwrap();

    let m = Move::new(Square::E1, Square::E2, MoveFlag::Quiet);

    // Simulate black rook on e8 attacking e2
    // The move should be legal in this position, adjust test accordingly
}

#[test]
fn castling_through_check_illegal() {
    // Test that can't castle through check
}
```

**Time Check:** 4 hours

---

### Day 10: Zobrist Hashing

**Goal:** Implement incremental position hashing

#### Tasks

**10.1 Create zobrist module** (30 min)

```bash
touch crates/engine/src/zobrist.rs
```

**10.2 Generate Zobrist keys** (1 hour)

```rust
// crates/engine/src/zobrist.rs
use crate::{Square, Piece, Color, CastlingRights};
use once_cell::sync::Lazy;

pub struct ZobristKeys {
    pieces: [[[u64; 64]; 6]; 2], // [color][piece][square]
    castling: [u64; 16],          // [castling rights]
    en_passant: [u64; 8],         // [file]
    side_to_move: u64,
}

impl ZobristKeys {
    fn new() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let mut keys = ZobristKeys {
            pieces: [[[0; 64]; 6]; 2],
            castling: [0; 16],
            en_passant: [0; 8],
            side_to_move: 0,
        };

        // Generate random keys
        for color in 0..2 {
            for piece in 0..6 {
                for sq in 0..64 {
                    keys.pieces[color][piece][sq] = rng.gen();
                }
            }
        }

        for i in 0..16 {
            keys.castling[i] = rng.gen();
        }

        for i in 0..8 {
            keys.en_passant[i] = rng.gen();
        }

        keys.side_to_move = rng.gen();

        keys
    }
}

pub static ZOBRIST: Lazy<ZobristKeys> = Lazy::new(|| ZobristKeys::new());
```

**10.3 Implement hash calculation** (1.5 hours)

```rust
// crates/engine/src/board.rs
impl Board {
    pub fn calculate_hash(&self) -> u64 {
        let mut hash = 0u64;

        // Hash pieces
        for color in [Color::White, Color::Black] {
            for piece in [Piece::Pawn, Piece::Knight, Piece::Bishop,
                          Piece::Rook, Piece::Queen, Piece::King] {
                let mut bb = self.pieces[color as usize][piece as usize];
                while let Some(sq) = bb.pop_lsb() {
                    hash ^= ZOBRIST.pieces[color as usize][piece as usize][sq.index()];
                }
            }
        }

        // Hash castling rights
        hash ^= ZOBRIST.castling[self.castling_rights.0 as usize];

        // Hash en passant
        if let Some(ep_sq) = self.en_passant_square {
            hash ^= ZOBRIST.en_passant[ep_sq.file() as usize];
        }

        // Hash side to move
        if self.side_to_move == Color::Black {
            hash ^= ZOBRIST.side_to_move;
        }

        hash
    }

    // Update hash incrementally in make_move/unmake_move
    fn update_hash_move(&mut self, m: Move, piece: Piece) {
        let from = m.from();
        let to = m.to();
        let color = self.side_to_move;

        // XOR out piece at from
        self.hash ^= ZOBRIST.pieces[color as usize][piece as usize][from.index()];

        // XOR in piece at to
        self.hash ^= ZOBRIST.pieces[color as usize][piece as usize][to.index()];

        // Toggle side to move
        self.hash ^= ZOBRIST.side_to_move;
    }
}
```

**10.4 Write hash tests** (30 min)

```rust
#[test]
fn hash_consistency() {
    let board = Board::startpos();
    let hash1 = board.hash;
    let hash2 = board.calculate_hash();

    assert_eq!(hash1, hash2);
}

#[test]
fn hash_incremental_update() {
    let mut board = Board::startpos();
    let m = Move::new(Square::E2, Square::E4, MoveFlag::DoublePawnPush);

    let undo = board.make_move(m);
    let hash_after_move = board.hash;

    board.unmake_move(m, undo);
    let hash_after_unmake = board.hash;

    // Hash should be restored after unmake
    assert_eq!(hash_after_unmake, Board::startpos().hash);
}
```

**10.5 Add Cargo dependency** (5 min)

```toml
# crates/engine/Cargo.toml
[dependencies]
rand = "0.8"
once_cell = "1.19"
```

**Time Check:** 3.5 hours

---

## Week 3: Perft & Polish (Days 11-15)

### Day 11: Perft Implementation

**Goal:** Build perft testing harness

#### Tasks

**11.1 Create perft module** (30 min)

```bash
mkdir -p crates/engine/tests
touch crates/engine/tests/perft.rs
```

**11.2 Implement perft function** (1 hour)

```rust
// crates/engine/src/lib.rs (add perft module)
pub mod perft;

// crates/engine/src/perft.rs
use crate::{Board, movegen::generate_legal_moves, MoveList};

pub fn perft(board: &mut Board, depth: u32) -> u64 {
    if depth == 0 {
        return 1;
    }

    let mut moves = MoveList::new();
    generate_legal_moves(board, &mut moves);

    if depth == 1 {
        return moves.len() as u64;
    }

    let mut nodes = 0u64;

    for &m in moves.iter() {
        let undo = board.make_move(m);
        nodes += perft(board, depth - 1);
        board.unmake_move(m, undo);
    }

    nodes
}

pub fn perft_divide(board: &mut Board, depth: u32) {
    let mut moves = MoveList::new();
    generate_legal_moves(board, &mut moves);

    let mut total = 0u64;

    for &m in moves.iter() {
        let undo = board.make_move(m);
        let nodes = perft(board, depth - 1);
        board.unmake_move(m, undo);

        println!("{}{}: {}",
            square_to_algebraic(m.from()),
            square_to_algebraic(m.to()),
            nodes
        );

        total += nodes;
    }

    println!("\nTotal nodes: {}", total);
}
```

**11.3 Add canonical perft test positions** (1.5 hours)

```rust
// crates/engine/tests/perft.rs
use engine::{Board, perft::perft};
use std::str::FromStr;

#[test]
fn perft_startpos_depth_1() {
    let mut board = Board::startpos();
    assert_eq!(perft(&mut board, 1), 20);
}

#[test]
fn perft_startpos_depth_2() {
    let mut board = Board::startpos();
    assert_eq!(perft(&mut board, 2), 400);
}

#[test]
fn perft_startpos_depth_3() {
    let mut board = Board::startpos();
    assert_eq!(perft(&mut board, 3), 8_902);
}

#[test]
fn perft_startpos_depth_4() {
    let mut board = Board::startpos();
    assert_eq!(perft(&mut board, 4), 197_281);
}

#[test]
fn perft_startpos_depth_5() {
    let mut board = Board::startpos();
    assert_eq!(perft(&mut board, 5), 4_865_609);
}

#[test]
#[ignore] // Run with --ignored flag
fn perft_startpos_depth_6() {
    let mut board = Board::startpos();
    assert_eq!(perft(&mut board, 6), 119_060_324);
}

#[test]
fn perft_kiwipete_depth_1() {
    // Position 2: Kiwipete
    let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
    let mut board = Board::from_str(fen).unwrap();
    assert_eq!(perft(&mut board, 1), 48);
}

#[test]
fn perft_kiwipete_depth_2() {
    let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
    let mut board = Board::from_str(fen).unwrap();
    assert_eq!(perft(&mut board, 2), 2_039);
}

#[test]
fn perft_kiwipete_depth_3() {
    let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
    let mut board = Board::from_str(fen).unwrap();
    assert_eq!(perft(&mut board, 3), 97_862);
}

#[test]
#[ignore]
fn perft_kiwipete_depth_4() {
    let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
    let mut board = Board::from_str(fen).unwrap();
    assert_eq!(perft(&mut board, 4), 4_085_603);
}

// Add more positions: position 3, 4, 5 from standard perft suite
```

**11.4 Run perft tests** (1 hour - debugging time)

```bash
cargo test perft -- --test-threads=1
cargo test perft_startpos_depth_6 -- --ignored
```

**Time Check:** 4 hours (includes debugging)

---

### Day 12: Performance Benchmarking

**Goal:** Measure and optimize performance

#### Tasks

**12.1 Add Criterion benchmark** (30 min)

```toml
# crates/engine/Cargo.toml
[dev-dependencies]
criterion = "0.5"

[[bench]]
name = "perft_bench"
harness = false
```

**12.2 Create benchmark file** (1 hour)

```rust
// crates/engine/benches/perft_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use engine::{Board, perft::perft};
use std::str::FromStr;

fn perft_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("perft");

    // Startpos depth 5
    group.bench_function("startpos_depth_5", |b| {
        b.iter(|| {
            let mut board = Board::startpos();
            perft(black_box(&mut board), 5)
        });
    });

    // Kiwipete depth 3
    group.bench_function("kiwipete_depth_3", |b| {
        let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
        let board = Board::from_str(fen).unwrap();

        b.iter(|| {
            let mut b = board.clone();
            perft(black_box(&mut b), 3)
        });
    });

    group.finish();
}

fn movegen_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("movegen");

    group.bench_function("startpos", |b| {
        let board = Board::startpos();

        b.iter(|| {
            let mut moves = MoveList::new();
            generate_legal_moves(black_box(&board), &mut moves);
            moves.len()
        });
    });

    group.finish();
}

criterion_group!(benches, perft_benchmark, movegen_benchmark);
criterion_main!(benches);
```

**12.3 Run benchmarks** (30 min)

```bash
cargo bench
```

**12.4 Profile and optimize hot paths** (2 hours)

```bash
# Install flamegraph
cargo install flamegraph

# Profile perft
cargo flamegraph --bench perft_bench -- --bench

# Identify hot spots:
# - Bitboard operations
# - Attack generation
# - Move generation loops
# - Make/unmake move

# Potential optimizations:
# - Inline critical functions
# - Use const lookup tables
# - Optimize bit scanning (use intrinsics)
# - Reduce allocations in MoveList
```

**Time Check:** 4 hours

---

### Day 13: Bug Fixes & Edge Cases

**Goal:** Fix any failing perft tests and edge cases

#### Tasks

**13.1 Debug failing perft tests** (2 hours)

```bash
# Run perft_divide to see where counts differ
cargo run --release --bin perft_divide

# Common bugs:
# - En passant legality (pinned pawns)
# - Castling through check
# - Castling with rook captured
# - Promotion captures
# - Double pawn push conditions
```

**13.2 Add edge case tests** (1.5 hours)

```rust
#[test]
fn en_passant_pin() {
    // Position where en passant would expose king to check
    let fen = "8/8/8/2k5/3Pp3/8/8/4KR2 b - d3 0 1";
    let mut board = Board::from_str(fen).unwrap();

    let mut moves = MoveList::new();
    generate_legal_moves(&mut board, &mut moves);

    // En passant should be illegal (exposes king)
    for &m in moves.iter() {
        assert_ne!(m.flag(), MoveFlag::EnPassant);
    }
}

#[test]
fn castling_rights_after_rook_capture() {
    let fen = "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1";
    let mut board = Board::from_str(fen).unwrap();

    // Capture opponent's rook
    let m = Move::new(Square::A1, Square::A8, MoveFlag::Capture);
    board.make_move(m);

    // Opponent should still have kingside castling
    assert!(board.castling_rights.has(CastlingRights::BLACK_KING));
}

#[test]
fn promotion_on_capture() {
    let fen = "4k3/P7/8/8/8/8/8/4K3 w - - 0 1";
    let mut board = Board::from_str(fen).unwrap();

    board.set_piece(Square::B8, Piece::Rook, Color::Black);

    let mut moves = MoveList::new();
    generate_legal_moves(&mut board, &mut moves);

    // Should have 4 capture-promotions
    let capture_promos = moves.iter()
        .filter(|m| m.to() == Square::B8 && m.is_promotion())
        .count();

    assert_eq!(capture_promos, 4);
}
```

**13.3 Fuzz testing** (30 min)

```rust
#[cfg(test)]
mod fuzz {
    use super::*;

    #[test]
    fn random_fen_no_panic() {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        for _ in 0..1000 {
            let mut fen = String::new();

            // Generate random (possibly invalid) FEN
            for rank in 0..8 {
                let pieces = ['p', 'n', 'b', 'r', 'q', 'k', 'P', 'N', 'B', 'R', 'Q', 'K'];
                for _ in 0..8 {
                    if rng.gen_bool(0.3) {
                        fen.push(pieces[rng.gen_range(0..pieces.len())]);
                    } else {
                        fen.push((rng.gen_range(1..=8) as u8 + b'0') as char);
                    }
                }
                if rank < 7 {
                    fen.push('/');
                }
            }

            fen.push_str(" w KQkq - 0 1");

            // Should not panic, even on invalid FEN
            let _ = Board::from_str(&fen);
        }
    }
}
```

**Time Check:** 4 hours

---

### Day 14: Documentation & Code Quality

**Goal:** Document API and clean up code

#### Tasks

**14.1 Add module documentation** (1.5 hours)

````rust
// crates/engine/src/lib.rs
//! Chess engine core library.
//!
//! This crate provides a complete chess engine implementation including:
//! - Bitboard-based board representation
//! - Complete move generation (all pieces + special moves)
//! - FEN parsing and serialization
//! - Zobrist hashing
//! - Perft testing
//!
//! # Example
//!
//! ```
//! use engine::{Board, movegen::generate_legal_moves, MoveList};
//! use std::str::FromStr;
//!
//! let mut board = Board::startpos();
//! let mut moves = MoveList::new();
//! generate_legal_moves(&mut board, &mut moves);
//!
//! println!("Legal moves from starting position: {}", moves.len());
//! ```

/// Bitboard wrapper providing efficient set operations on 64-bit boards.
pub struct Bitboard(pub u64);

/// Represents a square on the chess board (A1 = 0, H8 = 63).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum Square { /* ... */ }

// Add doc comments to all public items
````

**14.2 Run clippy and fix warnings** (1 hour)

```bash
cargo clippy --all-targets -- -D warnings

# Common issues:
# - Unnecessary borrows
# - Unused variables
# - Complex boolean expressions
# - Missing #[must_use] attributes
```

**14.3 Format code** (15 min)

```bash
cargo fmt
```

**14.4 Update module exports** (30 min)

```rust
// crates/engine/src/lib.rs
pub mod board;
pub mod bitboard;
pub mod square;
pub mod piece;
pub mod color;
pub mod r#move;
pub mod movegen;
pub mod movelist;
pub mod attacks;
pub mod fen;
pub mod zobrist;
pub mod castling;
pub mod perft;

// Re-exports
pub use board::Board;
pub use bitboard::Bitboard;
pub use square::Square;
pub use piece::Piece;
pub use color::Color;
pub use r#move::{Move, MoveFlag};
pub use movelist::MoveList;
pub use castling::CastlingRights;
```

**14.5 Write README for engine crate** (45 min)

```markdown
# Engine Crate

Core chess engine implementation with bitboard-based board representation.

## Features

- Complete move generation (all pieces + special moves)
- FEN parsing and serialization
- Perft validation
- Zobrist hashing
- ≥3M nodes/s performance

## Performance

| Test          | Nodes/s |
| ------------- | ------- |
| Perft depth 5 | 3.2M    |
| Perft depth 6 | 3.1M    |

## Usage

See main documentation.
```

**Time Check:** 4 hours

---

### Day 15: Final Testing & M2 Completion

**Goal:** Final validation and wrap-up

#### Tasks

**15.1 Run full test suite** (30 min)

```bash
cargo test --workspace --all-features
cargo test -- --ignored  # Run slow tests
```

**15.2 Verify all DoD criteria** (1 hour)

```markdown
## M2 Definition of Done Checklist

- [x] Perft d1-d6 match canonical values
- [x] No panics under fuzz testing
- [x] cargo test --workspace passes
- [x] Nodes/s ≥ 3M single-threaded
- [x] Board, movegen, and state modules documented
- [x] Code is lint-clean (clippy + rustfmt)
```

**15.3 Run CI locally** (15 min)

```bash
make ci
```

**15.4 Create benchmark report** (30 min)

```bash
cargo bench > benchmark_results.txt

# Compare with baseline (if exists)
# Document performance metrics
```

**15.5 Update protocol integration** (1 hour)

```rust
// Verify engine types match protocol types
// crates/engine/src/lib.rs

// Ensure types are compatible
use crate::types::{AnalyzeRequest, SearchInfo, BestMove};

impl EngineImpl {
    pub fn analyze(&mut self, req: AnalyzeRequest) -> BestMove {
        // This should already work from M1
        // Just verify it compiles and links correctly
    }
}
```

**15.6 Create PR and documentation** (45 min)

```bash
git checkout -b feature/m2-engine-core
git add .
git commit -m "feat(engine): Complete M2 - Engine Core

- Bitboard-based board representation
- Complete move generation (all pieces + special moves)
- FEN parsing and serialization
- Perft validation (depth 1-6 passing)
- Zobrist hashing
- Performance: 3.2M nodes/s on perft depth 5

Closes #M2"

git push origin feature/m2-engine-core

# Create PR on GitHub
# Add benchmark results to PR description
# Tag reviewers
```

**Time Check:** 4 hours

---

## Summary & Checklists

### Time Breakdown by Component

| Component            | Time    | %        |
| -------------------- | ------- | -------- |
| Board representation | 8h      | 13%      |
| Move generation      | 12h     | 20%      |
| Make/unmake moves    | 4h      | 7%       |
| FEN parser           | 4h      | 7%       |
| Legality checking    | 4h      | 7%       |
| Attack maps          | 4.5h    | 8%       |
| Zobrist hashing      | 3.5h    | 6%       |
| Perft testing        | 8h      | 13%      |
| Performance tuning   | 4h      | 7%       |
| Bug fixes            | 4h      | 7%       |
| Documentation        | 4h      | 7%       |
| Final polish         | 4h      | 7%       |
| **Total**            | **64h** | **100%** |

**Estimated calendar time:** 15 days (with some overlap/efficiency)

---

### Daily Checklist

#### Week 1

- [x] Day 1: Basic types (Square, Piece, Color, Bitboard)
- [x] Day 2: Board struct and accessors
- [x] Day 3: Move struct and castling rights
- [x] Day 4: Attack maps
- [x] Day 5: Move generation part 1

#### Week 2

- [x] Day 6: Move generation part 2 (sliding pieces, castling, en passant)
- [x] Day 7: Make/unmake move
- [x] Day 8: FEN parser and serializer
- [x] Day 9: Legality checking
- [x] Day 10: Zobrist hashing

#### Week 3

- [x] Day 11: Perft implementation
- [x] Day 12: Performance benchmarking
- [x] Day 13: Bug fixes and edge cases
- [x] Day 14: Documentation and code quality
- [x] Day 15: Final testing and M2 completion

---

### Critical Path Items

**Must be done in order:**

1. Basic types → Board → Move generation
2. Make/unmake → Legality → Perft
3. Everything → Performance tuning

**Can be done in parallel:**

- FEN parsing (can develop alongside move generation)
- Zobrist hashing (can add after basic board works)
- Documentation (ongoing throughout)

---

### Common Pitfalls & Solutions

| Pitfall                  | Solution                                      |
| ------------------------ | --------------------------------------------- |
| Perft counts don't match | Use perft_divide to find divergence point     |
| En passant bugs          | Check for pins and legality                   |
| Castling edge cases      | Validate squares not attacked                 |
| Performance issues       | Profile with flamegraph, optimize hot paths   |
| Hash collisions          | Verify Zobrist key generation is truly random |

---

### Resources & References

**Perft Positions:**

- [Chess Programming Wiki - Perft Results](https://www.chessprogramming.org/Perft_Results)

**Bitboard Techniques:**

- [Chess Programming Wiki - Bitboards](https://www.chessprogramming.org/Bitboards)

**Move Generation:**

- [Chess Programming Wiki - Move Generation](https://www.chessprogramming.org/Move_Generation)

**Zobrist Hashing:**

- [Chess Programming Wiki - Zobrist Hashing](https://www.chessprogramming.org/Zobrist_Hashing)

---

## Getting Started Command

```bash
# Start M2 today!
git checkout -b feature/m2-engine-core

# Create all module files
cd crates/engine
mkdir -p tests benches
touch src/{bitboard,square,piece,color,castling,board,move,movelist,movegen,attacks,fen,zobrist,perft}.rs
touch tests/perft.rs
touch benches/perft_bench.rs

# Start coding!
code src/square.rs  # Begin with Day 1, Task 1.2
```

**Good luck! 🚀**
