use brk_types::TxidPrefix;

use super::parent_read::ParentRead;

const SLOTS: usize = 1 << 16;

/// Bounded, disposable parent reads. Probes are shared across Rayon workers;
/// entries are filled only after a block's parent reads have succeeded.
#[derive(Default)]
pub(super) struct ParentCache {
    entries: Vec<Option<(TxidPrefix, ParentRead)>>,
}

impl ParentCache {
    pub fn get(&self, prefix: TxidPrefix) -> Option<ParentRead> {
        self.entries
            .get(Self::slot(prefix))
            .copied()
            .flatten()
            .filter(|(key, _)| *key == prefix)
            .map(|(_, read)| read)
    }

    pub fn insert(&mut self, prefix: TxidPrefix, read: ParentRead) {
        if self.entries.is_empty() {
            self.entries.resize(SLOTS, None);
        }
        self.entries[Self::slot(prefix)] = Some((prefix, read));
    }

    pub fn invalidate(&mut self, prefix: TxidPrefix) {
        if let Some(entry) = self.entries.get_mut(Self::slot(prefix))
            && entry.is_some_and(|(key, _)| key == prefix)
        {
            *entry = None;
        }
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    fn slot(prefix: TxidPrefix) -> usize {
        *prefix as usize & (SLOTS - 1)
    }
}
