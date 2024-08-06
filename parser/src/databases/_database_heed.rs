use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

use allocative::Allocative;

use heed::{BytesDecode, BytesEncode, Database, Env, EnvOpenOptions, RoTxn};

use crate::io::OUTPUTS_FOLDER_PATH;

#[derive(Allocative)]
#[allocative(bound = "Key: Allocative, Value: Allocative, KeyDB, ValueDB")]
/// There is no `cached_gets` since it's much cheaper and faster to do a parallel search first using `unsafe_get` than caching gets along the way.
pub struct HeedDatabase<Key, Value, KeyDB, ValueDB> {
    pub cached_puts: BTreeMap<Key, Value>,
    pub cached_dels: BTreeSet<Key>,
    #[allocative(skip)]
    env: Env,
    #[allocative(skip)]
    txn: Option<RoTxn<'static>>,
    #[allocative(skip)]
    db: Database<KeyDB, ValueDB>,
}

// unsafe impl<Key, Value, KeyDB, ValueDB> Sync for HeedDatabase<Key, Value, KeyDB, ValueDB> {}

impl<Key, Value, KeyDB, ValueDB> HeedDatabase<Key, Value, KeyDB, ValueDB>
where
    Key: Ord + Clone,
    KeyDB: 'static,
    ValueDB: 'static,
{
    pub fn open(folder: &str, name: &str) -> color_eyre::Result<Self> {
        let joined = format!("{folder}/{name}");
        let path_str = databases_folder_path(&joined);
        let path = Path::new(&path_str);

        fs::create_dir_all(path)?;

        let env = unsafe { EnvOpenOptions::new().open(path)? };

        let env = env.clone();

        let mut txn = env.write_txn()?;

        let db = env.create_database(&mut txn, None)?;

        txn.commit()?;

        let txn = env.clone().static_read_txn().unwrap();

        Ok(Self {
            cached_puts: BTreeMap::default(),
            cached_dels: BTreeSet::default(),
            env,
            txn: Some(txn),
            db,
        })
    }

    pub fn iter<'a, F>(&'a self, callback: &mut F)
    where
        F: FnMut((Key, Value)),
        KeyDB: BytesDecode<'a, DItem = Key>,
        ValueDB: BytesDecode<'a, DItem = Value>,
    {
        self.db
            .iter(self.txn.as_ref().unwrap())
            .unwrap()
            .map(|res| res.unwrap())
            .for_each(callback);
    }

    #[inline(always)]
    pub fn get<'a>(&'a self, key: &'a Key) -> Option<Value>
    where
        Value: Clone,
        KeyDB: BytesEncode<'a, EItem = Key> + BytesDecode<'a, DItem = Key>,
        ValueDB: BytesDecode<'a, DItem = Value>,
    {
        if let Some(cached_put) = self.get_from_puts(key) {
            return Some(cached_put.clone());
        }

        self.db_get(key)
    }

    #[inline(always)]
    pub fn db_get<'a>(&'a self, key: &'a Key) -> Option<Value>
    where
        KeyDB: BytesEncode<'a, EItem = Key> + BytesDecode<'a, DItem = Key>,
        ValueDB: BytesDecode<'a, DItem = Value>,
    {
        self.db.get(self.txn.as_ref().unwrap(), key).unwrap()
    }

    #[inline(always)]
    pub fn _db_get<'a>(&'a self, key: &'a Key) -> Option<Value>
    where
        KeyDB: BytesEncode<'a, EItem = Key> + BytesDecode<'a, DItem = Key>,
        ValueDB: BytesDecode<'a, DItem = Value>,
    {
        self.db.get(self.txn.as_ref().unwrap(), key).unwrap()
    }

    #[inline(always)]
    pub fn get_from_puts(&self, key: &Key) -> Option<&Value> {
        self.cached_puts.get(key)
    }

    #[inline(always)]
    pub fn get_mut_from_puts(&mut self, key: &Key) -> Option<&mut Value> {
        self.cached_puts.get_mut(key)
    }

    #[inline(always)]
    pub fn remove(&mut self, key: &Key) -> Option<Value> {
        self.remove_from_puts(key).or_else(|| {
            self.db_remove(key);

            None
        })
    }

    #[inline(always)]
    pub fn remove_from_puts(&mut self, key: &Key) -> Option<Value> {
        self.cached_puts.remove(key)
    }

    #[inline(always)]
    pub fn db_remove(&mut self, key: &Key) {
        self.cached_dels.insert(key.clone());
    }

    #[inline(always)]
    pub fn update(&mut self, key: Key, value: Value) -> Option<Value> {
        self.cached_dels.insert(key.clone());
        self.cached_puts.insert(key, value)
    }

    #[inline(always)]
    pub fn insert(&mut self, key: Key, value: Value) -> Option<Value> {
        self.cached_dels.remove(&key);

        self.unsafe_insert(key, value)
    }

    #[inline(always)]
    pub fn unsafe_insert(&mut self, key: Key, value: Value) -> Option<Value> {
        self.cached_puts.insert(key, value)
    }

    pub fn export<'a>(&'a mut self) -> color_eyre::Result<()>
    where
        KeyDB: BytesEncode<'a, EItem = Key>,
        ValueDB: BytesEncode<'a, EItem = Value>,
    {
        let db = self.db;

        self.txn.take().unwrap();
        // self.txn.take().unwrap().commit().unwrap();

        let env = self.env.clone();
        let mut txn = env.write_txn().unwrap();

        if self.cached_dels.is_empty() && self.cached_puts.is_empty() {
            return Ok(());
        }

        self.cached_dels
            .iter()
            .try_for_each(|key| -> color_eyre::Result<()> {
                db.delete(&mut txn, key)?;
                Ok(())
            })?;

        self.cached_puts
            .iter()
            .try_for_each(|(key, value)| -> color_eyre::Result<()> {
                db.put(&mut txn, key, value)?;
                Ok(())
            })?;

        txn.commit()?;

        Ok(())
    }
}

fn databases_folder_path(folder: &str) -> String {
    format!("{OUTPUTS_FOLDER_PATH}/databases/{folder}")
}
