
use crate::*;

use user::{User, UserAgent, UserKey};
use auth::{AuthContext, AuthToken};
use actix_web::HttpMessage;
use std::str::FromStr;
use std::time::Duration;
use handlebars::Handlebars;

pub struct SharedData {
    pub fs_root: String,

    pub login_db: login::LoginDb,
    pub user_db: user::UserDb,
    pub org_db: org::OrgDb,
    pub section_db: section::SectionDb,

    pub auth_manager: auth::AuthManager,

    pub handlebars: Handlebars<'static>,
}

impl SharedData {
    pub fn load_from_disk(fs_root: String) -> sled::Result<Self> {

        let login_db = login::LoginDb::open(fs_root.clone() + "/login.sleddb")?;
        let user_db = user::UserDb::open(fs_root.clone() + "/user.sleddb")?;
        let org_db = org::OrgDb::open(fs_root.clone() + "/org.sleddb")?;
        let section_db = section::SectionDb::open(fs_root.clone() + "/section.sleddb")?;

        let auth_manager = auth::AuthManager::open(fs_root.clone() + "/auth.sleddb")?;

        let mut handlebars = handlebars::Handlebars::new();

        handlebars.register_templates_directory(".html", "./templates".to_string()).unwrap();

        Ok(Self {
            fs_root,

            login_db,
            user_db,
            org_db,
            section_db,

            auth_manager,

            handlebars,
        })
    }

    pub fn login(&self, username: &str, password: &str, timeout: Duration) -> Result<AuthContext, login::AuthError> {
        match self.login_db.authenticate(username, password) {
            Ok(user_id) => {
                match self.user_db.fetch(&user_id) {
                    Ok(Some(user)) => {
                        // Create session.
                        match self.auth_manager.create_session(&user_id, timeout) {
                            Ok(auth_token) => Ok(
                                AuthContext {
                                    auth_token,
                                    user,
                                    user_id,
                                }
                            ),
                            Err(e) => Err(login::AuthError::DbError(e)),
                        }
                    },
                    Ok(None) => Err(login::AuthError::NoUser),
                    Err(e) => Err(login::AuthError::DbError(e)),
                }
            },
            Err(e) => Err(e),
        }
    }

    pub fn create_user(&self, username: &str, password: &str, user: &User) -> Result<(), login::LoginEntryError> {
        let user_id = UserKey::generate();
        match self.user_db.insert(&user_id, user) {
            Ok(_) => {
                let login_entry = login::LoginEntry {
                    user_id,
                    password: password.to_owned(),
                };
        
                match self.login_db.add_entry(username, &login_entry) {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        self.user_db.remove_silent(&user_id);
                        Err(e)
                    },
                }
            },

            Err(e) => Err(login::LoginEntryError::DbError(e)),
        }
    }

    pub fn authenticate_context_from_request(&self, response: &HttpRequest) -> Result<Option<AuthContext>, db::Error> {
        match response.cookie(dir::AUTH_COOKIE)  {
            Some(auth_token) => {
                match AuthToken::from_str(auth_token.value()) {
                    Ok(auth_token) => self.authenticate_context(auth_token),
                    Err(e) => Ok(None),
                }
                
            },
            None => Ok(None),
        }
    }

    pub fn authenticate_context(&self, auth_token: AuthToken) -> Result<Option<AuthContext>, db::Error> {
        match self.auth_manager.check_token(&auth_token) {
            Ok(Some(user_id)) => {
                match self.user_db.fetch(&user_id) {
                    Ok(Some(user)) => {
                        Ok(Some(
                            AuthContext {
                                auth_token,
                                user,
                                user_id,
                            }
                        ))
                    },
                    Ok(None) => Ok(None),
                    Err(e) => Err(e),
                }
            },
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn nav_items_for_context(&self, ctx: Option<AuthContext>) -> Vec<(String, String)> {
        match ctx {
            Some(ctx) => self.nav_items_for_agent(&ctx.user_id, &ctx.user.user_agent),
            None => Vec::new(),
        }
        
    }

    pub fn nav_items_for_agent(&self, user_key: &UserKey, agent: &UserAgent) -> Vec<(String, String)> {
        match agent {
            UserAgent::Owner => vec! [
                (dir::ORGS_PAGE.to_string(), dir::ORGS_TITLE.to_string()),
                (dir::OA_PAGE.to_string(), dir::OA_TITLE.to_string()),
            ],
            UserAgent::Admin => vec! [
                (dir::ORGS_PAGE.to_string(), dir::ORGS_TITLE.to_string()),
                (dir::OA_PAGE.to_string(), dir::OA_TITLE.to_string()),
            ],
            UserAgent::Orginisation(org_key) => vec! [
                (dir::ORG_ROOT_PATH.to_string() + "/" + &org_key.to_string(), dir::CLIENTS_TITLE.to_string()),
                //(dir::OA_PAGE.to_string(), dir::OA_TITLE.to_string()),
            ],
            UserAgent::Associate(org_key) => vec! [
                (dir::ORG_ROOT_PATH.to_string() + "/" + &org_key.to_string(), dir::CLIENTS_TITLE.to_string()),
            ],
            UserAgent::Client(org_key) => vec! [
                (dir::ORG_ROOT_PATH.to_string() + "/" + &org_key.to_string() + dir::CLIENT_ROOT_PATH + "/" + &user_key.to_string(), dir::SECTIONS_TITLE.to_string()),
            ],
        }
    }
}
