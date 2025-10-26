//! Transposition Table for caching search results.
//!
//! Uses Zobrist hashing to store and retrieve previously searched positions.

use crate::r#move::Move;

/// Transposition table entry size in bytes (16 bytes per entry).
const ENTRY_SIZE: usize = 16;

/// Bound type for transposition table entries.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Bound {
    /// Exact score (PV node)
    Exact,
    /// Lower bound (beta cutoff, score >= beta)
    Lower,
    /// Upper bound (alpha node, score <= alpha)
    Upper,
}

/// A transposition table entry.
#[derive(Debug, Copy, Clone)]
pub struct TTEntry {
    /// Zobrist hash of the position (for verification)
    pub hash: u64,
    /// Best move found in this position
    pub best_move: Move,
    /// Evaluation score
    pub score: i32,
    /// Search depth
    pub depth: u8,
    /// Bound type
    pub bound: Bound,
    /// Age/generation (for replacement scheme)
    pub age: u8,
}

impl TTEntry {
    /// Create a new empty entry.
    pub fn empty() -> Self {
        Self {
            hash: 0,
            best_move: Move::new(
                crate::square::Square::A1,
                crate::square::Square::A1,
                crate::r#move::MoveFlags::QUIET,
            ),
            score: 0,
            depth: 0,
            bound: Bound::Exact,
            age: 0,
        }
    }

    /// Check if this entry is valid for the given hash.
    #[inline]
    pub fn is_valid(&self, hash: u64) -> bool {
        self.hash == hash
    }
}

impl Default for TTEntry {
    fn default() -> Self {
        Self::empty()
    }
}

/// Transposition table using Zobrist hashing.
pub struct TranspositionTable {
    entries: Vec<TTEntry>,
    size: usize,
    generation: u8,
}

impl TranspositionTable {
    /// Create a new transposition table with the given size in MB.
    ///
    /// # Arguments
    /// * `size_mb` - Size in megabytes
    pub fn new(size_mb: usize) -> Self {
        let num_entries = (size_mb * 1024 * 1024) / ENTRY_SIZE;
        // Round to power of 2 for efficient modulo
        let size = num_entries.next_power_of_two();

        Self {
            entries: vec![TTEntry::empty(); size],
            size,
            generation: 0,
        }
    }

    /// Get the index for a given hash.
    #[inline]
    fn index(&self, hash: u64) -> usize {
        // Use modulo with power of 2 (fast bitwise AND)
        (hash as usize) & (self.size - 1)
    }

    /// Probe the transposition table.
    ///
    /// Returns the entry if found and valid, None otherwise.
    pub fn probe(&self, hash: u64) -> Option<&TTEntry> {
        let idx = self.index(hash);
        let entry = &self.entries[idx];

        if entry.is_valid(hash) {
            Some(entry)
        } else {
            None
        }
    }

    /// Store an entry in the transposition table.
    ///
    /// Uses a replacement scheme: replace if deeper search or same generation.
    pub fn store(&mut self, hash: u64, best_move: Move, score: i32, depth: u8, bound: Bound) {
        let idx = self.index(hash);
        let entry = &mut self.entries[idx];

        // Replacement scheme: replace if:
        // 1. Empty slot (hash == 0)
        // 2. Same position (hash match)
        // 3. Deeper search
        // 4. Older generation
        let should_replace = entry.hash == 0
            || entry.hash == hash
            || depth >= entry.depth
            || entry.age != self.generation;

        if should_replace {
            *entry = TTEntry {
                hash,
                best_move,
                score,
                depth,
                bound,
                age: self.generation,
            };
        }
    }

    /// Clear the transposition table.
    pub fn clear(&mut self) {
        for entry in &mut self.entries {
            *entry = TTEntry::empty();
        }
    }

    /// Increment the generation (for aging entries).
    pub fn new_search(&mut self) {
        self.generation = self.generation.wrapping_add(1);
    }

    /// Get the fill percentage (0-1000 permille).
    pub fn hashfull(&self) -> usize {
        // Sample first 1000 entries
        let sample_size = 1000.min(self.size);
        let mut filled = 0;

        for i in 0..sample_size {
            if self.entries[i].hash != 0 {
                filled += 1;
            }
        }

        (filled * 1000) / sample_size
    }

    /// Get the size in entries.
    pub fn size(&self) -> usize {
        self.size
    }
}

impl Default for TranspositionTable {
    fn default() -> Self {
        Self::new(16) // 16 MB default
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::square::Square;

    #[test]
    fn test_tt_create() {
        let tt = TranspositionTable::new(1);
        assert!(tt.size() > 0);
        assert!(tt.size().is_power_of_two());
    }

    #[test]
    fn test_tt_store_probe() {
        let mut tt = TranspositionTable::new(1);
        let hash = 0x1234_5678_9ABC_DEF0;
        let mv = Move::new(Square::E2, Square::E4, crate::r#move::MoveFlags::QUIET);

        tt.store(hash, mv, 100, 5, Bound::Exact);

        let entry = tt.probe(hash);
        assert!(entry.is_some());
        let entry = entry.unwrap();
        assert_eq!(entry.score, 100);
        assert_eq!(entry.depth, 5);
        assert_eq!(entry.bound, Bound::Exact);
    }

    #[test]
    fn test_tt_probe_miss() {
        let tt = TranspositionTable::new(1);
        let hash = 0x1234_5678_9ABC_DEF0;

        let entry = tt.probe(hash);
        assert!(entry.is_none());
    }

    #[test]
    fn test_tt_replacement() {
        let mut tt = TranspositionTable::new(1);
        let hash = 0x1234_5678_9ABC_DEF0;
        let mv = Move::new(Square::E2, Square::E4, crate::r#move::MoveFlags::QUIET);

        // Store shallow search
        tt.store(hash, mv, 50, 3, Bound::Lower);

        // Store deeper search (should replace)
        tt.store(hash, mv, 100, 5, Bound::Exact);

        let entry = tt.probe(hash).unwrap();
        assert_eq!(entry.depth, 5);
        assert_eq!(entry.score, 100);
    }

    #[test]
    fn test_tt_clear() {
        let mut tt = TranspositionTable::new(1);
        let hash = 0x1234_5678_9ABC_DEF0;
        let mv = Move::new(Square::E2, Square::E4, crate::r#move::MoveFlags::QUIET);

        tt.store(hash, mv, 100, 5, Bound::Exact);
        assert!(tt.probe(hash).is_some());

        tt.clear();
        assert!(tt.probe(hash).is_none());
    }

    #[test]
    fn test_tt_generation() {
        let mut tt = TranspositionTable::new(1);
        assert_eq!(tt.generation, 0);

        tt.new_search();
        assert_eq!(tt.generation, 1);

        tt.new_search();
        assert_eq!(tt.generation, 2);
    }

    #[test]
    fn test_tt_hashfull() {
        let mut tt = TranspositionTable::new(1);
        let mv = Move::new(Square::E2, Square::E4, crate::r#move::MoveFlags::QUIET);

        // Initially empty
        assert_eq!(tt.hashfull(), 0);

        // Fill some entries
        for i in 0..100 {
            tt.store(i as u64, mv, 0, 1, Bound::Exact);
        }

        // Should have some fill
        assert!(tt.hashfull() > 0);
    }
}
