

use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::{db, define_uuid_key};

#[derive(Debug, Serialize, Deserialize)]
pub struct Org {
    user: Uuid,
    associates: Vec<Uuid>,
    clients: Vec<Uuid>,
    unreviewed_sections: Vec<Uuid>,
}

define_uuid_key!(OrgKey);

pub type OrgDb = db::Database<OrgKey, Org>;

