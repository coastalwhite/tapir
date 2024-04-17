//! An implementation of an _Initial-Value Tracking Segment Tree_ (IVT Segment Tree).
//!
//! This datastructure stores data segments that are tagged with an address. It tracks both the
//! current value for this data and the value upon first assignment. It is an append-only
//! datastructure.
use std::collections::VecDeque;
use std::ops::Range;

/// A data structure that provides for intervals of addresses
///
/// Based on the idea of the [Interval Tree], we store sorted items that contain data for a
/// specific range of memory. There should never be two items that cover the same memory interval.
///
/// This implementation is currently a bit rough, but it works quite well. It is mostly
/// cache-aware, but there are some leftover optimization opportunities.
///
/// [Interval Tree]: https://en.wikipedia.org/wiki/Interval_tree
#[derive(Debug, Clone)]
pub struct SegmentTree {
    /// Sorted vector that contains the items covering specific memory ranges
    segments: Vec<Segment>,
}

/// A segement containing the data for a specific memory range
#[derive(Debug, Clone)]
pub struct Segment {
    /// Start the range for the segment
    start: u32,

    /// The current content of this interval
    content: VecDeque<u8>,

    /// The initial content of this interval. This is used to give back the needed initial state of
    /// the memory.
    initial: VecDeque<u8>,
}

fn copy_slices_to_slice((lhs, rhs): (&[u8], &[u8]), buffer: &mut [u8]) {
    if buffer.len() <= lhs.len() {
        buffer.copy_from_slice(&lhs[..buffer.len()]);
    } else {
        buffer[..lhs.len()].copy_from_slice(lhs);

        let end = buffer.len() - lhs.len();
        let buffer_end = usize::min(rhs.len(), end);
        buffer[lhs.len()..lhs.len() + buffer_end].copy_from_slice(&rhs[..buffer_end]);
    }
}

fn copy_slices_to_slice_with_offset((lhs, rhs): (&[u8], &[u8]), offset: usize, buffer: &mut [u8]) {
    let lhs_start = usize::min(offset, lhs.len());
    let rhs_start = if offset >= lhs.len() {
        usize::min(offset - lhs.len(), rhs.len())
    } else {
        0
    };
    let (lhs, rhs) = (&lhs[lhs_start..], &rhs[rhs_start..]);

    copy_slices_to_slice((lhs, rhs), buffer);
}

impl Segment {
    #[inline(always)]
    pub fn start(&self) -> u32 {
        self.start
    }

    #[inline(always)]
    pub fn end(&self) -> u32 {
        debug_assert!(u32::try_from(self.content.len()).is_ok());
        self.start + self.content.len() as u32
    }

    #[inline(always)]
    pub fn len(&self) -> u32 {
        self.end() - self.start()
    }

    #[inline(always)]
    pub fn contains(&self, x: u32) -> bool {
        x >= self.start() && x < self.end()
    }

    /// Get the address range stored by the [`Segment`]
    #[inline(always)]
    pub fn range(&self) -> Range<u32> {
        self.start()..self.end()
    }

    /// Generate a new item with `value` at the address `at`
    pub fn new(at: u32, value: u8) -> Self {
        Self {
            start: at,
            content: vec![value].into(),
            initial: vec![value].into(),
        }
    }

