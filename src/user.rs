use crate::{db, define_uuid_key, org::OrgKey};

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub email: String,

    pub forename: String,
    pub surname: String,
    
    pub user_agent: UserAgent,
}

impl User {
    pub fn name(&self) -> String {
        self.forename.clone() + " " + &self.surname
    }
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

impl UserAgent {
    pub fn can_view_orgs(&self) -> bool {
        match self {
            UserAgent::Owner => true,
            UserAgent::Admin => true,
            _ => false,
        }
    }
}

define_uuid_key!(UserKey);

pub type UserDb = db::Database<UserKey, User>;