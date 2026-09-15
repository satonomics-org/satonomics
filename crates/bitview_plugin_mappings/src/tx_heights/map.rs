use std::ops::Deref;

use brk_types::{Height, TxIndex};
use rangeindex::RangeMap;

const BUCKET_SHIFT: usize = 16;

/// Transaction-height boundaries with a small directory narrowing floor lookups.
/// The directory indexes the same boundary array; it stores no separate mapping.
pub struct TxHeightMap {
    ranges: RangeMap<TxIndex, Height>,
    // First boundary at or above each bucket; the final entry is ranges.len().
    bounds: Vec<u32>,
}

impl From<Vec<TxIndex>> for TxHeightMap {
    fn from(starts: Vec<TxIndex>) -> Self {
        let capacity = starts
            .last()
            .map_or(0, |&index| (usize::from(index) >> BUCKET_SHIFT) + 2);
        let mut bounds = Vec::with_capacity(capacity);
        for (position, &index) in starts.iter().enumerate() {
            extend_bounds(&mut bounds, position, index);
        }
        Self {
            ranges: RangeMap::from(starts),
            bounds,
        }
    }
}

impl TxHeightMap {
    #[inline]
    pub fn get_shared(&self, index: TxIndex) -> Option<Height> {
        let starts = self.ranges.as_slice();
        if index < *starts.first()? {
            return None;
        }
        if index >= *starts.last().unwrap() {
            return Some(Height::from(starts.len() - 1));
        }
        let bucket = usize::from(index) >> BUCKET_SHIFT;
        let from = (self.bounds[bucket] as usize).saturating_sub(1);
        let to = self.bounds[bucket + 1] as usize;
        let position = starts[from..to].partition_point(|&start| start <= index);
        Some(Height::from(from + position - 1))
    }

    pub(super) fn update_at(&mut self, from: usize, starts: impl IntoIterator<Item = TxIndex>) {
        assert!(from <= self.ranges.len());
        self.ranges.truncate(from);
        if let Some(&last) = self.ranges.as_slice().last() {
            self.bounds
                .truncate((usize::from(last) >> BUCKET_SHIFT) + 2);
            *self.bounds.last_mut().unwrap() = from as u32;
        } else {
            self.bounds.clear();
        }
        self.ranges.extend(starts);
        for (offset, &index) in self.ranges.as_slice()[from..].iter().enumerate() {
            extend_bounds(&mut self.bounds, from + offset, index);
        }
    }
}

impl Deref for TxHeightMap {
    type Target = RangeMap<TxIndex, Height>;

    fn deref(&self) -> &Self::Target {
        &self.ranges
    }
}

fn extend_bounds(bounds: &mut Vec<u32>, position: usize, index: TxIndex) {
    let bucket = usize::from(index) >> BUCKET_SHIFT;
    let position = u32::try_from(position).expect("height fits u32");
    let end = position.checked_add(1).expect("boundary count fits u32");
    while bounds.len() <= bucket {
        bounds.push(position);
    }
    if bounds.len() == bucket + 1 {
        bounds.push(end);
    } else {
        *bounds.last_mut().unwrap() = end;
    }
}

#[cfg(test)]
mod tests;
