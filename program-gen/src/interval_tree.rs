// @Hack
// This is a very bad implementation of a Interval Tree, but it suffices for now. I couldn't
// quickly find a good dynamic Interval Tree crate. So here we go.

use std::cmp::Ordering;
use std::collections::VecDeque;

use risico::memory::BackingStore;
use risico::repr::Addr;

/// An [BackingStore] that structures different intervals of addresses.
///
/// Based on the idea of the [Interval Tree], we store sorted items that contain data for a
/// specific range of memory. There should never be two items that cover the same memory interval.
///
/// This implementation is currently a bit rough, but it works quite well. It is mostly
/// cache-aware, but there are some leftover optimization opportunities.
///
/// [Interval Tree]: https://en.wikipedia.org/wiki/Interval_tree
/// [BackingStore]: risico::memory::BackingStore
pub struct IntervalTree {
    /// Sorted vector that contains the items covering specific memory ranges
    intervals: Vec<IntervalTreeItem>,
}

/// A item containing the data for a specific memory range
pub struct IntervalTreeItem {
    /// Start the range for the item
    start: u32,
    
    /// The current content of this interval
    content: VecDeque<u8>,

    /// The initial content of this interval. This is used to give back the needed initial state of
    /// the memory.
    initial: VecDeque<u8>,
}

/// Ordering between an index and a [`IntervalTreeItem`] range
pub enum IntervalIdxOrdering {
    /// Ordering given when `idx < range.start`
    Less,
    /// Ordering given when `idx >= range.end`
    Contains,
    /// Ordering given when `idx >= range.start && idx < range.end`
    Greater,
}

pub struct MemoryRanges {
    starts: Vec<u32>,
    data_starts: Vec<usize>,
    data: Vec<u8>,
}

pub struct MemoryRangesIter<'a> {
    idx: usize,
    inner: &'a MemoryRanges,
}

impl<'a> Iterator for MemoryRangesIter<'a> {
    type Item = (std::ops::Range<u32>, &'a [u8]);

    fn next(&mut self) -> Option<Self::Item> {
        let idx = self.idx;

        if idx >= self.inner.len() {
            return None;
        }

        let start = self.inner.starts[idx];
        let data_start = self.inner.data_starts[idx];
        let data_end = self.inner.data_starts.get(idx + 1).copied().unwrap_or(self.inner.data.len());

        let len = data_end - data_start;
        debug_assert!(u32::try_from(len).is_ok());
        let len = len as u32;

        Some((start..start + len, &self.inner.data[data_start..data_end]))
    }
}

impl MemoryRanges {
    pub fn len(&self) -> usize {
        self.starts.len()
    }
    
    pub fn iter(&self) -> MemoryRangesIter {
        MemoryRangesIter {
            idx: 0,
            inner: &self,
        }
    }
}

impl IntervalTreeItem {
    #[inline(always)]
    fn start(&self) -> u32 {
        self.start
    }

    #[inline(always)]
    fn end(&self) -> u32 {
        debug_assert!(u32::try_from(self.content.len()).is_ok());
        self.start + self.content.len() as u32
    }

    #[inline(always)]
    fn len(&self) -> u32 {
        self.end() - self.start()
    }

    #[inline(always)]
    fn contains(&self, x: u32) -> bool {
        x >= self.start() && x < self.end()
    }

    #[inline(always)]
    fn range(&self) -> std::ops::Range<u32> {
        self.start()..self.end()
    }

    /// Generate a new item with `value` at the address `at`
    fn new(at: u32, value: u8) -> Self {
        Self {
            start: at,
            content: vec![value].into(),
            initial: vec![value].into(),
        }
    }

    /// Compare the `idx` to the item range
    fn idx_cmp(&self, idx: u32) -> IntervalIdxOrdering {
        if idx < self.start() {
            IntervalIdxOrdering::Less
        } else if idx >= self.end() {
            IntervalIdxOrdering::Greater
        } else {
            IntervalIdxOrdering::Contains
        }
    }

    /// Merge `self` and `other`, where `other.start < self.start`
    ///
    /// This assumes that `other` and `self` are positioned immediately after each other i.e.
    /// `self.start == other.end`.
    fn merge_front(&mut self, mut other: Self) {
        debug_assert_eq!(self.start(), other.end());

        other.content.extend(std::mem::take(&mut self.content));
        other.initial.extend(std::mem::take(&mut self.initial));

        self.start = other.start;
        self.content = other.content;
        self.initial = other.initial;
    }

