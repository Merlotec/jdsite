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
    let login_body: String = data.handlebars.render("login", &json!({
        "login_err_msg": msg,
        "login_url": dir::LOGIN_POST_PATH,
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
        Ok(ctx) => {
            if !form.username.is_empty() && !form.password.is_empty() {
                match data.login(&form.username, &form.password, std::time::Duration::from_secs(15 * 60)) {
                    Ok(ctx) => {
                        // Add cookie...
                        let auth_cookie = Cookie::new(dir::AUTH_COOKIE, ctx.auth_token.to_string());

                        let mut r = HttpResponse::SeeOther();
                        r.cookie(auth_cookie);
                        
                        if let Some(redirect) = req.cookie(dir::LOGIN_REDIRECT_COOKIE) {
                            r.header(http::header::LOCATION, redirect.value());
                        } else {
                            r.header(http::header::LOCATION, ctx.root_page());
                        }
                        
                        r.body("")
                    },
                    Err(login::AuthError::IncorrectPassword) => login_template(ctx, &data, "Incorrect username and password combination".to_owned()),
                    Err(login::AuthError::NoUser) => login_template(ctx, &data, "Incorrect username and password combination".to_owned()),
                    Err(login::AuthError::DbError(e)) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                        .set_body(Body::from(format!("Error: {}", e))),
                }
            } else {
                login_template(ctx, &data, "Please provide a username and password!".to_owned())
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
                        let body: String = data.handlebars.render("change_password", &json!({
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
