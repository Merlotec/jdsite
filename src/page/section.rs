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

#[get("/org/{org}/client/{user}/section/{section}")]
pub async fn select_section_get(data: web::Data<Arc<SharedData>>, req: HttpRequest, path: web::Path<(String, String, usize)>) -> HttpResponse {
    if let Ok(org_id) = org::OrgKey::from_str(&path.0.0) {
        if let Ok(user_id) = user::UserKey::from_str(&path.0.1) {
            if path.0.2 < 6 {
                let section = &data.sections[path.0.2];
                match data.authenticate_context_from_request(&req, true) {
                    Ok(Some(ctx)) => {
                        match data.user_db.fetch(&user_id) {
                            Ok(Some(user)) => {
                                if ctx.user.user_agent.can_view_user(&user.user_agent) || ctx.user_id == user_id {
                                    match data.org_db.fetch(&org_id) {
                                        Ok(Some(org)) => {
    
                                            let mut activities: String = String::new();
    
                                            for (i, activity) in section.activities.iter().enumerate() {
                                                let desc: String = {
                                                    match data.handlebars.render(&activity.activity_url, &()) {
                                                        Ok(data) => data,
                                                        Err(e) => format!("Failed to render: {}", e),
                                                    }
                                                };

                                                activities += &data.handlebars.render("sections/activity_option", &json!({
                                                    "index": i,
                                                    "title": &activity.name,
                                                    "subtitle": &activity.subtitle,
                                                    "description": desc,
                                                })).unwrap();
                                            }
    
                                            let body: String = data.handlebars.render("sections/section_select", &json!({
                                                "section_name": &section.name,
                                                "back_url": dir::client_path(org_id, user_id),
                                                "activities": activities,
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
    
                                            let body = page::render_page(Some(ctx), &data, dir::APP_NAME.to_owned() + " | " + "Pupil Dashboard", dir::APP_NAME.to_owned(), root).unwrap();
                            
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
                    .set_body(Body::from("Invalid section index"))
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