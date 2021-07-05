use serde::{Serialize, de::DeserializeOwned};
use std::marker;
use std::path::Path;
use std::fmt;

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
    }
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
    _value: marker::PhantomData<K>,
    _key: marker::PhantomData<V>,
}

impl<K: AsRef<[u8]> + ?Sized, V: Serialize + DeserializeOwned> Database<K, V> {
    /// It is not guaranteed that each value in the db can be deserialized to type T.
    pub fn open<P: AsRef<Path>>(path: P) -> sled::Result<Self> {
        Ok(
            Self {
                db: sled::open(path)?,
                _value: marker::PhantomData,
                _key: marker::PhantomData,
            }
        )
    }

    pub fn raw_db(&self) -> &sled::Db {
        &self.db
    }

    pub fn insert(&self, key: &K, value: &V) -> Result<(), Error> {
        match bincode::serialize(value) {
            Ok(bytes) => {
                match self.db.insert(key, sled::IVec::from(bytes.as_slice())) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(Error::DbError(e)),
                }
            },
            Err(e) => Err(Error::SerializeError(e)),
        }   
    }

    pub fn fetch(&self, key: &K) -> Result<Option<V>, Error> {
        match self.db.get(key) {
            Ok(Some(bytes)) => {
                match bincode::deserialize(&bytes) {
                    Ok(v) => Ok(Some(v)),
                    Err(e) => Err(Error::DeserializeError(e)),
                }
            },
            Ok(None) => Ok(None),
            Err(e) => {
                Err(Error::DbError(e))
            },
        }
    }

    pub fn remove_silent(&self, key: &K) -> sled::Result<()> {
        self.db.remove(key)?;
        Ok(())
    }

    pub fn remove(&self, key: &K) -> Result<Option<V>, Error> {
        match self.db.remove(key) {
            Ok(Some(bytes)) => {
                match bincode::deserialize(&bytes) {
                    Ok(v) => Ok(Some(v)),
                    Err(e) => Err(Error::DeserializeError(e)),
                }
            },
            Ok(None) => Ok(None),
            Err(e) => {
                Err(Error::DbError(e))
            },
        }
    }

    /// Executes f for each value that can be deserialized in the database.
    pub fn for_each_val<F>(&self, f: F)
    where F: Fn(V) {
        for item in self.db.iter() {
            if let Ok((_, bytes)) = item {
                match bincode::deserialize(&bytes) {
                    Ok(v) => f(v),
                    Err(e) => {},
                }
            }
        }
    }

    pub fn retain<F>(&self, retain_undeserializable: bool, f: F)
    where F: Fn(V) -> bool {
        let mut deletion_list: Vec<_> = Vec::new();
        for item in self.db.iter() {
            if let Ok((raw_key, bytes)) = item {
                let retain = match bincode::deserialize(&bytes) {
                    Ok(v) => { 
                        f(v) 
                    },
                    Err(e) => retain_undeserializable,
                };

                if !retain {
                    deletion_list.push(raw_key);
                }
            }
        }

        for to_delete in deletion_list.iter() {
            self.db.remove(to_delete);
        }
    }
}

struct UuidKey(pub uuid::Uuid);

impl AsRef<[u8]> for UuidKey {
    fn as_ref(&self) -> &[u8] {
        self.0.as_bytes()
    }
}