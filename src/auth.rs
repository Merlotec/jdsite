use serde::{Serialize, Deserialize};

use crate::user::{User, UserKey};
use crate::{db, dir, define_uuid_key, org};
use std::time::{SystemTime, Duration};
use std::path::Path;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct AuthSession {
    user_id: UserKey,
    expiry: SystemTime,
    timeout: Duration,
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
            timeout,
        };

        self.db().insert(&token, &session)?;

        Ok(token)
    }

    pub fn destroy_session(&self, token: &AuthToken) -> sled::Result<()> {
        self.db().remove_silent(token)
    }

    pub fn check_token(&self, token: &AuthToken, push_expiry: bool) -> Result<Option<UserKey>, db::Error> {
        let session = self.db().fetch(token)?;
        match session {
            Some(mut s) => {
                if s.expiry > SystemTime::now() {
                    if push_expiry {
                        s.expiry = SystemTime::now() + s.timeout;
                        // Attempt to insert updated entry.
                        let _ = self.db.insert(token, &s);
                    }
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
    pub fn root_page(&self) -> String {
        self.user.user_agent.root_page(self.user_id)
    }

    pub fn org_items(&self, org_id: org::OrgKey, org: &org::Org) -> Vec<(String, String)> {
        vec![
            (dir::org_path(org_id) + dir::CLIENTS_PAGE, "Pupils (".to_owned() + &org.clients.len().to_string() + ")"),
            (dir::org_path(org_id) + dir::ASSOCIATES_PAGE, "Teachers (".to_owned() + &org.associates.len().to_string() + ")"),
            (dir::org_path(org_id) + dir::UNREVIEWED_SECTIONS_PAGE, "Unreviewed Sections (".to_owned() + &org.unreviewed_sections.len().to_string() + ")"),
        ]
    }
}