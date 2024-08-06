use std::{
    collections::BTreeMap,
    mem,
    ops::{Deref, DerefMut},
};

use allocative::Allocative;
use heed::{
    byteorder::NativeEndian,
    types::{SerdeBincode, U32},
};
use rayon::prelude::*;

use crate::structs::{Date, Height, TxoutIndex};

use super::{AnyDatabaseGroup, HeedDatabase, Metadata};

type Key = TxoutIndex;
type Value = u32;
type KeyDB = SerdeBincode<TxoutIndex>;
type ValueDB = U32<NativeEndian>;
type Database = HeedDatabase<Key, Value, KeyDB, ValueDB>;

#[derive(Allocative)]
pub struct TxoutIndexToAddressIndex {
    pub metadata: Metadata,

    map: BTreeMap<usize, Database>,
}

impl Deref for TxoutIndexToAddressIndex {
    type Target = BTreeMap<usize, Database>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl DerefMut for TxoutIndexToAddressIndex {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
    }
}

const DB_MAX_SIZE: usize = 10_000_000_000;

impl TxoutIndexToAddressIndex {
    pub fn unsafe_insert(&mut self, key: Key, value: Value) -> Option<Value> {
        self.metadata.called_insert();

        self.open_db(&key).unsafe_insert(key, value)
    }

    // pub fn undo_insert(&mut self, key: &Key) -> Option<Value> {
    //     self.open_db(key).remove_from_puts(key).map(|v| {
    //         self.metadata.called_remove();

    //         v
    //     })
    // }

    pub fn remove(&mut self, key: &Key) -> Option<Value> {
        self.metadata.called_remove();

        self.open_db(key).remove(key)
    }

    /// Doesn't check if the database is open contrary to `safe_get` which does and opens if needed
    /// Though it makes it easy to use with rayon.
    pub fn unsafe_get(&self, key: &Key) -> Option<Value> {
        let db_index = Self::db_index(key);

        self.get(&db_index).unwrap().get(key)
    }

    pub fn open_db(&mut self, key: &Key) -> &mut Database {
        let db_index = Self::db_index(key);

        self.entry(db_index).or_insert_with(|| {
            let db_name = format!(
                "{}..{}",
                db_index * DB_MAX_SIZE,
                (db_index + 1) * DB_MAX_SIZE
            );

            Database::open(Self::folder(), &db_name).unwrap()
        })
    }

    fn db_index(key: &Key) -> usize {
        key.as_u64() as usize / DB_MAX_SIZE
    }
}

impl AnyDatabaseGroup for TxoutIndexToAddressIndex {
    fn import() -> Self {
        Self {
            map: BTreeMap::default(),
            metadata: Metadata::import(&Self::full_path()),
        }
    }

    fn export(&mut self, height: Height, date: Date) -> color_eyre::Result<()> {
        mem::take(&mut self.map)
            .par_iter_mut()
            .try_for_each(|(_, db)| db.export())?;

        self.metadata.export(height, date)?;

        Ok(())
    }

    fn reset_metadata(&mut self) {
        self.metadata.reset();
    }

    fn folder<'a>() -> &'a str {
        "txout_index_to_address_index"
    }
}