    /// Merge `self` and `other`, where `self.start < other.start`
    ///
    /// This assumes that `self` and `other` are positioned immediately after each other i.e.
    /// `self.end == other.start`.
    fn merge_back(&mut self, other: Self) {
        debug_assert_eq!(self.end(), other.start);

        self.content.extend(other.content);
        self.initial.extend(other.initial);
    }

    /// Push a new element before `self`
    fn push_front(&mut self, value: u8) {
        self.content.push_front(value);
        self.initial.push_front(value);
    }

    /// Push a new element after `self`
    fn push_back(&mut self, value: u8) {
        self.content.push_back(value);
        self.initial.push_back(value);
    }

    /// Extend several items at the front of the range
    fn extend_front(&mut self, buffer: &[u8]) {
        // @Improve: Possible optimization
        for b in buffer.iter().cloned().rev() {
            self.content.push_front(b);
            self.initial.push_front(b);
        }
    }

    /// Extend several items at the extend of the range
    fn extend_back(&mut self, buffer: &[u8]) {
        self.content.extend(buffer);
        self.initial.extend(buffer);
    }

    /// Fill `buffer` with the content of `self` starting at range `offset`.
    ///
    /// Note: This is not taking into account the starting address of `self`. Instead it starts at
    /// the `offset`-th element of _this_ `self`.
    fn get_range(&self, offset: usize, buffer: &mut [u8]) {
        if buffer.is_empty() {
            return;
        }

        assert!(offset < self.content.len());
        assert!(offset + buffer.len() <= self.content.len());

        // @Improve: Possible optimization
        // This can probably be more efficient with VecDeque:as_slices

        for (i, b) in self
            .content
            .range(offset..offset + buffer.len())
            .copied()
            .enumerate()
        {
            buffer[i] = b;
        }
    }
}

impl IntervalTree {
    /// Create a new [`IntervalTree`]
    pub fn new() -> Self {
        Self {
            intervals: Vec::new(),
        }
    }

    /// Return the index of the [`IntervalTreeItem`] that covers `addr` address.
    ///
    /// If no such item exists, it returns an `Err` with the index where it an item covering `addr`
    /// needs to be inserted.
    fn internal_binary_search_interval(&self, addr: u32) -> Result<usize, usize> {
        // @Improve: Possible optimizations
        //
        // 1.
        // Seeing our usecase it might be interesting to investigate a 1-cell cache here. It is
        // very common that we access the same interval twice, almost to a fault. So, it would
        // probably make sense to check the last interval first before doing a binary search.
        //
        // 2.
        // Also, since we will probably not have that many intervals, it might make sense to just
        // do a linear search.
        //
        // 3.
        // There might be a smarter way we can handle each step here. We don't for example that
        //   - Less than start => Less
        //   - Greater than end => Greater
        // These should be the common cases.
        self.intervals.binary_search_by(|item| {
            use IntervalIdxOrdering as IO;
            use Ordering as O;

            match item.idx_cmp(addr) {
                IO::Less => O::Greater,
                IO::Contains => O::Equal,
                IO::Greater => O::Less,
            }
        })
    }

    fn binary_search_interval(&self, addr: u32) -> Result<(&IntervalTreeItem, usize), usize> {
        self.internal_binary_search_interval(addr).map(|interval_idx| {
            let interval = &self.intervals[interval_idx];
            debug_assert!(interval.range().contains(&addr));
            (interval, interval_idx)
        })
    }

    fn binary_search_interval_mut(
        &mut self,
        addr: u32,
    ) -> Result<(&mut IntervalTreeItem, usize), usize> {
        self.internal_binary_search_interval(addr)
            .map(|interval_idx| {
                let interval = &mut self.intervals[interval_idx];
                debug_assert!(interval.range().contains(&addr));
                (interval, interval_idx)
            })
    }

