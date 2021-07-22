use std::sync::Arc;
use std::str::FromStr;

use actix_web::{
    web,
    http,
    body::Body,
    HttpRequest,
    HttpResponse,
    http::Cookie,
};

use actix_web::HttpMessage;

use serde_json::json;

use crate::data::SharedData;

use crate::page;
use crate::user;
use crate::dir;
use crate::login;
use crate::link;
use crate::util;
use crate::auth::AuthContext;

use actix_web::{post, get};

#[get("/login")]
pub async fn login_get(data: web::Data<Arc<SharedData>>, req: HttpRequest) -> HttpResponse {
    match data.authenticate_context_from_request(&req, false) {
        Ok(ctx) => {
            login_template(ctx, &data, String::new())
        },
        Err(e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                .set_body(Body::from(format!("Login Database Error: {}", e))),
    }
}

pub fn login_template(ctx: Option<AuthContext>, data: &SharedData, msg: String) -> HttpResponse {
    let login_body: String = data.handlebars.render("login/login", &json!({
        "login_err_msg": msg,
        "login_url": dir::LOGIN_POST_PATH,
        "reset_password_url": "/reset_password"
    })).unwrap();

    match page::render_page(ctx, &data, dir::APP_NAME.to_owned() + " | Login", dir::APP_NAME.to_owned(), login_body) {
        Ok(body) => HttpResponse::new(http::StatusCode::OK)
            .set_body(Body::from(body)),

        Err(e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
            .set_body(Body::from(format!("Login Render Error: {}", e))),
    }
}


#[derive(serde::Deserialize)]
pub struct LoginForm {
    username: String,
    password: String,
}

#[post("/login")]
pub async fn login_post(data: web::Data<Arc<SharedData>>, req: HttpRequest, form: web::Form<LoginForm>) -> HttpResponse {
    match data.authenticate_context_from_request(&req, false) {
        Ok(old_ctx) => {
            if !form.username.is_empty() && !form.password.is_empty() {
                match data.login(&form.username, &form.password, std::time::Duration::from_secs(15 * 60)) {
                    Ok(ctx) => {
                        // Add cookie...
                        let auth_cookie = Cookie::new(dir::AUTH_COOKIE, ctx.auth_token.to_string());

                        let mut r = HttpResponse::SeeOther();
                        if let Some(existing) = req.cookie(dir::AUTH_COOKIE) {
                            r.del_cookie(&existing);
                        }
                        r.cookie(auth_cookie);
                        
                        r.header(http::header::LOCATION, ctx.root_page());
                        
                        r.body("")
                    },
                    Err(login::AuthError::IncorrectPassword) => login_template(old_ctx, &data, "Incorrect username and password combination".to_owned()),
                    Err(login::AuthError::NoUser) => login_template(old_ctx, &data, "Incorrect username and password combination".to_owned()),
                    Err(login::AuthError::DbError(e)) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                        .set_body(Body::from(format!("Error: {}", e))),
                }
            } else {
                login_template(old_ctx, &data, "Please provide a username and password!".to_owned())
            }
        },
        Err(e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                .set_body(Body::from(format!("Error: {}", e))),
    }
}

#[get("/logout")]
pub async fn logout_get(data: web::Data<Arc<SharedData>>, req: HttpRequest) -> HttpResponse {
    match data.logout(&req) {
        Ok(_) => {
            let mut r = HttpResponse::SeeOther();
            if let Some(ref cookie) = req.cookie(dir::AUTH_COOKIE) {
                r.del_cookie(cookie);
            }
            
            r.header(http::header::LOCATION, dir::LOGIN_PAGE);
            r.body("")
        },
        Err(e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                .set_body(Body::from(format!("Logout Error: {}", e))),
    }
}


pub fn change_password_page(ctx: Option<AuthContext>, data: &SharedData, token: link::LinkToken, msg: &str) -> HttpResponse {
    match data.link_manager.fetch(&token) {
        Ok(Some(entry)) => {
            if let link::Link::ChangePassword(user_id) = entry.link {
                match data.user_db.fetch(&user_id) {
                    Ok(Some(user)) => {
                        let body: String = data.handlebars.render("login/change_password", &json!({
                            "err_msg": msg,
                            "email": user.email,
                            "change_password_url": dir::CHANGE_PASSWORD_PATH.to_owned() + "/" + &token.to_string(),
                        })).unwrap();
                    
                        match page::render_page(ctx, &data, dir::APP_NAME.to_owned() + " | Change Password", dir::APP_NAME.to_owned(), body) {
                            Ok(body) => HttpResponse::new(http::StatusCode::OK)
                                .set_body(Body::from(body)),
                    
                            Err(e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                                .set_body(Body::from(format!("Login Render Error: {}", e))),
                        }
                    },
                    _ => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                        .set_body(Body::from("No user!")),
                }
                
            } else {
                HttpResponse::new(http::StatusCode::BAD_REQUEST)
                    .set_body(Body::from("bad link"))
            }
           
        }, 
        Ok(None) => HttpResponse::new(http::StatusCode::BAD_REQUEST)
            .set_body(Body::from("bad link")),

        Err(e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
            .set_body(Body::from(format!("Logout Error: {}", e))),
    }
    
}

#[get("/user/change_password/{link_token}")]
pub async fn change_password_get(data: web::Data<Arc<SharedData>>, req: HttpRequest, link_token_str: web::Path<String>) -> HttpResponse {
    match link::LinkToken::from_str(&link_token_str) {
        Ok(token) => {
            match data.authenticate_context_from_request(&req, false) {
                Ok(ctx) => {
                    change_password_page(ctx, &data, token, "")
                },
                Err(e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                        .set_body(Body::from(format!("Error: {}", e))),
            }
        },
        Err(e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
            .set_body(Body::from(format!("Error: {}", e))),
    }
}

#[derive(serde::Deserialize)]
pub struct ChangePasswordForm {
    pub password: String,
    pub confirm: String,
}

#[post("/user/change_password/{link_token}")]
pub async fn change_password_post(data: web::Data<Arc<SharedData>>, req: HttpRequest, link_token_str: web::Path<String>, form: web::Form<ChangePasswordForm>) -> HttpResponse {
    match link::LinkToken::from_str(&link_token_str) {
        Ok(token) => {
            match data.authenticate_context_from_request(&req, false) {
                Ok(ctx) => {
                    if form.password != form.confirm {
                        return change_password_page(ctx, &data, token, "Passwords do not match!");
                    }
                    if !util::is_string_server_valid(&form.password) {
                        return change_password_page(ctx, &data, token, "Invalid password string!");
                    }
                    match data.link_manager.fetch(&token) {
                        Ok(Some(entry)) => {
                            if let link::Link::ChangePassword(user_id) = entry.link {
                                match data.user_db.fetch(&user_id) {
                                    Ok(Some(user)) => {
                                        // Got user
                                        let login_key = user.email;

                                        match data.login_db.change_password(&login_key, &form.password, false) {
                                            Ok(_) => {
                                                // Destroy link
                                                let _ = data.link_manager.db().remove_silent(&token);

                                                let mut r = HttpResponse::SeeOther();
    
                                                r.header(http::header::LOCATION, dir::LOGIN_PAGE);
                                                r.body("")
                                            },
                                            Err(_) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                                                .set_body(Body::from("Failed to modify login db!")),
                                        }
                                    },
                                    _ => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                                        .set_body(Body::from("No user!")),
                                }
                                
                            } else {
                                HttpResponse::new(http::StatusCode::BAD_REQUEST)
                                    .set_body(Body::from("bad link"))
                            }
                        }, 
                        Ok(None) => HttpResponse::new(http::StatusCode::BAD_REQUEST)
                            .set_body(Body::from("bad link")),
                
                        Err(e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                            .set_body(Body::from(format!("Logout Error: {}", e))),
                    }
                },
                Err(e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                        .set_body(Body::from(format!("Error: {}", e))),
            }
        },
        Err(e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
            .set_body(Body::from(format!("Error: {}", e))),
    }
}

pub fn create_account_page(ctx: Option<AuthContext>, data: &SharedData, token: link::LinkToken, msg: &str) -> HttpResponse {
    match data.link_manager.fetch(&token) {
        Ok(Some(entry)) => {
            if let link::Link::CreateUser(user_agent) = entry.link {
                let mut info: String = "Create account:".to_owned();
                if let user::UserAgent::Organisation(org_id) = &user_agent {
                    if let Ok(Some(org)) = data.org_db.fetch(org_id) {
                        info = "Create an organisation administrator account for '".to_owned() + &org.name + "':"
                    }
                }
                let body: String = data.handlebars.render("login/create_account", &json!({
                    "err_msg": msg,
                    "account_info": info,
                    "create_account_url": "/user/create_account/".to_string() + &token.to_string(),
                })).unwrap();
            
                match page::render_page(ctx, &data, dir::APP_NAME.to_owned() + " | Create Account", dir::APP_NAME.to_owned(), body) {
                    Ok(body) => HttpResponse::new(http::StatusCode::OK)
                        .set_body(Body::from(body)),
            
                    Err(e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                        .set_body(Body::from(format!("Create Password Render Error: {}", e))),
                }
            } else {
                HttpResponse::new(http::StatusCode::BAD_REQUEST)
                    .set_body(Body::from("bad link"))
            }
           
        }, 
        Ok(None) => HttpResponse::new(http::StatusCode::BAD_REQUEST)
            .set_body(Body::from("bad link")),

        Err(e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
            .set_body(Body::from(format!("Logout Error: {}", e))),
    }
    
}

#[get("/user/create_account/{link_token}")]
pub async fn create_account_get(data: web::Data<Arc<SharedData>>, req: HttpRequest, link_token_str: web::Path<String>) -> HttpResponse {
    match link::LinkToken::from_str(&link_token_str) {
        Ok(token) => {
            match data.authenticate_context_from_request(&req, false) {
                Ok(ctx) => {
                    create_account_page(ctx, &data, token, "")
                },
                Err(e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                        .set_body(Body::from(format!("Error: {}", e))),
            }
        },
        Err(e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
            .set_body(Body::from(format!("Error: {}", e))),
    }
}


#[derive(serde::Deserialize)]
pub struct CreateAccountForm {
    pub forename: String,
    pub surname: String,
    pub email: String,
    pub password: String,
    pub confirm: String,
}

#[post("/user/create_account/{link_token}")]
pub async fn create_account_post(data: web::Data<Arc<SharedData>>, req: HttpRequest, link_token_str: web::Path<String>, form: web::Form<CreateAccountForm>) -> HttpResponse {
    match link::LinkToken::from_str(&link_token_str) {
        Ok(token) => {
            match data.authenticate_context_from_request(&req, false) {
                Ok(ctx) => {
                    if form.password != form.confirm {
                        return create_account_page(ctx, &data, token, "Passwords do not match!");
                    }
                    if !util::is_string_server_valid(&form.password) {
                        return create_account_page(ctx, &data, token, "Invalid password string!");
                    }
                    if !util::is_string_server_valid(&form.forename) ||
                        !util::is_string_server_valid(&form.surname) ||
                        !util::is_string_server_valid(&form.email) {
                            return create_account_page(ctx, &data, token, "Invalid user details!");
                    }

                    match data.link_manager.fetch(&token) {
                        Ok(Some(entry)) => {
                            if let link::Link::CreateUser(user_agent) = entry.link {

                                let user: user::User = user::User {
                                    forename: form.forename.clone(),
                                    surname: form.surname.clone(),
                                    email: form.email.clone(),
                                    notifications: true,
                                    user_agent,
                                };
                                
                                match data.register_user(&user, &form.password, false) {
                                    Ok(_) => {
                                        // Got user
                                        let _ = data.link_manager.db().remove_silent(&token);
                                        let mut r = HttpResponse::SeeOther();
                                        if let Some(existing) = req.cookie(dir::AUTH_COOKIE) {
                                            r.del_cookie(&existing);
                                        }

                                        r.header(http::header::LOCATION, dir::LOGIN_PAGE);
                                        r.body("")
                                    },
                                    _ => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                                        .set_body(Body::from("No user!")),
                                }
                                
                            } else {
                                HttpResponse::new(http::StatusCode::BAD_REQUEST)
                                    .set_body(Body::from("bad link"))
                            }
                        }, 
                        Ok(None) => HttpResponse::new(http::StatusCode::BAD_REQUEST)
                            .set_body(Body::from("bad link")),
                
                        Err(e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                            .set_body(Body::from(format!("Logout Error: {}", e))),
                    }
                },
                Err(e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                        .set_body(Body::from(format!("Error: {}", e))),
            }
        },
        Err(e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
            .set_body(Body::from(format!("Error: {}", e))),
    }
}

#[get("/reset_password")]
pub fn reset_password_get(data: web::Data<Arc<SharedData>>, req: HttpRequest) -> HttpResponse {
    match data.authenticate_context_from_request(&req, false) {
        Ok(ctx) => {
            let page = data.handlebars.render("login/reset_password", &()).unwrap();

            let body = page::render_page(ctx, &data, dir::APP_NAME.to_owned() + " | Organisations", dir::APP_NAME.to_owned(), page).unwrap();

            HttpResponse::new(http::StatusCode::OK)
                .set_body(Body::from(body))
        },
        Err(e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                .set_body(Body::from(format!("Error: {}", e))),
    }
}

#[derive(serde::Deserialize)]
pub struct ResetPasswordForm {
    pub username: String,
}

#[post("/reset_password")]
pub fn reset_password_post(data: web::Data<Arc<SharedData>>, req: HttpRequest, form: web::Form<ResetPasswordForm>) -> HttpResponse {
    match data.authenticate_context_from_request(&req, false) {
        Ok(ctx) => {
            if let Ok(Some(login_entry)) = data.login_db.db().fetch(&form.username) {
                if let Ok(link_token) = data.link_manager.create_link(link::Link::ChangePassword(login_entry.user_id), std::time::Duration::from_secs(dir::CHANE_PASSWORD_LINK_TIMEOUT_SECS)) {
                    // send email.
                    let link: String = "/user/change_password/".to_string() + &link_token.to_string();
                    let addr: String = form.username.clone();

                    let subtitle: String = "<a href=\"".to_owned() + &link + "\">" + "Click here</a> to change your account password.";

                    if let Err(e) = data.send_email(
                        &addr, 
                        "Senior Duke - Change Your Password", 
                        "Change Password",
                        &subtitle, 
                        ""
                    ) {
                        println!("Failed to send email: {}", e);
                    }
                }
            }

            let page = data.handlebars.render("login/reset_password_sent", &()).unwrap();

            let body = page::render_page(ctx, &data, dir::APP_NAME.to_owned() + " | Organisations", dir::APP_NAME.to_owned(), page).unwrap();

            HttpResponse::new(http::StatusCode::OK)
                .set_body(Body::from(body))
        },
        Err(e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                .set_body(Body::from(format!("Error: {}", e))),
    }
}