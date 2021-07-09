use std::sync::Arc;
use std::str::FromStr;
use actix_web::{
    web,
    http,
    body::Body,
    HttpRequest,
    HttpResponse,
};

use serde_json::json;

use crate::data::SharedData;

use crate::page;
use crate::dir;
use crate::org;
use crate::user;
use crate::util;
use crate::login;

use actix_web::{get, post};

#[get("/org/{org}/clients")]
pub async fn clients_get(data: web::Data<Arc<SharedData>>, req: HttpRequest, org_path_str: web::Path<String>) -> HttpResponse {
    if let Ok(org_id) = org::OrgKey::from_str(&org_path_str) {
        match data.authenticate_context_from_request(&req, true) {
            Ok(Some(ctx)) => {
                if ctx.user.user_agent.can_view_org(&org_id) {
                    match data.org_db.fetch(&org_id) {
                        Ok(Some(org)) => {
                            let mut rows: String = String::new();

                            for user_id in org.clients.iter() {
                                match data.user_db.fetch(user_id) {
                                    Ok(Some(user)) => {
                                        if let user::UserAgent::Client { org_id: client_org_id, .. } = user.user_agent {
                                            if client_org_id == org_id {
                                                rows += &data.handlebars.render("client/client_row", &json!({
                                                    "client_url": dir::client_path(org_id, *user_id),
                                                    "user_url": dir::user_path(*user_id),
                                                    "user_id": user_id,
                                                    "name": user.name(),
                                                    "email": user.email,
                                                    "completed_sections": "0/6",
                                                    "unreviewed_sections": "0",
                                                })).unwrap();
                                            }
                                        }
                                    },
                                    _ => {},
                                }
                            }

                            let add_client_button: String = {
                                if org.credits > 0 {
                                    data.handlebars.render("client/add_client_button", &json!({
                                        "add_client_url": dir::org_path(org_id) + dir::ADD_CLIENT_PATH,
                                    })).unwrap()
                                } else {
                                    String::new()
                                }
                            };

                            let content = data.handlebars.render("client/client_list", &json!({
                                "credits": org.credits,
                                "add_client_button": add_client_button,
                                "client_rows": rows,
                                "delete_user_url": dir::DELETE_USER_PATH.to_owned(),
                            })).unwrap();

                            let header: String = page::path_header(&data, &[
                                (dir::ORGS_PAGE.to_owned(), dir::ORGS_TITLE.to_owned()), 
                                (dir::org_path(org_id), org.name.clone()),
                            ]);

                            let nav = page::org_nav(&ctx, &data, org_id, dir::org_path(org_id) + dir::CLIENTS_PAGE);

                            let org_page = data.handlebars.render("org/org_root", &json!({
                                "header": header,
                                "org_nav": nav,
                                "body": content,
                            })).unwrap();

                            let body = page::render_page(Some(ctx), &data, dir::APP_NAME.to_owned() + " | " + &org.name, dir::APP_NAME.to_owned(), org_page).unwrap();

                            HttpResponse::new(http::StatusCode::OK)
                                .set_body(Body::from(body))
                                
                        },
                        _ => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                        .set_body(Body::from("Failed to fetch org!")),
                    }
                    
                    
                } else {
                    page::not_authorized_page(Some(ctx), &data)
                }
            },
            Ok(None) => page::redirect_to_login(&req),

            Err(e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                .set_body(Body::from(format!("Error: {}", e))),
                
        } 
    } else {
        HttpResponse::new(http::StatusCode::BAD_REQUEST)
            .set_body(Body::from("Invalid org_id"))
    }
}

pub fn add_client_page(data: web::Data<Arc<SharedData>>, req: HttpRequest, org_path_str: web::Path<String>, err_msg: &str) -> HttpResponse {
    if let Ok(org_id) = org::OrgKey::from_str(&org_path_str) {
        match data.authenticate_context_from_request(&req, true) {
            Ok(Some(ctx)) => {
                if ctx.user.user_agent.can_view_org(&org_id) {
                    match data.org_db.fetch(&org_id) {
                        Ok(Some(org)) => {
    
                            let content = data.handlebars.render("client/add_client", &json!({
                                "back_url": dir::org_path(org_id) + dir::CLIENTS_PAGE,
                                "add_client_url": dir::org_path(org_id) + dir::ADD_CLIENT_PATH,
                                "err_msg": err_msg,
                            })).unwrap();

                            let header: String = page::path_header(&data, &[
                                (dir::ORGS_PAGE.to_owned(), dir::ORGS_TITLE.to_owned()), 
                                (dir::org_path(org_id), org.name.clone()),
                            ]);

                            let nav = page::org_nav(&ctx, &data, org_id, dir::org_path(org_id) + dir::CLIENTS_PAGE);

                            let org_page = data.handlebars.render("org/org_root", &json!({
                                "header": header,
                                "org_nav": nav,
                                "body": content,
                            })).unwrap();

                            let body = page::render_page(Some(ctx), &data, dir::APP_NAME.to_owned() + " | " + &org.name, dir::APP_NAME.to_owned(), org_page).unwrap();

                            HttpResponse::new(http::StatusCode::OK)
                                .set_body(Body::from(body))
                                
                        },
                        _ => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                        .set_body(Body::from("Failed to fetch org!")),
                    }
                    
                    
                } else {
                    page::not_authorized_page(Some(ctx), &data)
                }
            },
            Ok(None) => page::redirect_to_login(&req),

            Err(e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                .set_body(Body::from(format!("Error: {}", e))),
                
        } 
    } else {
        HttpResponse::new(http::StatusCode::BAD_REQUEST)
            .set_body(Body::from("Invalid org_id"))
    }
}


#[get("/org/{org}/add_client")]
pub async fn add_client_get(data: web::Data<Arc<SharedData>>, req: HttpRequest, org: web::Path<String>) -> HttpResponse {
   add_client_page(data, req, org, "")
}

#[derive(serde::Deserialize)]
pub struct AddClientForm {
    forename: String,
    surname: String,
    email: String,
    class: String,
}

#[post("/org/{org}/add_client")]
pub async fn add_client_post(data: web::Data<Arc<SharedData>>, req: HttpRequest, form: web::Form<AddClientForm>, org_path_str: web::Path<String>) -> HttpResponse {
    if let Ok(org_id) = org::OrgKey::from_str(&org_path_str) {
        match data.authenticate_context_from_request(&req, true) {
            Ok(Some(ctx)) => {
                if ctx.user.user_agent.can_view_org(&org_id) {
                    match data.org_db.fetch(&org_id) {
                        Ok(Some(org)) => {
                            if org.credits > 0 {
                                if util::is_string_server_valid(&form.forename) && 
                                util::is_string_server_valid(&form.surname) &&
                                util::is_string_server_valid(&form.email) &&
                                util::is_string_server_valid(&form.class) {

                                    let user: user::User = user::User {
                                        email: form.email.clone(),
                                        forename: form.forename.clone(),
                                        surname: form.surname.clone(),
                                        user_agent: user::UserAgent::Client {
                                            org_id,
                                            class: form.class.clone(),
                                            sections: [None; 6],
                                        }
                                    };

                                    let password: String = util::gen_password(8);

                                    match data.register_user(&user, &password, true)  {
                                        Ok(_) => {
                                            let mut attrs: String = String::new();

                                            attrs += &data.handlebars.render("user/user_attribute", &json!({
                                                "attribute_name": "Username",
                                                "attribute_value": user.email,
                                            })).unwrap();
                
                                            attrs += "<br><br>";
                                            attrs += &data.handlebars.render("user/user_attribute", &json!({
                                                "attribute_name": "Password",
                                                "attribute_value": password,
                                            })).unwrap();

                                            let content = data.handlebars.render("client/client_added", &json!({
                                                "back_url": dir::org_path(org_id) + dir::CLIENTS_PAGE,
                                                "add_client_url": dir::org_path(org_id) + dir::ADD_CLIENT_PATH,
                                                "attributes": attrs,
                                            })).unwrap();
                
                                            let header: String = page::path_header(&data, &[
                                                (dir::ORGS_PAGE.to_owned(), dir::ORGS_TITLE.to_owned()), 
                                                (dir::org_path(org_id), org.name.clone()),
                                            ]);
                
                                            let nav = page::org_nav(&ctx, &data, org_id, dir::org_path(org_id) + dir::CLIENTS_PAGE);
                
                                            let org_page = data.handlebars.render("org/org_root", &json!({
                                                "header": header,
                                                "org_nav": nav,
                                                "body": content,
                                            })).unwrap();
                
                                            let body = page::render_page(Some(ctx), &data, dir::APP_NAME.to_owned() + " | " + "Pupil Account Created", dir::APP_NAME.to_owned(), org_page).unwrap();
                
                                            HttpResponse::new(http::StatusCode::OK)
                                                .set_body(Body::from(body))

                                        },
                                        Err(login::LoginEntryError::UsernameExists) =>  add_client_page(data, req, org_path_str, "This email is associated with another account!"),
                                        Err(e) =>  add_client_page(data, req, org_path_str, "Something went wrong: ensure that the email is unique!"),
                                    }   
                                } else {
                                    add_client_page(data, req, org_path_str, "Invalid pupil details provided!")
                                }
                            } else {
                                add_client_page(data, req, org_path_str, "No more pupil credits remaining! Please contact support to purchase more.")
                            }
                        },
                        _ => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                        .set_body(Body::from("Failed to fetch org!")),
                    }
                    
                    
                } else {
                    page::not_authorized_page(Some(ctx), &data)
                }
            },
            Ok(None) => page::redirect_to_login(&req),

            Err(e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                .set_body(Body::from(format!("Error: {}", e))),
                
        } 
    } else {
        HttpResponse::new(http::StatusCode::BAD_REQUEST)
            .set_body(Body::from("Invalid org_id"))
    }
}

/*
#[get("/org/{org}/client/{user}")]
pub async fn client_dashboard_get(data: web::Data<Arc<SharedData>>, req: HttpRequest, path: web::Path<(String, String)>) -> HttpResponse {
    if let Ok(org_id) = org::OrgKey::from_str(&path.0.0) {
        if let Ok(user_id) = user::UserKey::from_str(&path.0.1) {
            match data.authenticate_context_from_request(&req, true) {
                Ok(Some(ctx)) => {
                    match data.user_db.fetch(&user_id) {
                        Ok(Some(user)) => {
                            if ctx.user.user_agent.can_view_user(&user.user_agent) || ctx.user_id == user_id {
                                match data.org_db.fetch(&org_id) {
                                    Ok(Some(org)) => {

                                        

                                        let body: String = data.handlebars.render("client/client_dashboard", &json!({
                                            "sections": sections,
                                        })).unwrap();

                                        let header: String = page::path_header(&data, &[
                                            (dir::ORGS_PAGE.to_owned(), dir::ORGS_TITLE.to_owned()), 
                                            (dir::org_path(org_id), org.name.clone()),
                                            (dir::client_path(org_id, user_id), user.name())
                                        ]);

                                        let root: String = data.handlebars.render("client/client_root", &json!({
                                            "header": header,
                                            "body": body,
                                        })).unwrap();

                                        let body = page::render_page(Some(ctx), &data, dir::APP_NAME.to_owned() + " | " + "Pupil Account Created", dir::APP_NAME.to_owned(), root).unwrap();
                        
                                        HttpResponse::new(http::StatusCode::OK)
                                            .set_body(Body::from(body))
                                    },
                                    _ => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                                        .set_body(Body::from("Failed to fetch org!")),
                                }
                            } else {
                                page::not_authorized_page(Some(ctx), &data)
                            }
                        },
                        Ok(None) => HttpResponse::new(http::StatusCode::BAD_REQUEST)
                            .set_body(Body::from("Could not find user!")),
                        Err(e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                            .set_body(Body::from(format!("Error: {}", e))),
                    }
                },
                Ok(None) => page::redirect_to_login(&req),

                Err(e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                    .set_body(Body::from(format!("Error: {}", e))),
                    
            } 
        } else {
            HttpResponse::new(http::StatusCode::BAD_REQUEST)
                .set_body(Body::from("Invalid user_id"))
        }
    } else {
        HttpResponse::new(http::StatusCode::BAD_REQUEST)
            .set_body(Body::from("Invalid org_id"))
    }
}
*/