    fn insert_at(&mut self, addr: u32, insertion_idx: usize, value: u8) -> (&mut u8, usize) {
        let can_merge_with_prev =
            insertion_idx != 0 && self.intervals[insertion_idx - 1].end() == addr;
        let can_merge_with_next = insertion_idx != self.intervals.len()
            && self.intervals[insertion_idx].start() == addr + 1;

        // @Improve: Redudancy
        // This code can probably be made more practical
        match (can_merge_with_prev, can_merge_with_next) {
            (true, true) => {
                let next_interval = self.intervals.remove(insertion_idx);
                let offset = self.intervals[insertion_idx - 1].content.len();
                self.intervals[insertion_idx - 1].push_back(value);
                self.intervals[insertion_idx - 1].merge_back(next_interval);
                (
                    &mut self.intervals[insertion_idx - 1].content[offset],
                    insertion_idx - 1,
                )
            }
            (true, false) => {
                let offset = self.intervals[insertion_idx - 1].content.len();
                self.intervals[insertion_idx - 1].push_back(value);
                (
                    &mut self.intervals[insertion_idx - 1].content[offset],
                    insertion_idx - 1,
                )
            }
            (false, true) => {
                self.intervals[insertion_idx].push_front(value);
                (&mut self.intervals[insertion_idx].content[0], insertion_idx)
            }
            (false, false) => {
                self.intervals
                    .insert(insertion_idx, IntervalTreeItem::new(addr, value));
                (&mut self.intervals[insertion_idx].content[0], insertion_idx)
            }
        }
    }

    pub fn get(&self, idx: u32) -> Option<u8> {
        let (interval, _) = self.binary_search_interval(idx).ok()?;
        let offset = (idx - interval.start()) as usize;
        Some(interval.content[offset])
    }

    pub fn get_mut(&mut self, idx: u32) -> Option<&mut u8> {
        let (interval, _) = self.binary_search_interval_mut(idx).ok()?;
        let offset = (idx - interval.start()) as usize;
        Some(&mut interval.content[offset])
    }

    /// Initializes or sets the memory at `idx` to `value`
    pub fn insert(&mut self, idx: u32, value: u8) -> usize {
        let interval = self.binary_search_interval_mut(idx);

        match interval {
            Ok((interval, interval_idx)) => {
                let offset = (idx - interval.start()) as usize;
                interval.content[offset] = value;
                interval_idx
            }
            Err(insertion_idx) => self.insert_at(idx, insertion_idx, value).1,
        }
    }

    /// Fill `buffer` with memory from the interval tree starting at the `start` address
    ///
    /// This returns an error if it needs to copy memory that is not initialized.
    pub fn get_range(&self, start: u32, buffer: &mut [u8]) -> Result<(), ()> {
        if buffer.is_empty() {
            return Ok(());
        }

        let (interval, _) = self.binary_search_interval(start).map_err(|_| ())?;

        debug_assert!(u32::try_from(buffer.len()).is_ok());
        if interval.end() < start + buffer.len() as u32 {
            return Err(());
        }

        let offset = start - interval.start();
        debug_assert!(usize::try_from(offset).is_ok());
        let offset = offset as usize;

        interval.get_range(offset, buffer);

        Ok(())
    }

    /// Returns whether the interval tree contain specified memory for `idx`
    pub fn contains(&self, idx: u32) -> bool {
        self.binary_search_interval(idx).is_ok()
    }

    /// Returns whether the interval tree contain specified memory for all addresses in range
    /// `range`
    pub fn contains_range(&self, range: std::ops::Range<u32>) -> bool {
        if range.is_empty() {
            return true;
        }

        let Ok((interval, _)) = self.binary_search_interval(range.start) else {
            return false;
        };

        interval.end() >= range.end
    }

    fn initialize(&mut self, idx: u32, value: u8) -> usize {
        let interval = self.binary_search_interval(idx);

        match interval {
            Ok((_, interval_idx)) => interval_idx,
            Err(insertion_idx) => {
                let (_, interval_idx) = self.insert_at(idx, insertion_idx, value);
                interval_idx
            }
        }
    }

    pub fn fill_range(&mut self, start: u32, buffer: &[u8]) {
        if buffer.is_empty() {
            return;
        }

        let interval_start = self.insert(start, buffer[0]);

        if buffer.len() == 1 {
            return;
        }

        let end = start + buffer.len() as u32;

        // @Note
        // This is allowed since we know that that
        //   * range.start < range.end
        //   * intervals[interval_start].start() <= range.start
        //   * intervals[interval_start].end()   >  range.start
        //
        // Therefore, we know that if the interval end is further than range.end, we don't have to
        // initialize anything anymore.
        if self.intervals[interval_start].end() >= end {
            let offset = (start - self.intervals[interval_start].start()) as usize;
            let content = &mut self.intervals[interval_start].content;
            for (i, c) in content
                .range_mut(offset..offset + buffer.len())
                .enumerate()
                .skip(1)
            {
                *c = buffer[i];
            }
            return;
        }

        let interval_end = self.binary_search_interval(end - 1);

        debug_assert!(interval_start <= interval_end.map_or_else(|i| i, |(_, i)| i));

        let interval_end = match interval_end {
            Err(interval_end) if interval_end == interval_start + 1 => {
                let offset = (start - self.intervals[interval_start].start()) as usize;

                let range_end_point = self.intervals[interval_start].content.len();

                self.intervals[interval_start]
                    .content
                    .range_mut(offset..range_end_point)
                    .enumerate()
                    .for_each(|(i, c)| {
                        *c = buffer[i];
                    });

                self.intervals[interval_start].extend_back(&buffer[range_end_point..]);
                return;
            }
            Err(interval_end) => self.insert_at(end - 1, interval_end, 0).1,
            Ok((_, interval_end)) => interval_end,
        };

        debug_assert!(interval_start > interval_end);

        // @Improve: Possible optimization
        // This is extremely hacky and slow, but for now this is a good enough solution.
        for i in start..end {
            self.insert(i, buffer[i as usize]);
        }
    }

