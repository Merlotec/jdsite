use crate::{db, define_uuid_key, org::OrgKey, section::SectionKey};

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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserAgent {
    Owner,
    Admin,
    Orginisation(OrgKey),
    Associate(OrgKey),
    Client {
        org_id: OrgKey,
        class: String,
        sections: [Option<SectionKey>; 6],
    },
}

impl UserAgent {
    pub fn is_client(&self) -> bool {
        if let UserAgent::Client { .. } = self {
            true
        } else {
            false
        }
    }

    pub fn can_view_orgs(&self) -> bool {
        match self {
            UserAgent::Owner => true,
            UserAgent::Admin => true,
            _ => false,
        }
    }

    pub fn can_delete_orgs(&self) -> bool {
        match self {
            UserAgent::Owner => true,
            UserAgent::Admin => true,
            _ => false,
        }
    }

    pub fn can_view_org(&self, org_id: &OrgKey) -> bool {
        match self {
            UserAgent::Owner => true,
            UserAgent::Admin => true,
            UserAgent::Orginisation(agent_org_id) => agent_org_id == org_id,
            UserAgent::Associate(agent_org_id) => agent_org_id == org_id,
            UserAgent::Client { .. } => false,
        }
    }

    pub fn can_add_associate(&self, org_id: &OrgKey) -> bool {
        match self {
            UserAgent::Owner => true,
            UserAgent::Admin => true,
            UserAgent::Orginisation(agent_org_id) => agent_org_id == org_id,
            UserAgent::Associate(_) => false,
            UserAgent::Client { .. } => false,
        }
    }

    pub fn can_delete_user(&self, other: &UserAgent) -> bool {
        match self {
            UserAgent::Owner => true,
            UserAgent::Admin => other != &UserAgent::Owner && other != &UserAgent::Admin,
            UserAgent::Orginisation(agent_org_id) => other.org_id() == Some(*agent_org_id),
            UserAgent::Associate(_) => false,
            UserAgent::Client { .. } => false,
        }
    }

    pub fn can_view_user(&self, other: &UserAgent) -> bool {
        match self {
            UserAgent::Owner => true,
            UserAgent::Admin => other != &UserAgent::Owner && other != &UserAgent::Admin,
            UserAgent::Orginisation(agent_org_id) => other.org_id() == Some(*agent_org_id),
            UserAgent::Associate(agent_org_id) => other.org_id() == Some(*agent_org_id),
            UserAgent::Client { .. } => false,
        }
    }

    pub fn agent_string(&self) -> String {
        match self {
            UserAgent::Owner => "Owner".to_owned(),
            UserAgent::Admin => "Global Administrator".to_owned(),
            UserAgent::Orginisation(_) => "Organisation Administrator".to_owned(),
            UserAgent::Associate(_) => "Teacher".to_owned(),
            UserAgent::Client { .. } => "Pupil".to_owned(),
        }
    }

    pub fn org_id(&self) -> Option<OrgKey> {
        match self {
            UserAgent::Owner => None,
            UserAgent::Admin => None,
            UserAgent::Orginisation(org_id) => Some(*org_id),
            UserAgent::Associate(org_id) => Some(*org_id),
            UserAgent::Client { org_id, .. } => Some(*org_id),
        }
    }

    
}

define_uuid_key!(UserKey);

pub type UserDb = db::Database<UserKey, User>;