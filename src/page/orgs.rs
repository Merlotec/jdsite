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
use crate::util;

use actix_web::{get, post};

#[get("/org/{org}")]
pub async fn org_get(org: web::Path<String>) -> HttpResponse {
    if let Ok(org_id) = org::OrgKey::from_str(&org) {
        let mut r = HttpResponse::SeeOther();
        r.header(http::header::LOCATION, dir::org_path(org_id) + dir::CLIENTS_PAGE);
        r.body("")
    } else {
        HttpResponse::new(http::StatusCode::BAD_REQUEST)
            .set_body(Body::from("Invalid org_id"))
    }
}

#[get("/orgs")]
pub async fn orgs_get(data: web::Data<Arc<SharedData>>, req: HttpRequest) -> HttpResponse {
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
                        "org_id": org_id,
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
                    "assign_admin_url": dir::ASSIGN_ADMIN_PATH,
                    "delete_org_url": dir::DELETE_ORG_PATH,
                    "add_credits_url": dir::ADD_CREDITS_PATH,
                })).unwrap();

                let body = page::render_page(Some(ctx), &data, dir::APP_NAME.to_owned() + " | Organisations", dir::APP_NAME.to_owned(), org_page).unwrap();

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

                if !util::is_string_server_valid(&org.name) {
                    return HttpResponse::new(http::StatusCode::BAD_REQUEST)
                    .set_body(Body::from("Dissalowed characters in org name!"));
                }

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
                page::not_authorized_page(Some(ctx), &data)
            }
        },
        Ok(None) => page::redirect_to_login(&req),

        Err(e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
            .set_body(Body::from(format!("Error: {}", e))),
    }
}

#[derive(serde::Deserialize)]
pub struct DeleteOrgForm {
    org_id: org::OrgKey,
}

#[post("/del_org")]
pub async fn delete_org_post(data: web::Data<Arc<SharedData>>, req: HttpRequest, form: web::Form<DeleteOrgForm>) -> HttpResponse {
    match data.authenticate_context_from_request(&req, true) {
        Ok(Some(ctx)) => {
            if ctx.user.user_agent.can_delete_orgs() {
                match data.org_db.remove_silent(&form.org_id) {
                    Ok(_) => {
                        let mut r = HttpResponse::SeeOther();
                        r.header(http::header::LOCATION, dir::ORGS_PAGE);
                        r.body("")
                    },
                    Err(e) =>  HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                        .set_body(Body::from(format!("Error: {}", e))),
                } 
            } else {
                page::not_authorized_page(Some(ctx), &data)
            }
        },
        Ok(None) => page::redirect_to_login(&req),

        Err(e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
            .set_body(Body::from(format!("Error: {}", e))),
    }
}


#[derive(serde::Deserialize)]
pub struct AssignAdminForm {
    email: String,
    org_id: org::OrgKey,
}

#[post("/assign_admin")]
pub async fn assign_admin_post(data: web::Data<Arc<SharedData>>, req: HttpRequest, form: web::Form<AssignAdminForm>) -> HttpResponse {
    match data.authenticate_context_from_request(&req, true) {
        Ok(Some(ctx)) => {
            if ctx.user.user_agent.can_view_orgs() {
                let mut r = HttpResponse::SeeOther();
                        r.header(http::header::LOCATION, dir::ORGS_PAGE);
                        r.body("")
            } else {
                page::not_authorized_page(Some(ctx), &data)
            }
        },
        Ok(None) => page::redirect_to_login(&req),

        Err(e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
            .set_body(Body::from(format!("Error: {}", e))),
    }
}

#[derive(serde::Deserialize)]
pub struct AddCreditsForm {
    credits_count: u32,
    org_id: org::OrgKey,
}

#[post("/add_credits")]
pub async fn add_credits_post(data: web::Data<Arc<SharedData>>, req: HttpRequest, form: web::Form<AddCreditsForm>) -> HttpResponse {
    match data.authenticate_context_from_request(&req, true) {
        Ok(Some(ctx)) => {
            if ctx.user.user_agent.can_view_orgs() {
                match data.org_db.fetch(&form.org_id) {
                    Ok(Some(mut org)) => {
                        org.credits += form.credits_count;

                        match data.org_db.insert(&form.org_id, &org) {
                            Ok(_) => {
                                let mut r = HttpResponse::SeeOther();
                                r.header(http::header::LOCATION, dir::ORGS_PAGE);
                                r.body("")
                            },
                            _ => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                                .set_body(Body::from("Could not reinsert org!")),
                        }
                    },
                    _ => HttpResponse::new(http::StatusCode::BAD_REQUEST)
                        .set_body(Body::from("Invalid org id!")),
                }

                
            } else {
                page::not_authorized_page(Some(ctx), &data)
            }
        },
        Ok(None) => page::redirect_to_login(&req),

        Err(e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
            .set_body(Body::from(format!("Error: {}", e))),
    }
}