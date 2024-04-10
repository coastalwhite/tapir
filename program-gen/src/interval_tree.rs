// @Hack
// This is a very bad implementation of a Interval Tree, but it suffices for now. I couldn't
// quickly find a good dynamic Interval Tree crate. So here we go.

use std::cmp::Ordering;
use std::collections::VecDeque;

use risico::memory::BackingStore;
use risico::repr::Addr;

pub struct IntervalTree {
    intervals: Vec<IntervalTreeItem>,
}

pub struct IntervalTreeItem {
    start: u32,
    content: VecDeque<u8>,
}

pub enum IntervalOrdering {
    Lesser,
    Subset,
    Superset,
    Equal,
    AroundStart,
    AroundEnd,
    Greater,
}

pub enum IntervalIdxOrdering {
    Less,
    Contains,
    Greater,
}

impl IntervalTreeItem {
    #[inline(always)]
    fn start(&self) -> u32 {
        self.start
    }

    #[inline(always)]
    fn end(&self) -> u32 {
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

    fn interval_cmp(&self, other: &Self) -> IntervalOrdering {
        use IntervalOrdering as O;

        match () {
            _ if other.start() > self.end() => O::Greater,
            _ if other.end() < self.start() => O::Lesser,
            _ if self.range() == other.range() => O::Equal,
            _ if other.start() <= self.start() && other.end() >= self.end() => O::Superset,
            _ if other.start() <= self.end() => O::AroundEnd,
            _ if other.end() >= self.start() => O::AroundStart,
            _ if other.start() > self.start() || other.end() < self.end() => O::Subset,
            _ => unreachable!(),
        }
    }

    fn interval_idx_cmp(&self, idx: u32) -> IntervalIdxOrdering {
        if idx < self.start() {
            IntervalIdxOrdering::Less
        } else if idx >= self.end() {
            IntervalIdxOrdering::Greater
        } else {
            IntervalIdxOrdering::Contains
        }
    }
}

impl IntervalTree {
    fn internal_binary_search_interval(&self, idx: u32) -> Result<usize, usize> {
        self.intervals.binary_search_by(|i| {
            use IntervalIdxOrdering as IO;
            use Ordering as O;

            match i.interval_idx_cmp(idx) {
                IO::Less => O::Less,
                IO::Contains => O::Equal,
                IO::Greater => O::Greater,
            }
        })
    }

    fn binary_search_interval(&self, idx: u32) -> Result<(&IntervalTreeItem, usize), usize> {
        self.internal_binary_search_interval(idx)
            .map(|interval_idx| {
                let interval = &self.intervals[interval_idx];
                debug_assert!(interval.range().contains(&idx));
                (interval, interval_idx)
            })
    }

    fn binary_search_interval_mut(
        &mut self,
        idx: u32,
    ) -> Result<(&mut IntervalTreeItem, usize), usize> {
        self.internal_binary_search_interval(idx)
            .map(|interval_idx| {
                let interval = &mut self.intervals[interval_idx];
                debug_assert!(interval.range().contains(&idx));
                (interval, interval_idx)
            })
    }

    fn insert_at(&mut self, idx: u32, insertion_idx: usize, value: T) -> (&mut T, usize) {
        let can_merge_with_prev =
            insertion_idx != 0 && self.intervals[insertion_idx - 1].end() == idx;
        let can_merge_with_next =
            insertion_idx != self.intervals.len() && self.intervals[insertion_idx].start() == idx;

        match (can_merge_with_prev, can_merge_with_next) {
            (true, true) => {
                let next_interval = self.intervals.remove(insertion_idx);
                let offset = self.intervals[insertion_idx - 1].content.len();
                self.intervals[insertion_idx - 1].content.push_back(value);
                self.intervals[insertion_idx - 1]
                    .content
                    .extend(next_interval.content.into_iter());
                (
                    &mut self.intervals[insertion_idx - 1].content[offset],
                    insertion_idx - 1,
                )
            }
            (true, false) => {
                let offset = self.intervals[insertion_idx - 1].content.len();
                self.intervals[insertion_idx - 1].content.push_back(value);
                (
                    &mut self.intervals[insertion_idx - 1].content[offset],
                    insertion_idx - 1,
                )
            }
            (false, true) => {
                self.intervals[insertion_idx].content.push_front(value);
                (&mut self.intervals[insertion_idx].content[0], insertion_idx)
            }
            (false, false) => {
                let content = {
                    let mut vs = VecDeque::new();
                    vs.push_back(value);
                    vs
                };
                self.intervals.insert(
                    insertion_idx,
                    IntervalTreeItem {
                        start: idx,
                        content,
                    },
                );
                (&mut self.intervals[insertion_idx].content[0], insertion_idx)
            }
        }
    }

