use crate::{db, define_uuid_key, org::OrgKey, section::SectionKey, dir};

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
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub enum Privilege {
    RootLevel,
    OrgLevel,
    ClientLevel,
}

impl Privilege {
    pub fn magnitude(&self) -> i32 {
        match self {
            Privilege::ClientLevel => 1,
            Privilege::OrgLevel => 2,
            Privilege::RootLevel => 3,
        }
    }
}

/// Contains all the different types of user.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserAgent {
    Owner,
    Admin,
    Organisation(OrgKey),
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

    pub fn root_page(&self, user_id: UserKey) -> String {
        match self {
            UserAgent::Owner => dir::ORGS_PAGE.to_owned(),
            UserAgent::Admin => dir::ORGS_PAGE.to_owned(),
            UserAgent::Organisation(org_id) => dir::org_path(*org_id),
            UserAgent::Associate(org_id) => dir::org_path(*org_id),
            UserAgent::Client { org_id, .. } => dir::client_path(*org_id, user_id),
        }
    }

    pub fn privilege(&self) -> Privilege {
        match self {
            UserAgent::Owner => Privilege::RootLevel,
            UserAgent::Admin => Privilege::RootLevel,
            UserAgent::Organisation(_) => Privilege::OrgLevel,
            UserAgent::Associate(_) => Privilege::OrgLevel,
            UserAgent::Client { .. } => Privilege::ClientLevel,
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
            UserAgent::Organisation(agent_org_id) => agent_org_id == org_id,
            UserAgent::Associate(agent_org_id) => agent_org_id == org_id,
            UserAgent::Client { .. } => false,
        }
    }

    pub fn can_add_associate(&self, org_id: &OrgKey) -> bool {
        match self {
            UserAgent::Owner => true,
            UserAgent::Admin => true,
            UserAgent::Organisation(agent_org_id) => agent_org_id == org_id,
            UserAgent::Associate(_) => false,
            UserAgent::Client { .. } => false,
        }
    }

    pub fn can_delete_user(&self, other: &UserAgent) -> bool {
        match self {
            UserAgent::Owner => true,
            UserAgent::Admin => other != &UserAgent::Owner && other != &UserAgent::Admin,
            UserAgent::Organisation(agent_org_id) => other.org_id() == Some(*agent_org_id),
            UserAgent::Associate(_) => false,
            UserAgent::Client { .. } => false,
        }
    }

    pub fn can_view_outstanding(&self) -> bool {
        match self {
            UserAgent::Owner => true,
            UserAgent::Admin => true,
            UserAgent::Organisation(_) => false,
            UserAgent::Associate(_) => false,
            UserAgent::Client { .. } => false,
        }
    }

    pub fn can_view_user(&self, other: &UserAgent) -> bool {
        match self {
            UserAgent::Owner => true,
            UserAgent::Admin => other != &UserAgent::Owner && other != &UserAgent::Admin,
            UserAgent::Organisation(agent_org_id) => other.org_id() == Some(*agent_org_id),
            UserAgent::Associate(agent_org_id) => other.org_id() == Some(*agent_org_id),
            UserAgent::Client { .. } => false,
        }
    }

    pub fn agent_string(&self) -> String {
        match self {
            UserAgent::Owner => "Owner".to_owned(),
            UserAgent::Admin => "Global Administrator".to_owned(),
            UserAgent::Organisation(_) => "Organisation Administrator".to_owned(),
            UserAgent::Associate(_) => "Teacher".to_owned(),
            UserAgent::Client { .. } => "Pupil".to_owned(),
        }
    }

    pub fn org_id(&self) -> Option<OrgKey> {
        match self {
            UserAgent::Owner => None,
            UserAgent::Admin => None,
            UserAgent::Organisation(org_id) => Some(*org_id),
            UserAgent::Associate(org_id) => Some(*org_id),
            UserAgent::Client { org_id, .. } => Some(*org_id),
        }
    }

    
}

define_uuid_key!(UserKey);

pub type UserDb = db::Database<UserKey, User>;