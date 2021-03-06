use crate::*;

pub const LOG_PATH: &'static str = "log.txt";

pub const HOST: &'static str = "seniorportal.juniorduke.com";

// Misc
pub const AUTH_COOKIE: &'static str = "Auth";
pub const LOGIN_REDIRECT_COOKIE: &'static str = "LoginRedirect";
pub const ASSIGN_ADMIN_LINK_TIMEOUT_SECS: u64 = 60 * 60 * 24 * 5;
pub const CHANE_PASSWORD_LINK_TIMEOUT_SECS: u64 = 60 * 60 * 24 * 5;

pub const EXTENDED_APP_NAME: &'static str = "Senior Duke Portal";
pub const APP_NAME: &'static str = "Senior Duke";

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

pub const OA_PAGE: &'static str = "/outstanding";
pub const OA_TITLE: &'static str = "Outstanding Achievements";

pub const STATS_PAGE: &'static str = "/stats";
pub const STATS_TITLE: &'static str = "Stats";

pub const DELETE_USER_PATH: &'static str = "/delete_user";

// Organisation
pub const ORG_ROOT_PATH: &'static str = "/org";

pub fn org_path(org_id: org::OrgKey) -> String {
    ORG_ROOT_PATH.to_owned() + "/" + &org_id.to_string()
}

pub const ADMIN_TITLE: &'static str = "Admin";
pub const ADMIN_PATH: &'static str = "/admin";
pub const DELETE_PATH: &'static str = "/admin/delete";
pub const DOWNLOAD_LOG_PATH: &'static str = "/admin/log";

pub const ACCOUNTS_PATH: &'static str = "/admin/accounts";
pub const ACCOUNTS_TITLE: &'static str = "User Accounts";
pub const ADD_ADMIN_PATH: &'static str = "/admin/add_admin";

pub const ADD_ORG_PATH: &'static str = "/add_org";
pub const DELETE_ORG_PATH: &'static str = "/del_org";
pub const ADD_CREDITS_PATH: &'static str = "/add_credits";
pub const ASSIGN_ADMIN_PATH: &'static str = "/assign_admin";

pub const CLIENTS_PAGE: &'static str = "/clients";
pub const CLIENTS_TITLE: &'static str = "Pupils";
pub const ADD_CLIENT_PATH: &'static str = "/add_client";

pub const SECTION_ROOT: &'static str = "/section";
pub const SELECT_ACTIVITY_PATH: &'static str = "/select_activity";

pub const ASSOCIATES_PAGE: &'static str = "/associates";
pub const ASSOCIATES_TITLE: &'static str = "Teachers";

pub const UNREVIEWED_SECTIONS_PAGE: &'static str = "/unreviewed";
pub const UNREVIEWED_SECTIONS_TITLE: &'static str = "Unreviewed Sections";

pub const ADD_ASSOCIATE_PATH: &'static str = "/add_associate";

// Client
pub const CLIENT_ROOT_PATH: &'static str = "/client";

pub fn client_path(org_id: org::OrgKey, user_id: user::UserKey) -> String {
    org_path(org_id) + CLIENT_ROOT_PATH + "/" + &user_id.to_string()
}

pub const SECTIONS_TITLE: &'static str = "Sections";

pub const NOTIFICATION_INTERVAL_DAYS: u64 = 3;

pub const HELP_PAGE: &'static str = "/help";
pub const HELP_TITLE: &'static str = "Help";

pub fn make_absolute_url(path: &str) -> String {
    "https://".to_owned() + HOST + path
}