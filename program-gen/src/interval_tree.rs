// @Hack
// This is a very bad implementation of a Interval Tree, but it suffices for now. I couldn't
// quickly find a good dynamic Interval Tree crate. So here we go.

use std::cmp::Ordering;
use std::collections::VecDeque;

use risico::memory::BackingStore;
use risico::repr::Addr;

#[derive(Debug)]
pub struct IntervalTree {
    intervals: Vec<IntervalTreeItem>,
}

#[derive(Debug)]
pub struct IntervalTreeItem {
    start: u32,
    content: VecDeque<u8>,
    initial: VecDeque<u8>,
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

    fn new(at: u32, value: u8) -> Self {
        Self {
            start: at,
            content: vec![value].into(),
            initial: vec![value].into(),
        }
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

    fn merge_front(&mut self, mut other: Self) {
        debug_assert_eq!(self.start(), other.end());

        other.content.extend(std::mem::take(&mut self.content));
        other.initial.extend(std::mem::take(&mut self.initial));

        self.start = other.start;
        self.content = other.content;
        self.initial = other.initial;
    }

    fn merge_back(&mut self, other: Self) {
        debug_assert_eq!(self.end(), other.start);

        self.content.extend(other.content);
        self.initial.extend(other.initial);
    }

    fn push_front(&mut self, value: u8) {
        self.content.push_front(value);
        self.initial.push_front(value);
    }

    fn push_back(&mut self, value: u8) {
        self.content.push_back(value);
        self.initial.push_back(value);
    }

    fn extend_front(&mut self, buffer: &[u8]) {
        for b in buffer.iter().cloned().rev() {
            self.content.push_front(b);
            self.initial.push_front(b);
        }
    }

    fn extend_back(&mut self, buffer: &[u8]) {
        self.content.extend(buffer);
        self.initial.extend(buffer);
    }

    fn get_range(&self, at: usize, buffer: &mut [u8]) {
        if buffer.is_empty() {
            return;
        }

        assert!(at < self.content.len());
        assert!(at + buffer.len() <= self.content.len());

        // @TODO
        // This can probably be more efficient with VecDeque:as_slices

        for (i, b) in self
            .content
            .range(at..at + buffer.len())
            .copied()
            .enumerate()
        {
            buffer[i] = b;
        }
    }
}

impl IntervalTree {
    fn internal_binary_search_interval(&self, idx: u32) -> Result<usize, usize> {
        // @Improve
        // Seeing our usecase it might be interesting to investigate a 1-cell cache here. It is
        // very common that we access the same interval twice, almost to a fault. So, it would
        // probably make sense to check the last interval first before doing a binary search.
        //
        // Also, since we will probably not have that many intervals, it might make sense to just
        // do a linear search.
        self.intervals.binary_search_by(|i| {
            use IntervalIdxOrdering as IO;
            use Ordering as O;

            match i.interval_idx_cmp(idx) {
                IO::Less => O::Greater,
                IO::Contains => O::Equal,
                IO::Greater => O::Less,
            }
        })
    }

    fn binary_search_interval(&self, idx: u32) -> Result<(&IntervalTreeItem, usize), usize> {
        self.internal_binary_search_interval(idx).map(|interval_idx| {
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

    fn insert_at(&mut self, idx: u32, insertion_idx: usize, value: u8) -> (&mut u8, usize) {
        let can_merge_with_prev =
            insertion_idx != 0 && self.intervals[insertion_idx - 1].end() == idx;
        let can_merge_with_next = insertion_idx != self.intervals.len()
            && self.intervals[insertion_idx].start() == idx + 1;

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
                    .insert(insertion_idx, IntervalTreeItem::new(idx, value));
                (&mut self.intervals[insertion_idx].content[0], insertion_idx)
            }
        }
    }

    pub fn new() -> Self {
        Self {
            intervals: Vec::new(),
        }
    }

    pub fn get_interval(&self, idx: u32) -> Option<&IntervalTreeItem> {
        self.binary_search_interval(idx).ok().map(|(i, _)| &*i)
    }

    pub fn get_mut_interval(&mut self, idx: u32) -> Option<&mut IntervalTreeItem> {
        self.binary_search_interval_mut(idx).ok().map(|(i, _)| i)
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

    pub fn get_or_insert_with(&mut self, idx: u32, f: impl FnOnce() -> u8) -> u8 {
        let interval = self.binary_search_interval(idx);

        match interval {
            Ok((interval, _)) => {
                let offset = (idx - interval.start()) as usize;
                interval.content[offset]
            }
            Err(insertion_idx) => {
                let value = f();
                let (element, _) = self.insert_at(idx, insertion_idx, value);
                *element
            }
        }
    }

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

    pub fn get_range(&self, start: u32, buffer: &mut [u8]) -> Result<(), ()> {
        if buffer.is_empty() {
            return Ok(());
        }

        let (interval, _) = self.binary_search_interval(start).map_err(|_| ())?;

        if interval.end() < start + buffer.len() as u32 {
            return Err(());
        }

        let offset = (start - interval.start()) as usize;

        interval.get_range(offset, buffer);

        Ok(())
    }

    pub fn contains(&self, idx: u32) -> bool {
        self.binary_search_interval(idx).is_ok()
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

        // @Hack. This is extremely hacky and slow, but for now this is a good enough solution.
        for i in start..end {
            self.insert(i, buffer[i as usize]);
        }

        // for i in 0..interval_end - interval_start {
        //     let bridge_distance = self.intervals[interval_start + i + 1].start()
        //         - self.intervals[interval_start + i].end();
        //     debug_assert!(bridge_distance > 0);
        //
        //     let bridge_start = self.intervals[interval_start + i].end() - start;
        //     self.intervals[interval_start]
        //         .content
        //         .extend(&buffer[bridge_start as usize..(bridge_start + bridge_distance) as usize]);
        //
        //     // @Note. This can be way more efficient if we remove everything at once. Might
        //     // also just not be needed to remove at this point since we are memcpy-ing anyway.
        //     let interval = self.intervals.remove(interval_start + 1);
        //     self.intervals[interval_start]
        //         .content
        //         .extend(interval.content.into_iter());
        //     debug_assert_eq!(interval.start, self.intervals[interval_start].end());
        // }
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

        let interval_end = self.initialize(range.end - 1, f());

        debug_assert!(interval_start <= interval_end);

        if interval_start == interval_end {
            return;
        }

        for i in range.start + 1..range.end - 1 {
            self.initialize(i, f());
        }

        // for i in 0..interval_end - interval_start {
        //     let bridge_distance = self.intervals[interval_start + i + 1].start()
        //         - self.intervals[interval_start + i].end();
        //     debug_assert!(bridge_distance > 0);
        //
        //     // @Hack. This is really inefficient.
        //     let mut vs = Vec::with_capacity(bridge_distance as usize);
        //     for i in 0..bridge_distance as usize {
        //         vs[i] = f();
        //     }
        //
        //     self.intervals[interval_start].extend_back(&vs);
        //
        //     // @Note. This can be way more efficient if we remove everything at once. Might
        //     // also just not be needed to remove at this point since we are memcpy-ing anyway.
        //     let interval = self.intervals.remove(interval_start + 1);
        //     debug_assert_eq!(interval.start, self.intervals[interval_start].end());
        //     self.intervals[interval_start].merge_back(interval);
        // }
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