    pub fn get_interval(&self, idx: u32) -> Option<&IntervalTreeItem> {
        self.binary_search_interval(idx).ok().map(|(i, _)| &*i)
    }

    pub fn get_mut_interval(&mut self, idx: u32) -> Option<&mut IntervalTreeItem> {
        self.binary_search_interval_mut(idx).ok().map(|(i, _)| i)
    }

    pub fn get_interval_or_insert_with(
        &self,
        idx: u32,
        f: impl FnOnce() -> u8,
    ) -> &IntervalTreeItem {
        let interval = self.binary_search_interval(idx);

        match interval {
            Ok((interval, _)) => &interval,
            Err(insertion_idx) => {
                let (_, interval_idx) = self.insert_at(idx, insertion_idx, f());
                &self.intervals[interval_idx]
            }
        }
    }

    pub fn get(&self, idx: u32) -> Option<u8> {
        let (interval, _) = self.binary_search_interval(idx).ok()?;
        let offset = (idx - interval.start()) as usize;
        Some(&interval.content[offset])
    }

    pub fn get_mut(&mut self, idx: u32) -> Option<&mut u8> {
        let (interval, _) = self.binary_search_interval(idx).ok()?;
        let offset = (idx - interval.start()) as usize;
        Some(&mut interval.content[offset])
    }

    pub fn get_or_insert_with(&mut self, idx: u32, f: impl FnOnce() -> u8) -> u8 {
        let interval = self.binary_search_interval(idx);

        match interval {
            Ok((interval, _)) => {
                let offset = (idx - interval.start()) as usize;
                &interval.content[offset]
            }
            Err(insertion_idx) => {
                let value = f();
                let (element, _) = self.insert_at(idx, insertion_idx, value);
                element
            }
        }
    }

    pub fn insert(&mut self, idx: u32, value: u8) {
        let interval = self.binary_search_interval(idx);

        match interval {
            Ok((interval, _)) => {
                let offset = (idx - interval.start()) as usize;
                interval.content[offset] = value;
            }
            Err(insertion_idx) => {
                self.insert_at(idx, insertion_idx, value);
            }
        }
    }

    pub fn get_range(&self, start: u32, buffer: &mut [u8]) -> Result<(), ()> {
        if buffer.is_empty() {
            return Ok(());
        }

        let (interval, _) = self.binary_search_interval(start).map_err(|_| ())?;

        if interval.end() < start + buffer.len() as u32 {
            return Err(());
        }

        let offset_start = (start - interval.start()) as usize;
        let offset_end = offset_start + buffer.len();
        let (s1, s2) = interval.content.as_slices();

        let s1_range = offset_start.min(s1.len())..offset_end.min(s1.len());
        let s2_range = (offset_start - s1.len()).min(s2.len())..offset_end.min(s1.len());

        s1[].copy_from_slice(buffer);

        buffer.copy_from_slice(&interval.content[offset..offset + buffer.len()]);
        Ok(())
    }

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

        let interval_start = self.initialize(start, buffer[0]);

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
            for (i, c) in content.range_mut(offset..offset + buffer.len()).enumerate().skip(1) {
                *c = buffer[i];
            };
            return;
        }

        let interval_end = self.initialize(range.end, f());

        if interval_start == interval_end {
            return;
        }

        debug_assert!(interval_start > interval_end);

        let iter = std::iter::from_fn(move || Some(f()));

