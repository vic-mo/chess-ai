//! Polyglot opening book format reader
//!
//! Polyglot books are binary files with 16-byte entries:
//! - 8 bytes: position hash (big-endian u64)
//! - 2 bytes: move encoding (big-endian u16)
//! - 2 bytes: weight (big-endian u16)
//! - 4 bytes: learn (big-endian u32)

use crate::board::Board;
use crate::movegen::generate_moves;
use crate::piece::PieceType;
use crate::r#move::Move;
use crate::square::Square;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

/// A Polyglot book entry
#[derive(Debug, Clone)]
struct BookEntry {
    key: u64,
    move_data: u16,
    weight: u16,
    learn: u32,
}

impl BookEntry {
    /// Read an entry from bytes
    fn from_bytes(bytes: &[u8; 16]) -> Self {
        BookEntry {
            key: u64::from_be_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            ]),
            move_data: u16::from_be_bytes([bytes[8], bytes[9]]),
            weight: u16::from_be_bytes([bytes[10], bytes[11]]),
            learn: u32::from_be_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]),
        }
    }

    /// Decode the move from move_data
    fn decode_move(&self) -> (Square, Square, Option<PieceType>) {
        let to_file = (self.move_data & 0x7) as u8;
        let to_rank = ((self.move_data >> 3) & 0x7) as u8;
        let from_file = ((self.move_data >> 6) & 0x7) as u8;
        let from_rank = ((self.move_data >> 9) & 0x7) as u8;
        let promo = (self.move_data >> 12) & 0x7;

        let from = Square::from_coords(from_file, from_rank);
        let to = Square::from_coords(to_file, to_rank);

        let promotion = match promo {
            1 => Some(PieceType::Knight),
            2 => Some(PieceType::Bishop),
            3 => Some(PieceType::Rook),
            4 => Some(PieceType::Queen),
            _ => None,
        };

        (from, to, promotion)
    }
}

/// Polyglot opening book
pub struct PolyglotBook {
    entries: Vec<BookEntry>,
}

impl PolyglotBook {
    /// Load a Polyglot book from file
    pub fn from_file<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut entries = Vec::new();

        loop {
            let mut buf = [0u8; 16];
            match reader.read_exact(&mut buf) {
                Ok(_) => {
                    let entry = BookEntry::from_bytes(&buf);
                    entries.push(entry);
                }
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e),
            }
        }

        Ok(PolyglotBook { entries })
    }

    /// Get the number of entries in the book
    pub fn size(&self) -> usize {
        self.entries.len()
    }

    /// Compute Polyglot hash for a board position
    ///
    /// Polyglot uses a different hash than Zobrist.
    /// For now, we'll use our existing hash and note that we should implement
    /// proper Polyglot hashing if the book doesn't work correctly.
    fn polyglot_hash(board: &Board) -> u64 {
        // TODO: Implement proper Polyglot hash
        // For now, use our standard hash
        board.hash()
    }

    /// Probe the book for moves in a position
    pub fn probe(&self, board: &Board) -> Vec<(Move, u16)> {
        let hash = Self::polyglot_hash(board);
        let legal_moves = generate_moves(board);
        let mut book_moves = Vec::new();

        // Binary search for first entry with this hash
        let idx = match self.entries.binary_search_by_key(&hash, |e| e.key) {
            Ok(i) => i,
            Err(_) => return book_moves, // Not found
        };

        // Collect all entries with this hash
        let mut i = idx;
        while i > 0 && self.entries[i - 1].key == hash {
            i -= 1;
        }

        while i < self.entries.len() && self.entries[i].key == hash {
            let entry = &self.entries[i];
            let (from, to, promotion) = entry.decode_move();

            // Find matching legal move
            for &legal_move in legal_moves.iter() {
                if legal_move.from() == from && legal_move.to() == to {
                    // Check promotion matches if applicable
                    if let Some(promo_type) = promotion {
                        if legal_move.is_promotion() && legal_move.promotion_piece() == Some(promo_type) {
                            book_moves.push((legal_move, entry.weight));
                            break;
                        }
                    } else if !legal_move.is_promotion() {
                        book_moves.push((legal_move, entry.weight));
                        break;
                    }
                }
            }

            i += 1;
        }

        book_moves
    }

    /// Get the best move from the book (weighted random selection)
    pub fn get_move(&self, board: &Board) -> Option<Move> {
        let moves = self.probe(board);
        if moves.is_empty() {
            return None;
        }

        // For now, just pick the highest weighted move
        // TODO: Implement weighted random selection
        let best = moves.iter().max_by_key(|(_, weight)| weight)?;
        Some(best.0)
    }

    /// Check if a position is in the book
    pub fn contains(&self, board: &Board) -> bool {
        let hash = Self::polyglot_hash(board);
        self.entries.binary_search_by_key(&hash, |e| e.key).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entry_decode() {
        // Test move decoding: e2e4 would be encoded as:
        // from: e2 = file 4, rank 1 (0-indexed)
        // to: e4 = file 4, rank 3
        // move_data = (file_to) | (rank_to << 3) | (file_from << 6) | (rank_from << 9) | (promo << 12)
        // = 4 | (3 << 3) | (4 << 6) | (1 << 9) | (0 << 12)
        // = 4 | 24 | 256 | 512 | 0 = 796
        let bytes = [
            0, 0, 0, 0, 0, 0, 0, 0, // key (doesn't matter for this test)
            0x03, 0x1c, // move_data = 796 in big-endian
            0, 100, // weight
            0, 0, 0, 0, // learn
        ];

        let entry = BookEntry::from_bytes(&bytes);
        let (from, to, promo) = entry.decode_move();

        assert_eq!(from.file(), 4); // e-file
        assert_eq!(from.rank(), 1); // 2nd rank (0-indexed)
        assert_eq!(to.file(), 4);   // e-file
        assert_eq!(to.rank(), 3);   // 4th rank (0-indexed)
        assert_eq!(promo, None);
    }
}
