use std::str::FromStr;
use std::sync::Arc;

use actix_web::{body::Body, http, web, HttpRequest, HttpResponse};

use serde_json::json;

use crate::data::SharedData;

use crate::dir;
use crate::org;
use crate::page;

use crate::user::Privilege;

use actix_web::get;

#[get("/org/{org}/unreviewed")]
pub async fn unreviewed_get(
    data: web::Data<Arc<SharedData>>,
    req: HttpRequest,
    org_path_str: web::Path<String>,
) -> HttpResponse {
    if let Ok(org_id) = org::OrgKey::from_str(&org_path_str) {
        match data.authenticate_context_from_request(&req, true) {
            Ok(Some(ctx)) => {
                if ctx.user.user_agent.can_view_org(&org_id) {
                    match data.org_db.fetch(&org_id) {
                        Ok(Some(org)) => {
                            let mut rows: String = String::new();

                            for section_id in org.unreviewed_sections.iter() {
                                if let Ok(Some(section)) = data.section_db.fetch(section_id) {
                                    if let Ok(Some(user)) = data.user_db.fetch(&section.user_id) {
                                        if let Some(award) = data.awards.get(section.award_index) {
                                            let date_str: String = {
                                                if let Some(system_time) = section.state.time() {
                                                    let datetime: chrono::DateTime<
                                                        chrono::offset::Local,
                                                    > = system_time.into();
                                                    datetime.format("%d %B %Y at %H:%M").to_string()
                                                } else {
                                                    "Error: No date!".to_owned()
                                                }
                                            };

                                            rows += &data.handlebars.render("sections/section_row", &json!({
                                                "client_url": dir::client_path(org_id, section.user_id),
                                                "user_url": dir::user_path(section.user_id),
                                                "section_url": "/section/".to_owned() + &section_id.to_string(),
                                                "name": user.name(),
                                                "email": user.email,
                                                "award": &award.name,
                                                "section": &award.sections[section.section_index].name,
                                                "activity": &award.sections[section.section_index].activities[section.activity_index].name,
                                                "date": date_str
                                            })).unwrap();
                                        }
                                    }
                                }
                            }

                            let content = data
                                .handlebars
                                .render(
                                    "sections/section_list",
                                    &json!({
                                        "section_rows": rows,
                                    }),
                                )
                                .unwrap();

                            let header: String = page::path_header(
                                &data,
                                &ctx.user.user_agent.privilege(),
                                &[
                                    (
                                        dir::ORGS_PAGE.to_owned(),
                                        dir::ORGS_TITLE.to_owned(),
                                        Privilege::RootLevel,
                                    ),
                                    (dir::org_path(org_id), org.name.clone(), Privilege::OrgLevel),
                                ],
                            );

                            let nav = page::org_nav(
                                &ctx,
                                &data,
                                org_id,
                                &org,
                                dir::org_path(org_id) + dir::UNREVIEWED_SECTIONS_PAGE,
                            );

                            let org_page = data
                                .handlebars
                                .render(
                                    "org/org_root",
                                    &json!({
                                        "header": header,
                                        "org_nav": nav,
                                        "body": content,
                                    }),
                                )
                                .unwrap();

                            let body = page::render_page(
                                Some(ctx),
                                &data,
                                dir::APP_NAME.to_owned()
                                    + " | "
                                    + &org.name
                                    + " - Unreviewed Sections",
                                dir::APP_NAME.to_owned(),
                                org_page,
                            )
                            .unwrap();

                            HttpResponse::new(http::StatusCode::OK).set_body(Body::from(body))
                        }
                        _ => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                            .set_body(Body::from("Failed to fetch org!")),
                    }
                } else {
                    page::not_authorized_page(Some(ctx), &data)
                }
            }
            Ok(None) => page::redirect_to_login(&req),

            Err(e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                .set_body(Body::from(format!("Error: {}", e))),
        }
    } else {
        HttpResponse::new(http::StatusCode::BAD_REQUEST).set_body(Body::from("Invalid org_id"))
    }
}