        for i in 0..interval_end - interval_start {
            let bridge_distance = self.intervals[interval_start + i + 1].start()
                - self.intervals[interval_start + i].end();
            debug_assert!(bridge_distance > 0);

            self.intervals[interval_start]
                .content
                .extend(iter.take(bridge_distance as usize));

            // @Note. This can be way more efficient if we remove everything at once. Might
            // also just not be needed to remove at this point since we are memcpy-ing anyway.
            let interval = self.intervals.remove(interval_start + 1);
            self.intervals[interval_start]
                .content
                .extend(interval.content.into_iter());
            debug_assert_eq!(interval.start, self.intervals[interval_start].end());
        }
    }

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

        let interval_end = self.initialize(range.end, f());

        if interval_start == interval_end {
            return;
        }

        debug_assert!(interval_start > interval_end);

        let iter = std::iter::from_fn(move || Some(f()));

        for i in 0..interval_end - interval_start {
            let bridge_distance = self.intervals[interval_start + i + 1].start()
                - self.intervals[interval_start + i].end();
            debug_assert!(bridge_distance > 0);

            self.intervals[interval_start]
                .content
                .extend(iter.take(bridge_distance as usize));

            // @Note. This can be way more efficient if we remove everything at once. Might
            // also just not be needed to remove at this point since we are memcpy-ing anyway.
            let interval = self.intervals.remove(interval_start + 1);
            self.intervals[interval_start]
                .content
                .extend(interval.content.into_iter());
            debug_assert_eq!(interval.start, self.intervals[interval_start].end());
        }
    }
}

impl IntervalTree {
    /// Gets the element given by `idx` and the element given by `idx + 1`.
    pub fn get_u32(&self, idx: u32) -> Option<(u32, u32)> {
        let (interval, _) = self.binary_search_interval(idx).ok()?;
        let offset = idx - interval.start();

        if offset + 1 >= interval.len() {
            return None;
        }

        Some((
            interval.content[offset as usize],
            interval.content[(offset + 1) as usize],
        ))
    }

    /// Gets the mutable reference to the element given by `idx` and the element given by `idx +
    /// 1`.
    pub fn get_mut_two(&mut self, idx: u32) -> Option<(&mut u32, &mut u32)> {
        let (interval, _) = self.binary_search_interval(idx).ok()?;
        let offset = idx - interval.start();

        if offset + 1 >= interval.len() {
            return None;
        }

        Some((
            &mut interval.content[offset as usize],
            &mut interval.content[(offset + 1) as usize],
        ))
    }

    fn get_le_bytes_opt(&self, at: Addr) -> Option<u32> {
        let word_idx = at.as_u32() >> 2;

        if at.is_word_aligned() {
            self.get(word_idx).map(|i| i.swap_bytes())
        } else {
            let offset = at.word_offset();
            let shift = offset * 8;
            let (lhs, rhs) = self.get_two(word_idx)?;
            let result = (lhs << shift) | (rhs << (32 - shift));
            Some(result.swap_bytes())
        }
    }

    fn set_le_bytes_opt(&mut self, at: Addr, value: u32) -> Option<()> {
        let word_idx = at.as_u32() >> 2;

        if at.is_word_aligned() {
            let word = self.get_mut(word_idx)?;
            *word = value.swap_bytes();
        } else {
            let offset = at.word_offset();
            let shift = offset * 8;
            let (lhs, rhs) = self.get_mut_two(word_idx)?;

            *lhs = (*lhs & (0xFFFF_FFFF << shift)) | (value >> (32 - shift));
            *rhs = (*rhs & (0xFFFF_FFFF >> shift)) | (value << shift);
        }

        Some(())
    }
}

impl BackingStore for IntervalTree {
    fn get_le_bytes(&self, at: Addr) -> u32 {
        let mut bytes = [0u8; 4];
        self.get_range(at.as_u32(), &mut bytes).expect("Unable to load memory");
        u32::from_le_bytes(bytes)
    }

    fn set_le_bytes(&mut self, at: Addr, value: u32) {
        let mut bytes = [0u8; 4];
        self.get_range(at.as_u32(), &mut bytes).expect("Unable to load memory");
        u32::from_be_bytes(bytes)
    }
}
