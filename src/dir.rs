use crate::*;

// Misc
pub const AUTH_COOKIE: &'static str = "Auth";
pub const LOGIN_REDIRECT_COOKIE: &'static str = "LoginRedirect";
pub const APP_NAME: &'static str = "Senior Duke Portal";

pub const USER_ROOT_PATH: &'static str = "/user";

pub const LINK_BASE_PATH: &'static str = "/link";
pub const CHANGE_PASSWORD_PATH: &'static str = "/user/change_password";

pub fn user_path(user_id: user::UserKey) -> String {
    USER_ROOT_PATH.to_owned() + "/" + &user_id.to_string()
}

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

pub const DELETE_USER_PATH: &'static str = "/delete_user";

// Organisation
pub const ORG_ROOT_PATH: &'static str = "/org";

pub fn org_path(org_id: org::OrgKey) -> String {
    ORG_ROOT_PATH.to_owned() + "/" + &org_id.to_string()
}

pub const ADD_ORG_PATH: &'static str = "/add_org";
pub const DELETE_ORG_PATH: &'static str = "/del_org";
pub const ADD_CREDITS_PATH: &'static str = "/add_credits";
pub const ASSIGN_ADMIN_PATH: &'static str = "/assign_admin";

pub const CLIENTS_PAGE: &'static str = "/clients";
pub const CLIENTS_TITLE: &'static str = "Pupils";
pub const ADD_CLIENT_PATH: &'static str = "/add_client";

pub const ASSOCIATES_PAGE: &'static str = "/associates";
pub const ASSOCIATES_TITLE: &'static str = "Teachers";

pub const UNREVIEWED_SECTIONS_PAGE: &'static str = "/unreviewed";

pub const ADD_ASSOCIATE_PATH: &'static str = "/add_associate";

// Client
pub const CLIENT_ROOT_PATH: &'static str = "/client";

pub fn client_path(org_id: org::OrgKey, user_id: user::UserKey) -> String {
    org_path(org_id) + CLIENT_ROOT_PATH + "/" + &user_id.to_string()
}

pub const SECTIONS_TITLE: &'static str = "Sections";