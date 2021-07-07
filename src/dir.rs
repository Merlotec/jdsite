use crate::*;

// Misc
pub const AUTH_COOKIE: &'static str = "Auth";
pub const LOGIN_REDIRECT_COOKIE: &'static str = "LoginRedirect";
pub const APP_NAME: &'static str = "Senior Duke Portal";

// Login
pub const LOGIN_PAGE: &'static str = "/login";
pub const LOGIN_POST_PATH: &'static str = "/login";
pub const LOGIN_TITLE: &'static str = "Login";
pub const LOGOUT_PATH: &'static str = "/logout";

// Top Level
pub const ORGS_PAGE: &'static str = "/orgs";
pub const ORGS_TITLE: &'static str = "Organisations";

pub const OA_PAGE: &'static str = "/achievements";
pub const OA_TITLE: &'static str = "Outstanding Achievements";



// Organisation
pub const ORG_ROOT_PATH: &'static str = "/org";

pub fn org_path(org_id: org::OrgKey) -> String {
    ORG_ROOT_PATH.to_owned() + "/" + &org_id.to_string()
}

pub const ADD_ORG_PATH: &'static str = "/add_org";
pub const DELETE_ORG_PATH: &'static str = "/del_org";
pub const ADD_CREDIT_PATH: &'static str = "/add_credit";
pub const ASSIGN_ADMIN_PATH: &'static str = "/assign_admin";

pub const CLIENTS_PAGE: &'static str = "/clients";
pub const CLIENTS_TITLE: &'static str = "Clients";

pub const ASSOCIATES_PAGE: &'static str = "/associates";
pub const ASSOCIATES_TITLE: &'static str = "Associates";

pub const UNREVIEWED_SECTIONS_PAGE: &'static str = "/unreviewed";

pub const ADD_ASSOCIATE_PAGE: &'static str = "/add_associate";

// Client
pub const CLIENT_ROOT_PATH: &'static str = "/client";

pub fn client_path(org_id: org::OrgKey, user_id: user::UserKey) -> String {
    org_path(org_id) + CLIENT_ROOT_PATH + "/" + &user_id.to_string()
}

pub const SECTIONS_TITLE: &'static str = "Sections";