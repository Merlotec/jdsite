use serde::{Serialize, Deserialize};
use crate::user::UserKey;
use crate::section::SectionKey;

use crate::{db, define_uuid_key};

#[derive(Debug, Serialize, Deserialize)]
pub struct Org {
    pub admin: Option<UserKey>,
    pub name: String,
    pub associates: Vec<UserKey>,
    pub clients: Vec<UserKey>,
    pub unreviewed_sections: Vec<SectionKey>,
    pub credits: u32,
}

impl Org {
    pub fn new(name: String) -> Self {
        Self {
            admin: None,
            name,
            associates: Vec::new(),
            clients: Vec::new(),
            unreviewed_sections: Vec::new(),
            credits: 0,
        }
    }
}

define_uuid_key!(OrgKey);

pub type OrgDb = db::Database<OrgKey, Org>;

