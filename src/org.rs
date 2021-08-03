use crate::section::SectionKey;
use crate::user::UserKey;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use std::time::SystemTime;

use crate::{db, define_uuid_key, dir};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Org {
    pub admin: Option<UserKey>,
    pub name: String,
    pub associates: Vec<UserKey>,
    pub clients: Vec<UserKey>,
    pub unreviewed_sections: Vec<SectionKey>,
    pub credits: u32,
    pub last_notification: SystemTime,
    pub notification_interval: Duration,
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
            last_notification: SystemTime::now(),
            notification_interval: Duration::from_secs(
                60 * 60 * 24 * dir::NOTIFICATION_INTERVAL_DAYS,
            ),
        }
    }
}

define_uuid_key!(OrgKey);

pub type OrgDb = db::Database<OrgKey, Org>;
