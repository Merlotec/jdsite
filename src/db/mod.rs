use serde::{de::DeserializeOwned, Serialize};
use std::convert::TryFrom;
use std::fmt;
use std::marker;
use std::path::Path;
use std::sync::Mutex;

#[macro_export]
macro_rules! define_uuid_key {
    ($T:ident) => {
        #[derive(Debug, Copy, Clone, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
        pub struct $T(pub uuid::Uuid);

        impl $T {
            pub fn generate() -> Self {
                Self(uuid::Uuid::new_v4())
            }
        }

        impl std::str::FromStr for $T {
            type Err = <uuid::Uuid as std::str::FromStr>::Err;
            fn from_str(s: &str) -> Result<$T, Self::Err> {
                Ok(Self(uuid::Uuid::from_str(s)?))
            }
        }

        impl AsRef<[u8]> for $T {
            fn as_ref(&self) -> &[u8] {
                self.0.as_bytes()
            }
        }

        impl ToString for $T {
            fn to_string(&self) -> String {
                self.0.to_string()
            }
        }

        impl std::convert::TryFrom<sled::IVec> for $T {
            type Error = std::array::TryFromSliceError;
            fn try_from(vec: sled::IVec) -> Result<$T, Self::Error> {
                Ok(Self(uuid::Uuid::from_bytes(<[u8; 16]>::try_from(
                    <sled::IVec as AsRef<[u8]>>::as_ref(&vec),
                )?)))
            }
        }
    };
}

#[derive(Debug)]
pub enum Error {
    DeserializeError(bincode::Error),
    SerializeError(bincode::Error),
    DbError(sled::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::DeserializeError(e) => e.fmt(f),
            Error::SerializeError(e) => e.fmt(f),
            // The wrapped error contains additional information and is available
            // via the source() method.
            Error::DbError(e) => e.fmt(f),
        }
    }
}

/// A simple wrapper aroudn the sled db to allow us to use aribrary types.
pub struct Database<K: AsRef<[u8]> + ?Sized, V: Serialize + DeserializeOwned> {
    db: sled::Db,
    lock_map: Mutex<Vec<sled::IVec>>,
    _value: marker::PhantomData<K>,
    _key: marker::PhantomData<V>,
}

impl<K: AsRef<[u8]> + ?Sized, V: Serialize + DeserializeOwned> Database<K, V> {
    /// It is not guaranteed that each value in the db can be deserialized to type T.
    pub fn open<P: AsRef<Path>>(path: P) -> sled::Result<Self> {
        Ok(Self {
            db: sled::open(path)?,
            lock_map: Mutex::new(Vec::new()),
            _value: marker::PhantomData,
            _key: marker::PhantomData,
        })
    }

    pub fn raw_db(&self) -> &sled::Db {
        &self.db
    }

    pub fn insert(&self, key: &K, value: &V) -> Result<(), Error> {
        match bincode::serialize(value) {
            Ok(bytes) => match self.db.insert(key, sled::IVec::from(bytes.as_slice())) {
                Ok(_) => Ok(()),
                Err(e) => Err(Error::DbError(e)),
            },
            Err(e) => Err(Error::SerializeError(e)),
        }
    }

    pub fn insert_raw(&self, key_bytes: &sled::IVec, value: &V) -> Result<(), Error> {
        match bincode::serialize(value) {
            Ok(bytes) => {
                match self
                    .db
                    .insert(key_bytes, sled::IVec::from(bytes.as_slice()))
                {
                    Ok(_) => Ok(()),
                    Err(e) => Err(Error::DbError(e)),
                }
            }
            Err(e) => Err(Error::SerializeError(e)),
        }
    }

    fn lock(&self, key_bytes: sled::IVec) {
        loop {
            if let Ok(mut list) = self.lock_map.try_lock() {
                if !list.contains(&key_bytes) {
                    list.push(key_bytes);
                    return;
                }
            }
            std::thread::sleep(std::time::Duration::from_nanos(200));
        }
    }

