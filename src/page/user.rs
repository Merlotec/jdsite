use actix_web::{body::Body, http, web, HttpRequest, HttpResponse};
use std::str::FromStr;
use std::sync::Arc;

use serde_json::json;

use crate::data::SharedData;

use crate::dir;
use crate::page;

use crate::user;

use actix_web::{get, post};

#[get("/user/{user}")]
pub async fn user_get(
    data: web::Data<Arc<SharedData>>,
    req: HttpRequest,
    user_id_str: web::Path<String>,
) -> HttpResponse {
    if let Ok(user_id) = user::UserKey::from_str(&user_id_str) {
        match data.user_db.fetch(&user_id) {
            Ok(Some(user)) => match data.authenticate_context_from_request(&req, true) {
                Ok(Some(ctx)) => {
                    let mut can_view_user: bool = false;

                    if let Some(org_id) = user.user_agent.org_id() {
                        if ctx.user.user_agent.can_view_org(&org_id) {
                            can_view_user = true;
                        }
                    } else {
                        if ctx.user.user_agent.privilege().is_root() {
                            can_view_user = true;
                        } else if ctx.user.user_agent.privilege().magnitude() > user.user_agent.privilege().magnitude() {
                            can_view_user = true;
                        }
                    }

                    let is_active_user = ctx.user_id == user_id;

                    if is_active_user || can_view_user {
                        let mut attrs: String = String::new();

                        attrs += &data
                            .handlebars
                            .render(
                                "user/user_attribute",
                                &json!({
                                    "attribute_name": "Email",
                                    "attribute_value": user.email,
                                }),
                            )
                            .unwrap();

                        attrs += "<br><br>";
                        attrs += &data
                            .handlebars
                            .render(
                                "user/user_attribute",
                                &json!({
                                    "attribute_name": "Account Type",
                                    "attribute_value": user.user_agent.agent_string(),
                                }),
                            )
                            .unwrap();

                        if is_active_user || ctx.user.user_agent.privilege().is_root() {
                            attrs += "<br><br>";
                            attrs += &data.handlebars.render("user/user_attribute_notifications", &json!({
                                    "flag": user.notifications,
                                    "notifications_base_url": "/user/".to_owned() + &user_id.to_string() + "/enable_notifications",
                                })).unwrap();
                        } else {
                            attrs += "<br><br>";
                            attrs += &data
                                .handlebars
                                .render(
                                    "user/user_attribute",
                                    &json!({
                                        "attribute_name": "Email Notifications",
                                        "attribute_value": match user.notifications {
                                            true => "On".to_owned(),
                                            false => "Off".to_owned(),
                                        },
                                    }),
                                )
                                .unwrap();
                        }

                        if let Some(org_id) = user.user_agent.org_id() {
                            if let Ok(Some(org)) = data.org_db.fetch(&org_id) {
                                attrs += "<br><br>";
                                attrs += &data
                                    .handlebars
                                    .render(
                                        "user/user_attribute",
                                        &json!({
                                            "attribute_name": "Organisation",
                                            "attribute_value": org.name,
                                        }),
                                    )
                                    .unwrap();
                            }
                        }

                        if let Ok(Some(entry)) = data.login_db.db().fetch(&user.email) {
                            if entry.default_password {
                                attrs += "<br><br>";
                                attrs += &data
                                    .handlebars
                                    .render(
                                        "user/user_attribute",
                                        &json!({
                                            "attribute_name": "Password (Auto-Generated)",
                                            "attribute_value": entry.password,
                                        }),
                                    )
                                    .unwrap();
                            }
                        }

                        let page_body = data
                            .handlebars
                            .render(
                                "user/user",
                                &json!({
                                    "name": user.name(),
                                    "attributes": attrs,
                                }),
                            )
                            .unwrap();

                        let body = page::render_page(
                            Some(ctx),
                            &data,
                            dir::APP_NAME.to_owned() + " | " + &user.name(),
                            dir::EXTENDED_APP_NAME.to_owned(),
                            page_body,
                        )
                        .unwrap();

                        HttpResponse::new(http::StatusCode::OK).set_body(Body::from(body))
                    } else {
                        page::not_authorized_page(Some(ctx), &data)
                    }
                }
                Ok(None) => page::redirect_to_login(&req),

                Err(e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                    .set_body(Body::from(format!("Error: {}", e))),
            },
            _ => HttpResponse::new(http::StatusCode::BAD_REQUEST)
                .set_body(Body::from("Invalid user!")),
        }
    } else {
        HttpResponse::new(http::StatusCode::BAD_REQUEST).set_body(Body::from("Invalid org_id"))
    }
}

#[get("/user/{user}/enable_notifications/{flag}")]
pub async fn enable_notifications_get(
    data: web::Data<Arc<SharedData>>,
    req: HttpRequest,
    path: web::Path<(String, String)>,
) -> HttpResponse {
    if let Ok(user_id) = user::UserKey::from_str(&path.0 .0) {
        if let Ok(flag) = path.0 .1.parse::<bool>() {
            match data.user_db.fetch(&user_id) {
                Ok(Some(mut user)) => match data.authenticate_context_from_request(&req, true) {
                    Ok(Some(ctx)) => {
                        let mut can_view_org: bool = false;

                        if let Some(org_id) = user.user_agent.org_id() {
                            if ctx.user.user_agent.can_view_org(&org_id) {
                                can_view_org = true;
                            }
                        }

                        if ctx.user_id == user_id || can_view_org {
                            user.notifications = flag;
                            let _ = data.user_db.insert(&user_id, &user);
                            let mut r = HttpResponse::SeeOther();
                            if let Some(referer) = req.headers().get("Referer") {
                                r.header(http::header::LOCATION, referer.clone());
                            }
                            r.body("")
                        } else {
                            page::not_authorized_page(Some(ctx), &data)
                        }
                    }
                    Ok(None) => page::redirect_to_login(&req),

                    Err(e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                        .set_body(Body::from(format!("Error: {}", e))),
                },
                _ => HttpResponse::new(http::StatusCode::BAD_REQUEST)
                    .set_body(Body::from("Invalid user!")),
            }
        } else {
            HttpResponse::new(http::StatusCode::BAD_REQUEST)
                .set_body(Body::from("Invalid flag value"))
        }
    } else {
        HttpResponse::new(http::StatusCode::BAD_REQUEST).set_body(Body::from("Invalid org_id"))
    }
}

#[derive(serde::Deserialize)]
pub struct DeleteUserForm {
    user_id: user::UserKey,
}

#[post("/delete_user")]
pub async fn delete_user_post(
    data: web::Data<Arc<SharedData>>,
    req: HttpRequest,
    form: web::Form<DeleteUserForm>,
) -> HttpResponse {
    match data.authenticate_context_from_request(&req, true) {
        Ok(Some(ctx)) => match data.user_db.fetch(&form.user_id) {
            Ok(Some(target)) => {
                if ctx.user.user_agent.can_delete_user(&target.user_agent) {
                    match data.delete_user(&form.user_id) {
                        Ok(_) => {
                            let mut r = HttpResponse::SeeOther();
                            if let Some(referer) = req.headers().get("Referer") {
                                r.header(http::header::LOCATION, referer.clone());
                            }
                            r.body("")
                        }
                        Err(e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                            .set_body(Body::from(format!("Failed to delete use: {}!", e))),
                    }
                } else {
                    page::not_authorized_page(Some(ctx), &data)
                }
            }
            _ => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                .set_body(Body::from("Failed to get user!")),
        },
        Ok(None) => page::redirect_to_login(&req),

        Err(e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
            .set_body(Body::from(format!("Error: {}", e))),
    }
}
