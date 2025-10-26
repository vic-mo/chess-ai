use crate::r#move::Move;

/// Maximum number of moves in any chess position.
/// The theoretical maximum is 218, but we use 256 for safety and alignment.
const MAX_MOVES: usize = 256;

/// A stack-allocated list of moves.
///
/// This avoids heap allocation during move generation, which is critical
/// for performance during search. The list has a fixed capacity of 256 moves.
///
/// # Example
/// ```
/// use engine::movelist::MoveList;
/// use engine::r#move::{Move, MoveFlags};
/// use engine::square::Square;
///
/// let mut list = MoveList::new();
/// list.push(Move::new(Square::E2, Square::E4, MoveFlags::QUIET));
/// assert_eq!(list.len(), 1);
/// ```
#[derive(Clone)]
pub struct MoveList {
    moves: [Move; MAX_MOVES],
    len: usize,
}

impl MoveList {
    /// Creates a new empty move list.
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            moves: [Move::null(); MAX_MOVES],
            len: 0,
        }
    }

    /// Adds a move to the list.
    ///
    /// # Panics
    /// Panics if the list is already full (256 moves).
    #[inline(always)]
    pub fn push(&mut self, m: Move) {
        debug_assert!(self.len < MAX_MOVES, "MoveList overflow");
        self.moves[self.len] = m;
        self.len += 1;
    }

    /// Returns the number of moves in the list.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns true if the list is empty.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Clears all moves from the list.
    #[inline(always)]
    pub fn clear(&mut self) {
        self.len = 0;
    }

    /// Returns an iterator over the moves.
    #[inline(always)]
    pub fn iter(&self) -> impl Iterator<Item = &Move> {
        self.moves[..self.len].iter()
    }

    /// Returns a mutable iterator over the moves.
    #[inline(always)]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Move> {
        self.moves[..self.len].iter_mut()
    }

    /// Returns a reference to the move at the given index.
    ///
    /// # Panics
    /// Panics if index >= len.
    #[inline(always)]
    pub fn get(&self, index: usize) -> &Move {
        debug_assert!(index < self.len, "MoveList index out of bounds");
        &self.moves[index]
    }

    /// Returns a mutable reference to the move at the given index.
    ///
    /// # Panics
    /// Panics if index >= len.
    #[inline(always)]
    pub fn get_mut(&mut self, index: usize) -> &mut Move {
        debug_assert!(index < self.len, "MoveList index out of bounds");
        &mut self.moves[index]
    }

    /// Swaps two moves in the list.
    #[inline(always)]
    pub fn swap(&mut self, a: usize, b: usize) {
        debug_assert!(a < self.len && b < self.len, "MoveList swap out of bounds");
        self.moves.swap(a, b);
    }

    /// Returns a slice of all moves.
    #[inline(always)]
    pub fn as_slice(&self) -> &[Move] {
        &self.moves[..self.len]
    }

    /// Returns a mutable slice of all moves.
    #[inline(always)]
    pub fn as_mut_slice(&mut self) -> &mut [Move] {
        &mut self.moves[..self.len]
    }

    /// Sorts the moves in the list using the given comparison function.
    ///
    /// This is useful for move ordering in search.
    pub fn sort_by<F>(&mut self, compare: F)
    where
        F: FnMut(&Move, &Move) -> std::cmp::Ordering,
    {
        self.as_mut_slice().sort_by(compare);
    }

    /// Sorts the moves in the list using the given key extraction function.
    pub fn sort_by_key<K, F>(&mut self, key: F)
    where
        F: FnMut(&Move) -> K,
        K: Ord,
    {
        self.as_mut_slice().sort_by_key(key);
    }
}

impl Default for MoveList {
    fn default() -> Self {
        Self::new()
    }
}

impl std::ops::Index<usize> for MoveList {
    type Output = Move;

    #[inline(always)]
    fn index(&self, index: usize) -> &Self::Output {
        self.get(index)
    }
}

impl std::ops::IndexMut<usize> for MoveList {
    #[inline(always)]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index)
    }
}