    /// Ensure that the `range` is initialized, filling all uninitialized memory by calling `f`
    /// 
    /// Note: `f` is not necessarily called in the order of the memory addresses.
    pub fn initialize_fill(&mut self, range: std::ops::Range<u32>, mut f: impl FnMut() -> u8) {
        if range.is_empty() {
            return;
        }

        let interval_start = self.initialize(range.start, f());

        if range.len() == 1 {
            return;
        }

        // @Note
        // This is allowed since we know that that
        //   * range.start < range.end
        //   * intervals[interval_start].start() <= range.start
        //   * intervals[interval_start].end()   >  range.start
        //
        // Therefore, we know that if the interval end is further than range.end, we don't have to
        // initialize anything anymore.
        if self.intervals[interval_start].end() >= range.end {
            return;
        }

        // @Improve: Possible optimization
        // It would probably be a good idea to add a special case here for when `interval_start ==
        // Err(interval_end - 1)`. In that case, we don't have to initialize the end and can just
        // `extend_back` the interval used for the start. This would require quite a lot of extra
        // code though.
        let interval_end = self.initialize(range.end - 1, f());

        debug_assert!(interval_start <= interval_end);

        if interval_start == interval_end {
            return;
        }

        // @Improve: Possible optimization
        // The following is extremely hacky and slow, but for now this is a good enough solution.
        for i in range.start + 1..range.end - 1 {
            self.initialize(i, f());
        }
    }

    pub fn initial(&self) -> MemoryRanges {
        let mut starts = Vec::with_capacity(self.intervals.len());
        let mut data_starts = Vec::with_capacity(self.intervals.len());
        let mut data = Vec::with_capacity(self.intervals.len());

        for item in &self.intervals {
            starts.push(item.start());
            data_starts.push(data.len());

            let (lhs, rhs) = item.initial.as_slices();
            data.extend_from_slice(lhs);
            data.extend_from_slice(rhs);
        }

        MemoryRanges {
            starts,
            data_starts,
            data,
        }
    }

    pub fn content(&self) -> MemoryRanges {
        let mut starts = Vec::with_capacity(self.intervals.len());
        let mut data_starts = Vec::with_capacity(self.intervals.len());
        let mut data = Vec::with_capacity(self.intervals.len());

        for item in &self.intervals {
            starts.push(item.start());
            data_starts.push(data.len());

            let (lhs, rhs) = item.content.as_slices();
            data.extend_from_slice(lhs);
            data.extend_from_slice(rhs);
        }

        MemoryRanges {
            starts,
            data_starts,
            data,
        }
    }
}

impl BackingStore for IntervalTree {
    fn get_le_bytes(&self, at: Addr) -> u32 {
        let mut bytes = [0u8; 4];
        self.get_range(at.as_u32(), &mut bytes)
            .expect("Unable to load memory");
        u32::from_le_bytes(bytes)
    }

    fn set_le_bytes(&mut self, at: Addr, value: u32) {
        let value = value.to_le_bytes();
        self.fill_range(at.as_u32(), &value);
    }

    fn write_to(&mut self, at: Addr, src: &[u8]) {
        self.fill_range(at.as_u32(), src)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let mut tree = IntervalTree::new();

        let mut i = 0;
        tree.initialize_fill(0..16, || {
            let value = 42 + i;
            i += 1;
            value
        });

        dbg!(&tree);

        assert_eq!(tree.intervals.len(), 1);

        for i in 0..16 {
            assert!(tree.get(i).is_some());
        }

        tree.fill_range(4, &[5, 7, 9, 11]);

        assert_eq!(tree.intervals.len(), 1);

        for i in 0..4 {
            assert_eq!(tree.get(i + 4), Some(i as u8 * 2 + 5));
        }
    }
}
