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

use actix_web::{get, post};

#[get("/user/{user}")]
pub async fn user_get(data: web::Data<Arc<SharedData>>, req: HttpRequest, user_id_str: web::Path<String>) -> HttpResponse {
    if let Ok(user_id) = user::UserKey::from_str(&user_id_str) {
        match data.user_db.fetch(&user_id) {
            Ok(Some(user)) => {
                match data.authenticate_context_from_request(&req, true) {
                    Ok(Some(ctx)) => {
                        let mut can_view_org: bool = false;

                        if let Some(org_id) = user.user_agent.org_id() {
                            if ctx.user.user_agent.can_view_org(&org_id) {
                                can_view_org = true;
                            }
                        }

                        if ctx.user_id == user_id || can_view_org {
                            
                            let mut attrs: String = String::new();

                            attrs += &data.handlebars.render("user_attribute", &json!({
                                "attribute_name": "Email",
                                "attribute_value": user.email,
                            })).unwrap();

                            attrs += "<br><br>";
                            attrs += &data.handlebars.render("user_attribute", &json!({
                                "attribute_name": "Account Type",
                                "attribute_value": user.user_agent.agent_string(),
                            })).unwrap();

                            if let Some(org_id) = user.user_agent.org_id() {
                                if let Ok(Some(org)) = data.org_db.fetch(&org_id) {
                                    attrs += "<br><br>";
                                    attrs += &data.handlebars.render("user_attribute", &json!({
                                        "attribute_name": "Organisation",
                                        "attribute_value": org.name,
                                    })).unwrap();
                                }
                            }

                            if let Ok(Some(entry)) = data.login_db.db().fetch(&user.email) {
                                if entry.default_password {
                                    attrs += "<br><br>";
                                    attrs += &data.handlebars.render("user_attribute", &json!({
                                        "attribute_name": "Password (Auto-Generated)",
                                        "attribute_value": entry.password,
                                    })).unwrap();
                                }
                                
                            }

                            let page_body = data.handlebars.render("user", &json!({
                                "name": user.name(),
                                "attributes": attrs,
                            })).unwrap();

                            let body = page::render_page(Some(ctx), &data, dir::APP_NAME.to_owned() + " | " + &user.name(), dir::APP_NAME.to_owned(), page_body).unwrap();

                            HttpResponse::new(http::StatusCode::OK)
                                .set_body(Body::from(body))
                        } else {
                            page::not_authorized_page(Some(ctx), &data)
                        }
                    },
                    Ok(None) => page::redirect_to_login(&req),
        
                    Err(e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                        .set_body(Body::from(format!("Error: {}", e))),
                        
                } 
            },
            _ => HttpResponse::new(http::StatusCode::BAD_REQUEST)
                .set_body(Body::from("Invalid user!")),
        }
        
    } else {
        HttpResponse::new(http::StatusCode::BAD_REQUEST)
            .set_body(Body::from("Invalid org_id"))
    }
}

#[derive(serde::Deserialize)]
pub struct DeleteUserForm {
    user_id: user::UserKey,
}

#[post("/delete_user")]
pub async fn delete_user_post(data: web::Data<Arc<SharedData>>, req: HttpRequest, form: web::Form<DeleteUserForm>) -> HttpResponse {
    match data.authenticate_context_from_request(&req, true) {
        Ok(Some(ctx)) => {
            match data.user_db.fetch(&form.user_id) {
                Ok(Some(target)) => {
                    if ctx.user.user_agent.can_delete_user(&target.user_agent) {
                        match data.delete_user(&form.user_id) {
                            Ok(_) => {
                                let mut r = HttpResponse::SeeOther();        
                                if let Some(referer) = req.headers().get("Referer") {
                                    r.header(http::header::LOCATION, referer.clone());
                                }
                                r.body("")
                            },
                            Err(e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                            .set_body(Body::from("Failed to delete user!")),
                        }
                        
                    } else {
                        page::not_authorized_page(Some(ctx), &data)
                    }
                },
                _ => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                    .set_body(Body::from("Failed to get user!")),
            }
        },
        Ok(None) => page::redirect_to_login(&req),

        Err(e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
            .set_body(Body::from(format!("Error: {}", e))),
    }
}