impl IntoIterator for MoveList {
    type Item = Move;
    type IntoIter = MoveListIntoIter;

    fn into_iter(self) -> Self::IntoIter {
        MoveListIntoIter {
            list: self,
            index: 0,
        }
    }
}

/// Iterator that consumes a MoveList.
pub struct MoveListIntoIter {
    list: MoveList,
    index: usize,
}

impl Iterator for MoveListIntoIter {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.list.len {
            let m = self.list.moves[self.index];
            self.index += 1;
            Some(m)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.list.len - self.index;
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for MoveListIntoIter {
    fn len(&self) -> usize {
        self.list.len - self.index
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::r#move::MoveFlags;
    use crate::square::Square;

    #[test]
    fn movelist_new() {
        let list = MoveList::new();
        assert_eq!(list.len(), 0);
        assert!(list.is_empty());
    }

    #[test]
    fn movelist_push() {
        let mut list = MoveList::new();
        let m1 = Move::new(Square::E2, Square::E4, MoveFlags::QUIET);
        let m2 = Move::new(Square::D2, Square::D4, MoveFlags::DOUBLE_PAWN_PUSH);

        list.push(m1);
        assert_eq!(list.len(), 1);
        assert_eq!(list[0], m1);

        list.push(m2);
        assert_eq!(list.len(), 2);
        assert_eq!(list[1], m2);
    }

    #[test]
    fn movelist_clear() {
        let mut list = MoveList::new();
        list.push(Move::new(Square::E2, Square::E4, MoveFlags::QUIET));
        list.push(Move::new(Square::D2, Square::D4, MoveFlags::QUIET));
        assert_eq!(list.len(), 2);

        list.clear();
        assert_eq!(list.len(), 0);
        assert!(list.is_empty());
    }

    #[test]
    fn movelist_iter() {
        let mut list = MoveList::new();
        let moves = [
            Move::new(Square::E2, Square::E4, MoveFlags::QUIET),
            Move::new(Square::D2, Square::D4, MoveFlags::QUIET),
            Move::new(Square::E7, Square::E5, MoveFlags::QUIET),
        ];

        for &m in &moves {
            list.push(m);
        }

        let collected: Vec<_> = list.iter().copied().collect();
        assert_eq!(collected, moves);
    }

    #[test]
    fn movelist_into_iter() {
        let mut list = MoveList::new();
        let moves = [
            Move::new(Square::E2, Square::E4, MoveFlags::QUIET),
            Move::new(Square::D2, Square::D4, MoveFlags::QUIET),
        ];

        for &m in &moves {
            list.push(m);
        }

        let collected: Vec<_> = list.into_iter().collect();
        assert_eq!(collected, moves);
    }

    #[test]
    fn movelist_swap() {
        let mut list = MoveList::new();
        let m1 = Move::new(Square::E2, Square::E4, MoveFlags::QUIET);
        let m2 = Move::new(Square::D2, Square::D4, MoveFlags::QUIET);

        list.push(m1);
        list.push(m2);

        list.swap(0, 1);
        assert_eq!(list[0], m2);
        assert_eq!(list[1], m1);
    }

    #[test]
    fn movelist_sort() {
        let mut list = MoveList::new();
        list.push(Move::new(Square::H8, Square::H7, MoveFlags::QUIET));
        list.push(Move::new(Square::A1, Square::A2, MoveFlags::QUIET));
        list.push(Move::new(Square::E4, Square::E5, MoveFlags::QUIET));

        list.sort_by_key(|m| m.from().index());

        assert_eq!(list[0].from(), Square::A1);
        assert_eq!(list[1].from(), Square::E4);
        assert_eq!(list[2].from(), Square::H8);
    }

    #[test]
    fn movelist_size() {
        // MoveList should be reasonably sized
        let size = std::mem::size_of::<MoveList>();
        // MAX_MOVES * 2 bytes (Move) + 8 bytes (len) = 520 bytes
        assert!(size <= 1024, "MoveList is too large: {} bytes", size);
    }
}
