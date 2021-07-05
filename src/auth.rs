use serde::{Serialize, Deserialize};

use crate::user::{User, UserKey, UserAgent};
use crate::{db, dir, define_uuid_key};
use std::time::{SystemTime, Duration};
use std::path::Path;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct AuthSession {
    user_id: UserKey,
    expiry: SystemTime,
}

define_uuid_key!(AuthToken);

pub type AuthDb = db::Database<AuthToken, AuthSession>;

pub struct AuthManager {
    db: AuthDb,
}

impl AuthManager {
    pub fn open<P: AsRef<Path>>(path: P) -> sled::Result<Self> {
        Ok(
            Self {
                db: AuthDb::open(path)?,
            }
        )
    }

    pub fn db(&self) -> &AuthDb {
        &self.db
    }

    pub fn create_session(&self, user_id: &UserKey, timeout: Duration) -> Result<AuthToken, db::Error> {
        let token = AuthToken::generate();

        let session = AuthSession {
            user_id: *user_id,
            expiry: SystemTime::now() + timeout,
        };

        self.db().insert(&token, &session)?;

        Ok(token)
    }

    pub fn check_token(&self, token: &AuthToken) -> Result<Option<UserKey>, db::Error> {
        let session = self.db().fetch(token)?;
        match session {
            Some(s) => {
                if s.expiry > SystemTime::now() {
                    Ok(Some(s.user_id))
                } else {
                    // If the key is expired but still in the table, remove it.
                    match self.db().remove_silent(token) {
                        Ok(_) => Ok(None),
                        Err(e) => Err(db::Error::DbError(e)),
                    }
                }
            },
            None => Ok(None),
        }        
    }

    pub fn clear_expired_sessions(&self) {
        self.db().retain(false, |v| {
            if v.expiry > SystemTime::now() {
                true
            } else {
                false
            }
        });
    }
}

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub auth_token: AuthToken,
    pub user_id: UserKey,
    pub user: User,
}

impl AuthContext {
    pub fn home_page(&self) -> String {
        match self.user.user_agent {
            UserAgent::Owner => dir::ORGS_PAGE.to_owned(),
            UserAgent::Admin => dir::ORGS_PAGE.to_owned(),
            UserAgent::Orginisation(org_id) => dir::ORG_ROOT_PATH.to_owned() + "/" + &org_id.to_string(),
            UserAgent::Associate(org_id) => dir::ORG_ROOT_PATH.to_owned() + "/" + &org_id.to_string(),
            UserAgent::Client(org_id) => dir::ORG_ROOT_PATH.to_owned()
        }
    }
}