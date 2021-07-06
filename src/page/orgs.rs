use std::sync::Arc;

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

use actix_web::{get, post};


#[get("/orgs")]
pub async fn org_get(data: web::Data<Arc<SharedData>>, req: HttpRequest) -> HttpResponse {
    match data.authenticate_context_from_request(&req, true) {
        Ok(Some(ctx)) => {
            if ctx.user.user_agent.can_view_orgs() {
                let mut rows: String = String::new();

                data.org_db.for_each(|org_id, org| {
                    let admin: String = {
                        if let Some(user_id) = org.admin {
                            if let Ok(Some(admin_user)) = data.user_db.fetch(&user_id) {
                                admin_user.name()
                            } else {
                                data.handlebars.render("assign_admin", &json!({
                                    "org": org_id,
                                })).unwrap()
                            }
                        } else {
                            data.handlebars.render("assign_admin", &json!({
                                "org": org_id,
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

                let org_page = data.handlebars.render("org_list", &json!({
                    "org_rows": rows,
                    "add_org_url": dir::ADD_ORG_PATH,
                })).unwrap();

                let body = page::render_page(Some(ctx), &data, dir::APP_NAME.to_owned() + " | Organisations", dir::APP_NAME.to_owned(), org_page).unwrap();

                HttpResponse::new(http::StatusCode::OK)
                    .set_body(Body::from(body))
                
            } else {
                HttpResponse::new(http::StatusCode::UNAUTHORIZED)
                    .set_body(Body::from(format!("Error: {}", "not authenticated")))
            }
        },
        Ok(None) => HttpResponse::new(http::StatusCode::UNAUTHORIZED)
            .set_body(Body::from(format!("Error: {}", "not authenticated"))),

        Err(e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
            .set_body(Body::from(format!("Error: {}", e))),
    }
}

#[derive(serde::Deserialize)]
pub struct AddOrgForm {
    name: String,
}

#[post("/add_org")]
pub async fn add_org_post(data: web::Data<Arc<SharedData>>, req: HttpRequest, form: web::Form<AddOrgForm>) -> HttpResponse {
    match data.authenticate_context_from_request(&req, true) {
        Ok(Some(ctx)) => {
            if ctx.user.user_agent.can_view_orgs() {
                
                let org = org::Org::new(form.name.clone());

                match data.org_db.insert(&org::OrgKey::generate(), &org) {
                    Ok(_) => {
                        let mut r = HttpResponse::SeeOther();
                        r.header(http::header::LOCATION, dir::ORGS_PAGE);
                        r.body("")
                    },
                    Err(e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                        .set_body(Body::from(format!("Error: {}", e))),
                }
                
            } else {
                HttpResponse::new(http::StatusCode::UNAUTHORIZED)
                    .set_body(Body::from(format!("Error: {}", "not authenticated")))
            }
        },
        Ok(None) => HttpResponse::new(http::StatusCode::UNAUTHORIZED)
            .set_body(Body::from(format!("Error: {}", "not authenticated"))),

        Err(e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
            .set_body(Body::from(format!("Error: {}", e))),
    }
}
