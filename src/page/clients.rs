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

use actix_web::get;

#[get("/org/{org}/clients")]
pub async fn clients_get(data: web::Data<Arc<SharedData>>, req: HttpRequest, org: web::Path<String>) -> HttpResponse {
    if let Ok(org_id) = org::OrgKey::from_str(&org) {
        match data.authenticate_context_from_request(&req, true) {
            Ok(Some(ctx)) => {
                if ctx.user.user_agent.can_view_org(&org_id) {
                    match data.org_db.fetch(&org_id) {
                        Ok(Some(org)) => {
                            let mut rows: String = String::new();

                            for client_id in org.clients.iter() {
                                match data.user_db.fetch(client_id) {
                                    Ok(Some(user)) => {
                                        if let user::UserAgent::Client { org_id: client_org_id, .. } = user.user_agent {
                                            if client_org_id == org_id {
                                                rows += &data.handlebars.render("client_row", &json!({
                                                    "client_url": dir::client_path(org_id, *client_id),
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

                            let content = data.handlebars.render("client_list", &json!({
                                "client_rows": rows,
                            })).unwrap();

                            let header: String = page::path_header(&data, &[
                                (dir::ORGS_PAGE.to_owned(), dir::ORGS_TITLE.to_owned()), 
                                (dir::org_path(org_id), org.name.clone()),
                            ]);

                            let nav = page::org_nav(&ctx, &data, org_id, dir::org_path(org_id) + dir::CLIENTS_PAGE);

                            let org_page = data.handlebars.render("org_root", &json!({
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
