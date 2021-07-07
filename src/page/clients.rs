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

use actix_web::get;

#[get("/org/{org}/clients")]
pub async fn clients_get(data: web::Data<Arc<SharedData>>, req: HttpRequest, org: web::Path<String>) -> HttpResponse {
    if let Ok(org_id) = org::OrgKey::from_str(&org) {
        match data.authenticate_context_from_request(&req, true) {
            Ok(Some(ctx)) => {
                if ctx.user.user_agent.can_view_org(&org_id) {
                    match data.org_db.fetch(&org_id) {
                        Ok(Some(org)) => {
                            /*
                            let mut rows: String = String::new();

                            data.org_db.for_each(|org_id, org| {
                                let admin: String = {
                                    if let Some(user_id) = org.admin {
                                        if let Ok(Some(admin_user)) = data.user_db.fetch(&user_id) {
                                            admin_user.name()
                                        } else {
                                            data.handlebars.render("assign_admin", &json!({
                                                "org_id": org_id,
                                                "org_name": &org.name,
                                            })).unwrap()
                                        }
                                    } else {
                                        data.handlebars.render("assign_admin", &json!({
                                            "org_id": org_id,
                                            "org_name": &org.name,
                                        })).unwrap()
                                    }
                                };

                                rows += &data.handlebars.render("org_row", &json!({
                                    "org_url": dir::org_path(org_id),
                                    "admin": admin,
                                    "name": org.name,
                                    "unreviewed_sections": org.unreviewed_sections.len(),
                                    "teachers": org.associates.len(),
                                    "pupils": org.clients.len(),
                                    "credits": org.credits,
                                })).unwrap();
                            });
                            */
                        
                            let nav = page::org_nav(&ctx, &data, org_id, dir::org_path(org_id) + dir::CLIENTS_PAGE);

                            let header: String = page::path_header(&data, &[
                                (dir::ORGS_PAGE.to_owned(), dir::ORGS_TITLE.to_owned()), 
                                (dir::org_path(org_id), org.name.clone()),
                            ]);

                            let org_page = data.handlebars.render("org_root", &json!({
                                "header": header,
                                "org_nav": nav,
                                "body": "",
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