    fn unlock(&self, key_bytes: &sled::IVec) {
        if let Ok(mut list) = self.lock_map.lock() {
            list.retain(|x| x != key_bytes);
        }
    }

    pub fn write_lock<'db>(&'db self, key: &K) -> Result<Option<WriteGuard<'db, K, V>>, Error> {
        WriteGuard::lock(self, key)
    }

    pub fn fetch(&self, key: &K) -> Result<Option<V>, Error> {
        self.fetch_into::<V>(key)
    }

    fn fetch_into<O: DeserializeOwned>(&self, key: &K) -> Result<Option<O>, Error> {
        match self.db.get(key) {
            Ok(Some(bytes)) => match bincode::deserialize::<'_, O>(&bytes) {
                Ok(v) => Ok(Some(v)),
                Err(e) => Err(Error::DeserializeError(e)),
            },
            Ok(None) => Ok(None),
            Err(e) => Err(Error::DbError(e)),
        }
    }

    pub fn remove_silent(&self, key: &K) -> sled::Result<()> {
        self.db.remove(key)?;
        Ok(())
    }

    pub fn remove(&self, key: &K) -> Result<Option<V>, Error> {
        match self.db.remove(key) {
            Ok(Some(bytes)) => match bincode::deserialize(&bytes) {
                Ok(v) => Ok(Some(v)),
                Err(e) => Err(Error::DeserializeError(e)),
            },
            Ok(None) => Ok(None),
            Err(e) => Err(Error::DbError(e)),
        }
    }

    pub fn contains_key(&self, key: &K) -> sled::Result<bool> {
        self.db.contains_key(key)
    }

    /// Executes f for each value that can be deserialized in the database.
    pub fn for_each_val<F>(&self, mut f: F)
    where
        F: FnMut(V),
    {
        for item in self.db.iter() {
            if let Ok((_, bytes)) = item {
                match bincode::deserialize(&bytes) {
                    Ok(v) => f(v),
                    Err(_) => {}
                }
            }
        }
    }

    /// Only executes if both key and value can be properly deserialised.
    pub fn for_each<F, OwnedKey>(&self, mut f: F)
    where
        K: ToOwned<Owned = OwnedKey>,
        OwnedKey: TryFrom<sled::IVec> + Sized,
        F: FnMut(&OwnedKey, V),
    {
        for item in self.db.iter() {
            if let Ok((key, bytes)) = item {
                match OwnedKey::try_from(key) {
                    Ok(k) => match bincode::deserialize(&bytes) {
                        Ok(v) => f(&k, v),
                        Err(_) => {}
                    },
                    Err(_) => {}
                }
                
            }
        }
    }

    pub fn for_each_key<F, OwnedKey>(&self, mut f: F) 
    where
        K: ToOwned<Owned = OwnedKey>,
        OwnedKey: TryFrom<sled::IVec> + Sized,
        F: FnMut(&OwnedKey, Option<V>),
    {
        for item in self.db.iter() {
            if let Ok((key, bytes)) = item {
                match OwnedKey::try_from(key) {
                    Ok(k) => match bincode::deserialize(&bytes) {
                        Ok(v) => f(&k, Some(v)),
                        Err(_) => f(&k, None),
                    },
                    Err(_) => {}
                }
                
            }
        }
    }

    pub fn for_each_write<F>(&self, mut f: F)
    where
        F: for<'db> FnMut(WriteGuard<'db, K, V>),
    {
        for item in self.db.iter() {
            if let Ok((key, bytes)) = item {
                match bincode::deserialize(&bytes) {
                    Ok(v) => {
                        let guard = WriteGuard::lock_raw(&self, key, v);
                        f(guard);
                    }
                    Err(_) => {}
                }
            }
        }
    }

    pub fn retain<F>(&self, retain_undeserializable: bool, f: F)
    where
        F: Fn(V) -> bool,
    {
        let mut deletion_list: Vec<_> = Vec::new();
        for item in self.db.iter() {
            if let Ok((raw_key, bytes)) = item {
                let retain = match bincode::deserialize(&bytes) {
                    Ok(v) => f(v),
                    Err(_) => retain_undeserializable,
                };

                if !retain {
                    deletion_list.push(raw_key);
                }
            }
        }

        for to_delete in deletion_list.iter() {
            let _ = self.db.remove(to_delete);
        }
    }

    pub fn migrate<F, OwnedKey, O: DeserializeOwned>(&self, f: F)
    where
        K: ToOwned<Owned = OwnedKey>,
        OwnedKey: AsRef<K> + TryFrom<sled::IVec> + Sized,
        F: Fn(&OwnedKey, O) -> V,
    {
        for item in self.db.iter() {
            if let Ok((key, bytes)) = item {
                match bincode::deserialize::<'_, O>(&bytes) {
                    Ok(v) => match OwnedKey::try_from(key) {
                        Ok(k) => {
                            let new_val = f(&k, v);
                            let _ = self.insert(k.as_ref(), &new_val);
                        }
                        Err(_) => {}
                    },
                    Err(_) => {}
                }
            }
        }
    }
}

