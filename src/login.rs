use crate::{db, db::Database, user::UserKey};
use std::fmt;
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct LoginEntry {
    pub password: String,
    pub user_id: UserKey,
    pub default_password: bool,
}

#[derive(Debug)]
pub enum AuthError {
    NoUser,
    IncorrectPassword,
    DbError(db::Error),
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AuthError::NoUser => write!(f, "No matching username found!"),
            AuthError::IncorrectPassword => write!(f, "Incorrect password for username!"),
            AuthError::DbError(e) => e.fmt(f),
        }
    }
}

#[derive(Debug)]
pub enum LoginEntryError {
    NotUnique,
    UsernameExists,
    //PasswordInvalid(String),
    DbError(db::Error),
}

impl fmt::Display for LoginEntryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LoginEntryError::UsernameExists => {
                write!(f, "An account with the given username already exists!")
            }
            LoginEntryError::NotUnique => write!(
                f,
                "An account with similar characteristics exists where it should not!"
            ),
            //LoginEntryError::PasswordInvalid(ref s) => write!(f, "Password invalid: {}!", s),
            LoginEntryError::DbError(e) => e.fmt(f),
        }
    }
}

pub struct LoginDb(Database<str, LoginEntry>);

impl LoginDb {
    pub fn open<P: AsRef<Path>>(path: P) -> sled::Result<Self> {
        Ok(Self(Database::open(path)?))
    }

    pub fn db(&self) -> &Database<str, LoginEntry> {
        &self.0
    }

    pub fn authenticate(&self, username: &str, password: &str) -> Result<UserKey, AuthError> {
        match self.db().fetch(username) {
            Ok(Some(entry)) => {
                if entry.password == password {
                    Ok(entry.user_id)
                } else {
                    Err(AuthError::IncorrectPassword)
                }
            }
            Ok(None) => Err(AuthError::NoUser),
            Err(e) => Err(AuthError::DbError(e)),
        }
    }

    pub fn add_entry(&self, username: &str, entry: &LoginEntry) -> Result<(), LoginEntryError> {
        match self.db().raw_db().contains_key(username) {
            Ok(contains_key) => {
                if contains_key {
                    Err(LoginEntryError::UsernameExists)
                } else {
                    match self.db().insert(username, entry) {
                        Ok(_) => Ok(()),
                        Err(e) => Err(LoginEntryError::DbError(e)),
                    }
                }
            }
            Err(e) => Err(LoginEntryError::DbError(db::Error::DbError(e))),
        }
    }

    pub fn change_password(
        &self,
        username: &str,
        password: &str,
        default_password: bool,
    ) -> Result<(), AuthError> {
        match self.db().fetch(username) {
            Ok(Some(mut entry)) => {
                entry.password = password.to_owned();
                entry.default_password = default_password;
                match self.db().insert(username, &entry) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(AuthError::DbError(e)),
                }
            }
            Ok(None) => Err(AuthError::NoUser),
            Err(e) => Err(AuthError::DbError(e)),
        }
    }
}
