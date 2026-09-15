use std::sync::Arc;

use bitview_plugin_indexer::Indexer;
use brk_types::{Height, TxIndex};
use parking_lot::{RwLock, RwLockReadGuard};
use vecdb::{AnyVec, ReadableVec, VecIndex};

mod map;

pub use map::TxHeightMap;

#[cfg(test)]
mod benchmark;

/// Reverse mapping from `TxIndex` → `Height` via indexed block boundaries.
///
/// Built from `first_tx_index` (the first TxIndex in each block). A floor lookup
/// on any TxIndex gives the block height that contains it.
///
/// The compute thread maintains one resident range map shared with query readers.
#[derive(Clone)]
pub struct TxHeights(Arc<RwLock<TxHeightMap>>);

impl TxHeights {
    /// Hold a stable mapping for a batch of lookups, with one read lock.
    pub fn read(&self) -> RwLockReadGuard<'_, TxHeightMap> {
        self.0.read_recursive()
    }

    /// Build from the full `first_tx_index` vec at startup.
    pub fn init(indexer: &Indexer) -> Self {
        let source = &indexer.vecs().transactions.first_tx_index;
        Self(Arc::new(RwLock::new(TxHeightMap::from(source.collect()))))
    }

    /// Extend with new blocks since last call. Truncates on reorg.
    pub fn update(&self, indexer: &Indexer, reorg_height: Height) {
        let source = &indexer.vecs().transactions.first_tx_index;
        let from = self
            .read()
            .len()
            .min(reorg_height.to_usize())
            .min(source.len());
        let starts = source.collect_range_at(from, source.len());
        self.0.write().update_at(from, starts);
    }

    /// Look up the block height for a given tx_index.
    #[inline]
    pub fn get_shared(&self, tx_index: TxIndex) -> Option<Height> {
        self.0.read_recursive().get_shared(tx_index)
    }

    /// Resume at the block containing the next transaction, or the height end
    /// when every transaction in the requested range is already computed.
    pub fn resume_height(&self, tx_len: usize, target_tx: usize, target_height: usize) -> usize {
        if tx_len >= target_tx {
            target_height
        } else {
            self.get_shared(TxIndex::from(tx_len)).unwrap().to_usize()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clones_share_directory_updates_and_reorgs() {
        let heights = TxHeights(Arc::new(RwLock::new(TxHeightMap::from(
            [0, 65_536, 131_072].map(TxIndex::new).to_vec(),
        ))));
        let reader = heights.clone();
        assert_eq!(
            reader.get_shared(TxIndex::new(100_000)),
            Some(Height::new(1))
        );
        heights.0.write().update_at(1, [TxIndex::new(200_000)]);
        assert_eq!(reader.get_shared(TxIndex::new(100_000)), Some(Height::ZERO));
        assert_eq!(
            reader.get_shared(TxIndex::new(200_000)),
            Some(Height::new(1))
        );
        // Ordered query cursors still observe the same boundary source.
        let map = reader.read();
        let mut cursor = map.cursor();
        for index in [0, 100_000, 200_000, u32::MAX] {
            assert_eq!(
                cursor.get(TxIndex::new(index)),
                map.get_shared(TxIndex::new(index))
            );
        }
    }

    #[test]
    fn resume_height_includes_partial_blocks_and_stops_at_the_target() {
        let heights = TxHeights(Arc::new(RwLock::new(TxHeightMap::from(
            [0usize, 1, 4].map(TxIndex::from).to_vec(),
        ))));
        for (tx_len, expected) in [
            (0, 0),
            (1, 1),
            (2, 1),
            (3, 1),
            (4, 2),
            (5, 2),
            (6, 3),
            (7, 3),
        ] {
            assert_eq!(heights.resume_height(tx_len, 6, 3), expected);
        }
        assert_eq!(heights.resume_height(4, 4, 2), 2);
        assert_eq!(heights.resume_height(0, 0, 0), 0);
    }
}