struct UuidKey(pub uuid::Uuid);

impl AsRef<[u8]> for UuidKey {
    fn as_ref(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

/// Only writes if the data is fetched in a mutable fashion.
pub struct WriteGuard<'db, K: AsRef<[u8]> + ?Sized, V: Serialize + DeserializeOwned> {
    db: &'db Database<K, V>,
    key_bytes: sled::IVec,
    value: V,
    mutated: bool,
    _key: marker::PhantomData<K>,
}

impl<'db, K: AsRef<[u8]> + ?Sized, V: Serialize + DeserializeOwned> WriteGuard<'db, K, V> {
    pub fn lock(db: &'db Database<K, V>, key: &K) -> Result<Option<Self>, Error> {
        let fetch = db.fetch(key)?;
        if let Some(value) = fetch {
            // Add the write guard to the list.
            let key_bytes: sled::IVec = sled::IVec::from(key.as_ref());
            Ok(Some(Self::lock_raw(db, key_bytes, value)))
        } else {
            Ok(None)
        }
    }

    fn lock_raw(db: &'db Database<K, V>, key_bytes: sled::IVec, value: V) -> Self {
        db.lock(key_bytes.clone());

        Self {
            db,
            key_bytes,
            mutated: false,
            value,
            _key: marker::PhantomData,
        }
    }

    pub fn key<OwnedKey, E>(&self) -> Result<OwnedKey, E>
    where
        K: ToOwned<Owned = OwnedKey>,
        OwnedKey: TryFrom<sled::IVec, Error = E> + Sized,
    {
        OwnedKey::try_from(self.key_bytes.clone())
    }

    #[inline]
    pub fn value(&self) -> &V {
        &self.value
    }

    #[inline]
    pub fn value_mut(&mut self) -> &mut V {
        self.mutated = true;
        &mut self.value
    }
}

impl<'db, K: AsRef<[u8]> + ?Sized, V: Serialize + DeserializeOwned> std::ops::Deref
    for WriteGuard<'db, K, V>
{
    type Target = V;
    fn deref(&self) -> &Self::Target {
        self.value()
    }
}

impl<'db, K: AsRef<[u8]> + ?Sized, V: Serialize + DeserializeOwned> std::ops::DerefMut
    for WriteGuard<'db, K, V>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value_mut()
    }
}

impl<'db, K: AsRef<[u8]> + ?Sized, V: Serialize + DeserializeOwned> Drop for WriteGuard<'db, K, V> {
    fn drop(&mut self) {
        if self.mutated {
            if let Err(e) = self.db.insert_raw(&self.key_bytes, &self.value) {
                println!(
                    "Failed to re-insert value into database with WriteGuard: {}",
                    e
                );
            }
        }
        self.db.unlock(&self.key_bytes)
    }
}
