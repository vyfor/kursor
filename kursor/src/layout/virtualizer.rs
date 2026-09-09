use std::ops::Range;

#[derive(Clone, Debug)]
pub struct Virtualizer {
    extents: Vec<u32>,
    measured_count: usize,
    measured_sum: u64,
    gap: u32,
    seed: u32,
    prefix: Vec<u32>,
    dirty_from: usize,
}

impl Virtualizer {
    pub fn new(seed: u32) -> Self {
        Self {
            extents: Vec::new(),
            measured_count: 0,
            measured_sum: 0,
            gap: 0,
            seed: seed.max(1),
            prefix: vec![0],
            dirty_from: 0,
        }
    }

    pub fn count(&self) -> usize {
        self.extents.len()
    }

    pub fn set_count(&mut self, count: usize) {
        let old = self.extents.len();
        if count == old {
            return;
        }
        if count < old {
            for &extent in &self.extents[count..] {
                if extent != 0 {
                    self.measured_count = self.measured_count.saturating_sub(1);
                    self.measured_sum = self.measured_sum.saturating_sub(u64::from(extent));
                }
            }
            self.extents.truncate(count);
            self.prefix.truncate(count + 1);
        } else {
            self.extents.resize(count, 0);
            self.prefix.resize(count + 1, 0);
        }
        self.dirty_from = self.dirty_from.min(old.min(count));
    }

    pub fn set_gap(&mut self, gap: u32) {
        if self.gap != gap {
            self.gap = gap;
            self.dirty_from = 0;
        }
    }

    pub fn gap(&self) -> u32 {
        self.gap
    }

    pub fn estimate(&self) -> u32 {
        if self.measured_count == 0 {
            self.seed
        } else {
            ((self.measured_sum + self.measured_count as u64 / 2) / self.measured_count as u64)
                .max(1) as u32
        }
    }

    pub fn observe(&mut self, index: usize, extent: u32) {
        if index >= self.extents.len() {
            return;
        }
        let extent = extent.max(1);
        let old = self.extents[index];
        if old == extent {
            return;
        }
        if old != 0 {
            self.measured_count -= 1;
            self.measured_sum -= u64::from(old);
        }
        self.extents[index] = extent;
        self.measured_count += 1;
        self.measured_sum += u64::from(extent);
        self.dirty_from = 0;
    }

    pub fn extent_of(&self, index: usize) -> u32 {
        self.extents
            .get(index)
            .copied()
            .filter(|&e| e != 0)
            .unwrap_or_else(|| self.estimate())
    }

    fn ensure_prefix(&mut self, upto: usize) {
        let upto = upto.min(self.extents.len());
        if self.dirty_from >= upto {
            return;
        }
        let estimate = self.estimate();
        let gap = self.gap;
        let mut acc = self.prefix[self.dirty_from];
        for i in self.dirty_from..upto {
            let extent = match self.extents[i] {
                0 => estimate,
                e => e,
            };
            acc = acc.saturating_add(extent.saturating_add(gap));
            self.prefix[i + 1] = acc;
        }
        self.dirty_from = upto;
    }

    pub fn offset_of(&mut self, index: usize) -> u32 {
        self.ensure_prefix(index);
        self.prefix[index.min(self.extents.len())]
    }

    pub fn total_extent(&mut self) -> u32 {
        let count = self.extents.len();
        if count == 0 {
            return 0;
        }
        self.ensure_prefix(count);
        self.prefix[count].saturating_sub(self.gap)
    }

    pub fn item_at(&mut self, pos: u32) -> Option<usize> {
        let count = self.extents.len();
        if count == 0 {
            return None;
        }
        self.ensure_prefix(count);
        let idx = match self.prefix[1..].binary_search(&pos) {
            Ok(i) => (i + 1).min(count - 1),
            Err(i) => i.min(count - 1),
        };
        let start = self.prefix[idx];
        if pos >= start && pos < start.saturating_add(self.extent_of(idx)) {
            Some(idx)
        } else {
            None
        }
    }

    pub fn item_nearest(&mut self, pos: u32) -> usize {
        let count = self.extents.len();
        if count == 0 {
            return 0;
        }
        self.item_at(pos).unwrap_or_else(|| {
            self.ensure_prefix(count);
            match self.prefix[1..].binary_search(&pos) {
                Ok(i) => (i + 1).min(count - 1),
                Err(i) => i.min(count - 1),
            }
        })
    }

    pub fn range_at(
        &mut self,
        scroll: u32,
        viewport: u32,
        overscan: usize,
    ) -> (Range<usize>, Range<usize>) {
        let count = self.extents.len();
        if count == 0 {
            return (0..0, 0..0);
        }
        let start = self.item_nearest(scroll);
        let mut end = start;
        let limit = scroll.saturating_add(viewport);
        while end < count && self.offset_of(end) < limit {
            end += 1;
        }
        let visible = start..end.max(start).min(count);
        let rendered = visible.start.saturating_sub(overscan)..(visible.end + overscan).min(count);
        (visible, rendered)
    }
}