    /// Merge `self` and `other`, where `other.start < self.start`
    ///
    /// This assumes that `other` and `self` are positioned immediately after each other i.e.
    /// `self.start == other.end`.
    pub fn merge_front(&mut self, mut other: Self) {
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
    pub fn merge_back(&mut self, other: Self) {
        debug_assert_eq!(self.end(), other.start);

        self.content.extend(other.content);
        self.initial.extend(other.initial);
    }

    /// Push a new element before `self`
    pub fn push_front(&mut self, value: u8) {
        self.content.push_front(value);
        self.initial.push_front(value);
    }

    /// Push a new element after `self`
    pub fn push_back(&mut self, value: u8) {
        self.content.push_back(value);
        self.initial.push_back(value);
    }

    /// Extend several items at the front of the range
    pub fn extend_front(&mut self, buffer: &[u8]) {
        // @Improve: Possible optimization
        for b in buffer.iter().cloned().rev() {
            self.content.push_front(b);
            self.initial.push_front(b);
        }
    }

    /// Extend several items at the extend of the range
    pub fn extend_back(&mut self, buffer: &[u8]) {
        self.content.extend(buffer);
        self.initial.extend(buffer);
    }

    /// Extend several items at the front of the range
    pub fn extend_front_with(&mut self, num: u32, mut f: impl FnMut() -> u8) {
        self.content.reserve(num as usize);
        self.initial.reserve(num as usize);

        for _ in 0..num {
            self.push_front(f());
        }
    }

    /// Extend several items at the back of the range
    pub fn extend_back_with(&mut self, num: u32, mut f: impl FnMut() -> u8) {
        self.content.reserve(num as usize);
        self.initial.reserve(num as usize);

        for _ in 0..num {
            self.push_back(f());
        }
    }

    /// Fill `buffer` with the initial values of `self`.
    ///
    /// Note: This is not taking into account the starting address of `self`. Instead it starts at
    /// the `offset`-th element of _this_ `self`.
    pub fn copy_initial_to_slice(&self, buffer: &mut [u8]) {
        copy_slices_to_slice(self.initial.as_slices(), buffer);
    }

    /// Fill `buffer` with the content of `self`.
    ///
    /// Note: This is not taking into account the starting address of `self`. Instead it starts at
    /// the `offset`-th element of _this_ `self`.
    pub fn copy_content_to_slice(&self, buffer: &mut [u8]) {
        copy_slices_to_slice(self.content.as_slices(), buffer);
    }

    /// Fill `buffer` with the initial values of `self` starting at range `offset`.
    ///
    /// Note: This is not taking into account the starting address of `self`. Instead it starts at
    /// the `offset`-th element of _this_ `self`.
    pub fn copy_initial_to_slice_with_offset(&self, offset: usize, buffer: &mut [u8]) {
        copy_slices_to_slice_with_offset(self.initial.as_slices(), offset, buffer);
    }

    /// Fill `buffer` with the content of `self` starting at range `offset`.
    ///
    /// Note: This is not taking into account the starting address of `self`. Instead it starts at
    /// the `offset`-th element of _this_ `self`.
    pub fn copy_content_to_slice_with_offset(&self, offset: usize, buffer: &mut [u8]) {
        copy_slices_to_slice_with_offset(self.content.as_slices(), offset, buffer);
    }
}

impl SegmentTree {
    /// Create a new [`SegmentTree`]
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    /// Return the index of the [`Segment`] that covers `addr` address.
    ///
    /// If no such segment exists, it returns an `Err` with the index where it an segment contains
    /// `addr` needs to be inserted.
    fn internal_segment_search(&self, addr: u32) -> Result<usize, usize> {
        // @Improve: Possible optimizations
        //
        // 1.
        // Seeing our usecase it might be interesting to investigate a 1-cell cache here. It is
        // very common that we access the same interval twice, almost to a fault. So, it would
        // probably make sense to check the last interval first before doing a binary search.
        //
        // 2.
        // Also, since we will probably not have that many segments, it might make sense to just
        // do a linear search.
        //
        // @Note: Here we search for where the start address would land in the segments. Then, we
        // look at the previous segment to see if it includes the searched for address.
        self.segments.binary_search_by_key(&addr, |segment| segment.start()).or_else(|insertion_idx| {
            if insertion_idx > 0 && self.segments[insertion_idx - 1].end() > addr {
                Ok(insertion_idx - 1)
            } else {
                Err(insertion_idx)
            }
        })
    }

    fn segment_search(&self, addr: u32) -> Result<(&Segment, usize), usize> {
        self.internal_segment_search(addr)
            .map(|segment_idx| {
                let segment = &self.segments[segment_idx];
                debug_assert!(segment.contains(addr));
                (segment, segment_idx)
            })
    }

    fn segment_search_mut(&mut self, addr: u32) -> Result<(&mut Segment, usize), usize> {
        self.internal_segment_search(addr)
            .map(|segment_idx| {
                let segment = &mut self.segments[segment_idx];
                debug_assert!(segment.contains(addr));
                (segment, segment_idx)
            })
    }

