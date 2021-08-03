use crate::user::{UserAgent, UserKey};
use crate::{db, define_uuid_key};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Link {
    CreateUser(UserAgent),
    ChangePassword(UserKey),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkEntry {
    pub link: Link,
    pub expiry: SystemTime,
    pub timeout: Duration,
}

define_uuid_key!(LinkToken);

pub type LinkDb = db::Database<LinkToken, LinkEntry>;

pub struct LinkManager {
    db: LinkDb,
}

impl LinkManager {
    pub fn open<P: AsRef<Path>>(path: P) -> sled::Result<Self> {
        Ok(Self {
            db: LinkDb::open(path)?,
        })
    }

    pub fn db(&self) -> &LinkDb {
        &self.db
    }

    pub fn create_link(&self, link: Link, timeout: Duration) -> Result<LinkToken, db::Error> {
        let token = LinkToken::generate();

        let session = LinkEntry {
            link,
            expiry: SystemTime::now() + timeout,
            timeout,
        };

        self.db().insert(&token, &session)?;

        Ok(token)
    }

    pub fn destroy_link(&self, token: &LinkToken) -> sled::Result<()> {
        self.db().remove_silent(token)
    }

    pub fn fetch(&self, token: &LinkToken) -> Result<Option<LinkEntry>, db::Error> {
        let session = self.db().fetch(token)?;
        match session {
            Some(l) => {
                if l.expiry > SystemTime::now() {
                    Ok(Some(l))
                } else {
                    // If the key is expired but still in the table, remove it.
                    match self.db().remove_silent(token) {
                        Ok(_) => Ok(None),
                        Err(e) => Err(db::Error::DbError(e)),
                    }
                }
            }
            None => Ok(None),
        }
    }

    pub fn clear_expired_links(&self) {
        self.db().retain(false, |v| {
            if v.expiry > SystemTime::now() {
                true
            } else {
                false
            }
        });
    }
}
