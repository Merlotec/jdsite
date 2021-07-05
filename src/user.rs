use std::path::Path;
use sled::Db;
use uuid::Uuid;

use crate::{db, define_uuid_key, org::OrgKey};

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub email: String,

    pub forename: String,
    pub surname: String,
    
    pub user_agent: UserAgent,
}

/// Contains all the different types of user.
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum UserAgent {
    Owner,
    Admin,
    Orginisation(OrgKey),
    Associate(OrgKey),
    Client(OrgKey),
}

define_uuid_key!(UserKey);

pub type UserDb = db::Database<UserKey, User>;