
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

#[get("/org/{org}/associates")]
pub async fn associates_get(data: web::Data<Arc<SharedData>>, req: HttpRequest, org_path_str: web::Path<String>) -> HttpResponse {
    if let Ok(org_id) = org::OrgKey::from_str(&org_path_str) {
        match data.authenticate_context_from_request(&req, true) {
            Ok(Some(ctx)) => {
                if ctx.user.user_agent.can_view_org(&org_id) {
                    match data.org_db.fetch(&org_id) {
                        Ok(Some(org)) => {
                            let mut rows: String = String::new();

                            for user_id in org.associates.iter() {
                                match data.user_db.fetch(user_id) {
                                    Ok(Some(user)) => {
                                        if let user::UserAgent::Associate(associate_org_id) = user.user_agent {
                                            // confirm they're of the correct org
                                            let delete_user_hidden: String = {
                                                if !ctx.user.user_agent.can_delete_user(&user.user_agent) {
                                                    "hidden=\"true\"".to_owned()
                                                } else {
                                                    String::new()
                                                }
                                            };
                                            
                                            if associate_org_id == org_id {
                                                rows += &data.handlebars.render("associate/associate_row", &json!({
                                                    "user_url": dir::user_path(*user_id),
                                                    "name": user.name(),
                                                    "email": user.email,
                                                    "user_id": user_id,
                                                    "delete_user_hidden": delete_user_hidden,
                                                })).unwrap();
                                            }
                                        }
                                    },
                                    _ => {},
                                }
                            }

                            let hide_add_associate: String = {
                                if !ctx.user.user_agent.can_add_associate(&org_id) {
                                    "hidden = \"true\"".to_owned()
                                } else {
                                    String::new()
                                }
                            };

                            let content = data.handlebars.render("associate/associates_list", &json!({
                                "hide_add_associate": hide_add_associate,
                                "add_associate_url": dir::org_path(org_id) + dir::ADD_ASSOCIATE_PATH,
                                "delete_user_url": dir::DELETE_USER_PATH.to_owned(),
                                "associate_rows": rows,
                            })).unwrap();

                            let header: String = page::path_header(&data, &[
                                (dir::ORGS_PAGE.to_owned(), dir::ORGS_TITLE.to_owned()), 
                                (dir::org_path(org_id), org.name.clone()),
                            ]);

                            let nav = page::org_nav(&ctx, &data, org_id, dir::org_path(org_id) + dir::ASSOCIATES_PAGE);

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

pub fn add_associate_page(data: web::Data<Arc<SharedData>>, req: HttpRequest, org_path_str: web::Path<String>, err_msg: &str) -> HttpResponse {
    if let Ok(org_id) = org::OrgKey::from_str(&org_path_str) {
        match data.authenticate_context_from_request(&req, true) {
            Ok(Some(ctx)) => {
                if ctx.user.user_agent.can_add_associate(&org_id) {
                    match data.org_db.fetch(&org_id) {
                        Ok(Some(org)) => {
    
                            let content = data.handlebars.render("associate/add_associate", &json!({
                                "back_url": dir::org_path(org_id) + dir::ASSOCIATES_PAGE,
                                "add_associate_url": dir::org_path(org_id) + dir::ADD_ASSOCIATE_PATH,
                                "err_msg": err_msg,
                            })).unwrap();

                            let header: String = page::path_header(&data, &[
                                (dir::ORGS_PAGE.to_owned(), dir::ORGS_TITLE.to_owned()), 
                                (dir::org_path(org_id), org.name.clone()),
                            ]);

                            let nav = page::org_nav(&ctx, &data, org_id, dir::org_path(org_id) + dir::ASSOCIATES_PAGE);

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


#[get("/org/{org}/add_associate")]
pub async fn add_associate_get(data: web::Data<Arc<SharedData>>, req: HttpRequest, org: web::Path<String>) -> HttpResponse {
   add_associate_page(data, req, org, "")
}

#[derive(serde::Deserialize)]
pub struct AddClientForm {
    forename: String,
    surname: String,
    email: String,
}

#[post("/org/{org}/add_associate")]
pub async fn add_associate_post(data: web::Data<Arc<SharedData>>, req: HttpRequest, form: web::Form<AddClientForm>, org_path_str: web::Path<String>) -> HttpResponse {
    if let Ok(org_id) = org::OrgKey::from_str(&org_path_str) {
        match data.authenticate_context_from_request(&req, true) {
            Ok(Some(ctx)) => {
                if ctx.user.user_agent.can_add_associate(&org_id) {
                    match data.org_db.fetch(&org_id) {
                        Ok(Some(org)) => {
                            
                            if util::is_string_server_valid(&form.forename) && 
                            util::is_string_server_valid(&form.surname) &&
                            util::is_string_server_valid(&form.email) {

                                let user: user::User = user::User {
                                    email: form.email.clone(),
                                    forename: form.forename.clone(),
                                    surname: form.surname.clone(),
                                    user_agent: user::UserAgent::Associate(org_id),
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

                                        let content = data.handlebars.render("associate/associate_added", &json!({
                                            "back_url": dir::org_path(org_id) + dir::ASSOCIATES_PAGE,
                                            "add_associate_url": dir::org_path(org_id) + dir::ADD_ASSOCIATE_PATH,
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
            
                                        let body = page::render_page(Some(ctx), &data, dir::APP_NAME.to_owned() + " | " + "Associate Account Created", dir::APP_NAME.to_owned(), org_page).unwrap();
            
                                        HttpResponse::new(http::StatusCode::OK)
                                            .set_body(Body::from(body))

                                    },
                                    Err(login::LoginEntryError::UsernameExists) =>  add_associate_page(data, req, org_path_str, "This email is associated with another account!"),
                                    Err(e) =>  add_associate_page(data, req, org_path_str, "Something went wrong: ensure that the email is unique!"),
                                }   
                            } else {
                                add_associate_page(data, req, org_path_str, "Invalid teacher details provided!")
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