    fn insert_at(&mut self, addr: u32, insertion_idx: usize, value: u8) -> (&mut u8, usize) {
        let can_merge_with_prev =
            insertion_idx != 0 && self.segments[insertion_idx - 1].end() == addr;
        let can_merge_with_next = insertion_idx != self.segments.len()
            && self.segments[insertion_idx].start() == addr + 1;

        // @Improve: Redudancy
        // This code can probably be made more practical
        match (can_merge_with_prev, can_merge_with_next) {
            (true, true) => {
                let next_segment = self.segments.remove(insertion_idx);
                let offset = self.segments[insertion_idx - 1].content.len();
                self.segments[insertion_idx - 1].push_back(value);
                self.segments[insertion_idx - 1].merge_back(next_segment);
                (
                    &mut self.segments[insertion_idx - 1].content[offset],
                    insertion_idx - 1,
                )
            }
            (true, false) => {
                let offset = self.segments[insertion_idx - 1].content.len();
                self.segments[insertion_idx - 1].push_back(value);
                (
                    &mut self.segments[insertion_idx - 1].content[offset],
                    insertion_idx - 1,
                )
            }
            (false, true) => {
                self.segments[insertion_idx].push_front(value);
                (&mut self.segments[insertion_idx].content[0], insertion_idx)
            }
            (false, false) => {
                self.segments
                    .insert(insertion_idx, Segment::new(addr, value));
                (&mut self.segments[insertion_idx].content[0], insertion_idx)
            }
        }
    }

    /// Get a reference to a byte within the [`SegmentTree`]
    pub fn get(&self, addr: u32) -> Option<u8> {
        let (interval, _) = self.segment_search(addr).ok()?;
        let offset = (addr - interval.start()) as usize;
        Some(interval.content[offset])
    }

    /// Get a mutable reference to a byte within the [`SegmentTree`]
    pub fn get_mut(&mut self, addr: u32) -> Option<&mut u8> {
        let (interval, _) = self.segment_search_mut(addr).ok()?;
        let offset = (addr - interval.start()) as usize;
        Some(&mut interval.content[offset])
    }

    /// Initializes or sets the memory at `idx` to `value`
    pub fn insert(&mut self, addr: u32, value: u8) -> usize {
        let segment = self.segment_search_mut(addr);

        match segment {
            Ok((segment, segment_idx)) => {
                let offset = (addr - segment.start()) as usize;
                segment.content[offset] = value;
                segment_idx
            }
            Err(insertion_idx) => self.insert_at(addr, insertion_idx, value).1,
        }
    }

    /// Fill `buffer` with memory from the segment tree starting at the `start` address
    ///
    /// This returns an error if it needs to copy memory that is not initialized.
    pub fn get_range(&self, start: u32, buffer: &mut [u8]) -> Result<(), ()> {
        if buffer.is_empty() {
            return Ok(());
        }

        let (segment, _) = self.segment_search(start).map_err(|_| ())?;

        debug_assert!(u32::try_from(buffer.len()).is_ok());
        if segment.end() < start + buffer.len() as u32 {
            return Err(());
        }

        let offset = start - segment.start();
        debug_assert!(usize::try_from(offset).is_ok());
        let offset = offset as usize;

        segment.copy_content_to_slice_with_offset(offset, buffer);

        Ok(())
    }

    /// Returns whether the segment tree contain specified memory for `idx`
    pub fn contains(&self, addr: u32) -> bool {
        self.segment_search(addr).is_ok()
    }

    /// Returns whether the segment tree contain specified memory for all addresses in range
    /// `range`
    pub fn contains_range(&self, range: Range<u32>) -> bool {
        if range.is_empty() {
            return true;
        }

        let Ok((segment, _)) = self.segment_search(range.start) else {
            return false;
        };

        segment.end() >= range.end
    }

    fn initialize(&mut self, idx: u32, value: u8) -> usize {
        let segment = self.segment_search(idx);

        match segment {
            Ok((_, segment_idx)) => segment_idx,
            Err(insertion_idx) => {
                let (_, segment_idx) = self.insert_at(idx, insertion_idx, value);
                segment_idx
            }
        }
    }

