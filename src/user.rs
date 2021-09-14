use crate::{db, define_uuid_key, dir, org::OrgKey, section::SectionKey};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub email: String,

    pub forename: String,
    pub surname: String,

    pub notifications: bool,

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
    AdminLevel,
    OrgLevel,
    ClientLevel,
}

impl Privilege {
    pub fn magnitude(&self) -> i32 {
        match self {
            Privilege::ClientLevel => 1,
            Privilege::OrgLevel => 2,
            Privilege::AdminLevel => 3,
            Privilege::RootLevel => 4,
        }
    }

    pub fn is_root(&self) -> bool {
        if let Privilege::RootLevel = self {
            true
        } else {
            false
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
        award: String,
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

    pub fn is_owner(&self) -> bool {
        if let UserAgent::Owner = self {
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
            UserAgent::Admin => Privilege::AdminLevel,
            UserAgent::Organisation(_) => Privilege::OrgLevel,
            UserAgent::Associate(_) => Privilege::OrgLevel,
            UserAgent::Client { .. } => Privilege::ClientLevel,
        }
    }

    pub fn lower_string(&self) -> String {
        match self {
            UserAgent::Owner => "owner".to_owned(),
            UserAgent::Admin => "admin".to_owned(),
            UserAgent::Organisation(_) => "organisation administrator".to_owned(),
            UserAgent::Associate(_) => "teacher".to_owned(),
            UserAgent::Client { .. } => "pupil".to_owned(),
        }
    }

    pub fn can_delete_invalid_users(&self) -> bool {
        match self {
            UserAgent::Owner => true,
            UserAgent::Admin => true,
            _ => false,
        }
    }

    pub fn can_view_accounts(&self) -> bool {
        match self {
            UserAgent::Owner => true,
            UserAgent::Admin => true,
            _ => false,
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

    pub fn can_add_admin(&self) -> bool {
        match self {
            UserAgent::Owner => true,
            _ => false,
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
