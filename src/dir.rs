use crate::*;

// Misc
pub const AUTH_COOKIE: &'static str = "Auth";
pub const APP_NAME: &'static str = "Senior Duke";

// Login
pub const LOGIN_PAGE: &'static str = "/login";
pub const LOGIN_TITLE: &'static str = "Login";

// Top Level
pub const ORGS_PAGE: &'static str = "/orgs";
pub const ORGS_TITLE: &'static str = "Organisations";

pub const OA_PAGE: &'static str = "/achievements";
pub const OA_TITLE: &'static str = "Outstanding Achievements";

pub const ADD_ORG_PAGE: &'static str = "/add_org";
pub const ADD_CREDIT_PATH: &'static str = "/add_credit";

// Organisation
pub const ORG_ROOT_PATH: &'static str = "/org";

pub fn org_path(org_id: org::OrgKey) -> String {
    ORG_ROOT_PATH.to_owned() + "/" + &org_id.to_string()
}

pub const CLIENTS_PAGE: &'static str = "/clients";
pub const CLIENTS_TITLE: &'static str = "Clients";

pub const ASSOCIATES_PAGE: &'static str = "/associates";
pub const ASSOCIATES_TITLE: &'static str = "Associates";

pub const ADD_TEACHER_PAGE: &'static str = "/add_teacher";

// Client
pub const CLIENT_ROOT_PATH: &'static str = "/client";

pub fn client_path(org_id: org::OrgKey, user_id: user::UserKey) -> String {
    org_path(org_id) + CLIENT_ROOT_PATH + "/" + &user_id.to_string()
}

pub const SECTIONS_TITLE: &'static str = "Sections";