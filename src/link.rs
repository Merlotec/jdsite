use serde::{Serialize, Deserialize};
use crate::user::{UserAgent, UserKey};

use crate::{db, define_uuid_key};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Link {
    CreateUser(UserAgent),
    ResetPassword(UserKey),
}

define_uuid_key!(LinkToken);

pub type OrgDb = db::Database<LinkToken, Link>;
