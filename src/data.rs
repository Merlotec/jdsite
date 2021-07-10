
use crate::*;

use user::{User, UserAgent, UserKey};
use auth::{AuthContext, AuthToken};
use actix_web::HttpMessage;
use std::str::FromStr;
use std::time::Duration;
use section::Section;
use handlebars::{
    Handlebars,
    Helper,
    RenderContext,
    Context,
    Output,
    HelperResult,
    RenderError,
};


pub struct SharedData {
    pub fs_root: String,

    pub login_db: login::LoginDb,
    pub user_db: user::UserDb,
    pub org_db: org::OrgDb,
    pub section_db: section::SectionDb,

    pub sections: [section::SectionInfo; 6],

    pub link_manager: link::LinkManager,

    pub auth_manager: auth::AuthManager,

    pub handlebars: Handlebars<'static>,
}

impl SharedData {
    pub fn load_from_disk(fs_root: String) -> sled::Result<Self> {

        let login_db = login::LoginDb::open(fs_root.clone() + "/login.sleddb")?;
        let user_db = user::UserDb::open(fs_root.clone() + "/user.sleddb")?;
        let org_db = org::OrgDb::open(fs_root.clone() + "/org.sleddb")?;
        let section_db = section::SectionDb::open(fs_root.clone() + "/section.sleddb")?;

        let sections: [section::SectionInfo; 6] = section::SectionInfo::sections_list();

        let auth_manager = auth::AuthManager::open(fs_root.clone() + "/auth.sleddb")?;
        let link_manager = link::LinkManager::open(fs_root.clone() + "/link.sleddb")?;

        let mut handlebars = handlebars::Handlebars::new();

        handlebars.register_templates_directory(".html", "./templates".to_string()).unwrap();

        Ok(Self {
            fs_root,

            login_db,
            user_db,
            org_db,
            section_db,

            sections,

            link_manager,

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

    pub fn logout(&self, req: &HttpRequest) -> sled::Result<()> {
        match req.cookie(dir::AUTH_COOKIE)  {
            Some(auth_token) => {
                match AuthToken::from_str(auth_token.value()) {
                    Ok(auth_token) => self.auth_manager.destroy_session(&auth_token),
                    Err(e) => Ok(()),
                }
                
            },
            None => Ok(()),
        }
    }

    pub fn register_user(&self, user: &User, password: &str, default_password: bool) -> Result<UserKey, login::LoginEntryError> {
        match self.login_db.db().contains_key(&user.email) {
            Ok(exists) => {
                if !exists {
                    let user_id = UserKey::generate();
                    match self.user_db.insert(&user_id, user) {
                        Ok(_) => {

                            let login_entry = login::LoginEntry {
                                user_id,
                                password: password.to_owned(),
                                default_password,
                            };

                            match self.login_db.add_entry(&user.email, &login_entry) {
                                Ok(_) => {
                                    match user.user_agent {
                                        UserAgent::Client { org_id, .. } => {
                                            if let Ok(Some(mut org)) = self.org_db.fetch(&org_id) {
                                                if !org.clients.contains(&user_id) && org.credits > 0 {
                                                    org.clients.push(user_id);
                                                    org.credits -= 1;
                                                    if let Err(e) = self.org_db.insert(&org_id, &org) {
                                                        println!("Failed to update org db for new client! {}", e);
                                                    }
                                                }
                                            }
                                        },
                                        UserAgent::Associate(org_id) => {
                                            if let Ok(Some(mut org)) = self.org_db.fetch(&org_id) {
                                                if !org.associates.contains(&user_id) {
                                                    org.associates.push(user_id);
                                                    if let Err(e) = self.org_db.insert(&org_id, &org) {
                                                        println!("Failed to update org db for new associate! {}", e);
                                                    }
                                                }
                                            }
                                        },
                                        _ => {},
                                    }
                                    Ok(user_id)
                                },
                                Err(e) => {
                                    let _ = self.user_db.remove_silent(&user_id);
                                    Err(e)
                                },
                            }
                        },
            
                        Err(e) => Err(login::LoginEntryError::DbError(e)),
                    }
                } else {
                    Err(login::LoginEntryError::UsernameExists)
                }
            },
            Err(e) => Err(login::LoginEntryError::DbError(db::Error::DbError(e))),
        }

        
    }

    pub fn delete_user(&self, user_id: &UserKey) -> Result<(), db::Error> {
        match self.user_db.remove(user_id) {
            Ok(Some(user)) => {
                if let Err(e) = self.login_db.db().remove_silent(&user.email) {
                    println!("Failed to delete login entry! {}", e);
                }
                match user.user_agent {
                    UserAgent::Client { org_id, .. } => {
                        if let Ok(Some(mut org)) = self.org_db.fetch(&org_id) {
                            org.clients.retain(|x| x != user_id);
                            org.credits += 1;
                            if let Err(e) = self.org_db.insert(&org_id, &org) {
                                println!("Failed to update org db for new client! {}", e);
                            }
                        }
                    },
                    UserAgent::Associate(org_id) => {
                        if let Ok(Some(mut org)) = self.org_db.fetch(&org_id) {
                            org.associates.retain(|x| x != user_id);
                            if let Err(e) = self.org_db.insert(&org_id, &org) {
                                println!("Failed to update org db for new associate! {}", e);
                            }
                        }
                    },
                    _ => {},
                }
                Ok(())
            },
            Ok(None) => Ok(()),
            Err(e) => Err(e),

        }
    }

    pub fn authenticate_context_from_request(&self, req: &HttpRequest, push_expiry: bool) -> Result<Option<AuthContext>, db::Error> {
        match req.cookie(dir::AUTH_COOKIE)  {
            Some(auth_token) => {
                match AuthToken::from_str(auth_token.value()) {
                    Ok(auth_token) => self.authenticate_context(auth_token, push_expiry),
                    Err(_) => Ok(None),
                }
                
            },
            None => Ok(None),
        }
    }

    pub fn authenticate_context(&self, auth_token: AuthToken, push_expiry: bool) -> Result<Option<AuthContext>, db::Error> {
        match self.auth_manager.check_token(&auth_token, push_expiry) {
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

    pub fn add_section(&self, section: &section::Section) -> Result<section::SectionKey, db::Error> {
        let section_id = section::SectionKey::generate();
        match self.section_db.insert(&section_id, section) {
            Ok(_) => Ok(section_id),
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
            UserAgent::Orginisation(org_id) => vec! [
                (dir::ORG_ROOT_PATH.to_string() + "/" + &org_id.to_string(), dir::CLIENTS_TITLE.to_string()),
                //(dir::OA_PAGE.to_string(), dir::OA_TITLE.to_string()),
            ],
            UserAgent::Associate(org_id) => vec! [
                (dir::ORG_ROOT_PATH.to_string() + "/" + &org_id.to_string(), dir::CLIENTS_TITLE.to_string()),
            ],
            UserAgent::Client { org_id, .. } => vec! [
                (dir::ORG_ROOT_PATH.to_string() + "/" + &org_id.to_string() + dir::CLIENT_ROOT_PATH + "/" + &user_key.to_string(), dir::SECTIONS_TITLE.to_string()),
            ],
        }
    }

    pub fn section_path(&self, section_id: &section::SectionKey) -> String {
        format!("{}/sections/{}/", &self.fs_root, section_id.to_string())
    }

    pub fn path_for_asset(&self, section_id: &section::SectionKey, filename: &str) -> String {
        format!("{}/sections/{}/{}", &self.fs_root, section_id.to_string(), filename)
    }
}
