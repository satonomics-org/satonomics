use std::{
    collections::BTreeMap,
    mem,
    ops::{Deref, DerefMut},
};

use allocative::Allocative;
use biter::bitcoin::Txid;
use heed::types::SerdeBincode;
use rayon::prelude::*;

use crate::structs::{Date, Height, TxData};

use super::{AnyDatabaseGroup, HeedDatabase, Metadata};

type Key = [u8; 31];
type Value = TxData;
type KeyDB = SerdeBincode<Key>;
type ValueDB = SerdeBincode<Value>;
type Database = HeedDatabase<Key, Value, KeyDB, ValueDB>;

#[derive(Allocative)]
pub struct TxidToTxData {
    pub metadata: Metadata,

    map: BTreeMap<u8, Database>,
}

impl Deref for TxidToTxData {
    type Target = BTreeMap<u8, Database>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl DerefMut for TxidToTxData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
    }
}

impl TxidToTxData {
    pub fn insert(&mut self, txid: &Txid, tx_index: Value) -> Option<Value> {
        self.metadata.called_insert();

        let txid_key = Self::txid_to_key(txid);

        self.open_db(txid).insert(txid_key, tx_index)
    }

    // pub fn safe_get(&mut self, txid: &Txid) -> Option<&Value> {
    //     let txid_key = Self::txid_to_key(txid);
    //     self.open_db(txid).get(&txid_key)
    // }

    /// Doesn't check if the database is open contrary to `safe_get` which does and opens if needed.
    /// Though it makes it easy to use with rayon
    pub fn unsafe_get(&self, txid: &Txid) -> Option<Value> {
        let txid_key = Self::txid_to_key(txid);

        let db_index = Self::db_index(txid);

        self.get(&db_index).unwrap().get(&txid_key)
    }

    // pub fn unsafe_get_from_puts(&self, txid: &Txid) -> Option<&Value> {
    //     let txid_key = Self::txid_to_key(txid);

    //     let db_index = Self::db_index(txid);

    //     self.get(&db_index).unwrap().get_from_puts(&txid_key)
    // }

    pub fn unsafe_get_mut_from_puts(&mut self, txid: &Txid) -> Option<&mut Value> {
        let txid_key = Self::txid_to_key(txid);

        let db_index = Self::db_index(txid);

        self.get_mut(&db_index)
            .unwrap()
            .get_mut_from_puts(&txid_key)
    }

    pub fn remove_from_db(&mut self, txid: &Txid) {
        self.metadata.called_remove();

        let txid_key = Self::txid_to_key(txid);

        self.open_db(txid).db_remove(&txid_key);
    }

    pub fn remove_from_puts(&mut self, txid: &Txid) {
        self.metadata.called_remove();

        let txid_key = Self::txid_to_key(txid);

        self.open_db(txid).remove_from_puts(&txid_key);
    }

    pub fn update(&mut self, txid: &Txid, tx_data: TxData) {
        let txid_key = Self::txid_to_key(txid);

        self.open_db(txid).update(txid_key, tx_data);
    }

    #[inline(always)]
    pub fn open_db(&mut self, txid: &Txid) -> &mut Database {
        let db_index = Self::db_index(txid);

        self.entry(db_index)
            .or_insert_with(|| Database::open(Self::folder(), &db_index.to_string()).unwrap())
    }

    fn txid_to_key(txid: &Txid) -> [u8; 31] {
        let mut arr = [0_u8; 31];
        arr.copy_from_slice(&txid[1..]);
        arr
    }

    fn db_index(txid: &Txid) -> u8 {
        txid[0]
    }
}

impl AnyDatabaseGroup for TxidToTxData {
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
        "txid_to_tx_data"
    }
}