    pub fn fill_range(&mut self, start: u32, buffer: &[u8]) {
        if buffer.is_empty() {
            return;
        }

        let start_segment = self.insert(start, buffer[0]);

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
        if self.segments[start_segment].end() >= end {
            let offset = (start - self.segments[start_segment].start()) as usize;
            let content = &mut self.segments[start_segment].content;
            for (i, c) in content
                .range_mut(offset..offset + buffer.len())
                .enumerate()
                .skip(1)
            {
                *c = buffer[i];
            }
            return;
        }

        let end_segment = self.segment_search(end - 1);

        debug_assert!(start_segment <= end_segment.map_or_else(|i| i, |(_, i)| i));

        let end_segment = match end_segment {
            Err(end_segment) if end_segment == start_segment + 1 => {
                let offset = (start - self.segments[start_segment].start()) as usize;

                let range_end_point = self.segments[start_segment].content.len();

                self.segments[start_segment]
                    .content
                    .range_mut(offset..range_end_point)
                    .enumerate()
                    .for_each(|(i, c)| {
                        *c = buffer[i];
                    });

                self.segments[start_segment].extend_back(&buffer[range_end_point..]);

                if end_segment != self.num_segments() && self.segments[end_segment].start() == end {
                    let next_segment = self.segments.remove(end_segment);
                    self.segments[start_segment].merge_back(next_segment);
                }

                return;
            }
            Err(end_segment) => self.insert_at(end - 1, end_segment, 0).1,
            Ok((_, end_segment)) => end_segment,
        };

        debug_assert!(start_segment < end_segment);

        // @Improve: Possible optimization
        // This is extremely hacky and slow, but for now this is a good enough solution.
        for i in 0..end - start {
            self.insert(start + i, buffer[i as usize]);
        }
    }

    /// Ensure that the `range` is initialized, filling all uninitialized memory by calling `f`
    ///
    /// Note: `f` is not necessarily called in the order of the memory addresses.
    pub fn initialize_fill(&mut self, range: Range<u32>, mut f: impl FnMut() -> u8) {
        if range.is_empty() {
            return;
        }

        let start_segment = self.initialize(range.start, f());

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
        if self.segments[start_segment].end() >= range.end {
            return;
        }

        let end_segment = self.segment_search(range.end - 1);

        match end_segment {
            Err(end_segment) if end_segment == start_segment + 1 => {
                let initialize_len = range.end - self.segments[start_segment].end();

                self.segments[start_segment].extend_back_with(initialize_len, f);

                if end_segment != self.num_segments() && self.segments[end_segment].start() == range.end {
                    let next_segment = self.segments.remove(end_segment);
                    self.segments[start_segment].merge_back(next_segment);
                }
            }
            _ => {
                // @Improve: Possible optimization
                // The following is extremely hacky and slow, but for now this is a good enough solution.
                for i in range.start + 1..range.end {
                    self.initialize(i, f());
                }
            }
        }
    }

    pub fn segments_iter(&self) -> std::slice::Iter<Segment> {
        self.segments.iter()
    }

    pub fn num_segments(&self) -> usize {
        self.segments.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let mut tree = SegmentTree::new();

        let mut i = 0;
        tree.initialize_fill(0..16, || {
            let value = 42 + i;
            i += 1;
            value
        });

        // dbg!(&tree);

        assert_eq!(tree.num_segments(), 1);

        for i in 0..16 {
            assert!(tree.get(i).is_some());
        }

        tree.fill_range(4, &[5, 7, 9, 11]);

        assert_eq!(tree.num_segments(), 1);

        for i in 0..4 {
            assert_eq!(tree.get(i + 4), Some(i as u8 * 2 + 5));
        }
    }

    #[test]
    fn sequential_initialize_fills() {
        let mut tree = SegmentTree::new();

        tree.initialize_fill(0..8, || 0);
        tree.initialize_fill(16..24, || 0);
        tree.initialize_fill(8..16, || 0);

        dbg!(&tree);

        assert_eq!(tree.num_segments(), 1);
    }
}
