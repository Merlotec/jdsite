use std::sync::Arc;

use actix_web::{body::Body, http, web, HttpRequest, HttpResponse};

use serde_json::json;

use crate::data::SharedData;

use crate::dir;
use crate::page;

use actix_web::get;

#[get("/outstanding")]
pub async fn outstanding_get(data: web::Data<Arc<SharedData>>, req: HttpRequest) -> HttpResponse {
    match data.authenticate_context_from_request(&req, true) {
        Ok(Some(ctx)) => {
            if ctx.user.user_agent.can_view_outstanding() {
                let mut rows: String = String::new();
                data.outstanding_sections_db.for_each(|section_id, _| {
                    if let Ok(Some(section)) = data.section_db.fetch(&section_id) {
                        if let Ok(Some(user)) = data.user_db.fetch(&section.user_id) {
                            if let Some(award) = data.awards.get(&section.award) {
                                let client_url: String = {
                                    if let Some(org_id) = user.user_agent.org_id() {
                                        dir::client_path(org_id, section.user_id)
                                    } else {
                                        String::new()
                                    }
                                };

                                if let Some(activity) = award.sections[section.section_index].activities.get(&section.activity) {
                                    rows += &data.handlebars.render("sections/outstanding_section_row", &json!({
                                        "client_url": client_url,
                                        "user_url": dir::user_path(section.user_id),
                                        "section_url": "/section/".to_owned() + &section_id.to_string(),
                                        "name": user.name(),
                                        "email": user.email,
                                        "award": &award.name,
                                        "section": &award.sections[section.section_index].name,
                                        "activity": &activity.name,
                                    })).unwrap();
                                }
                            }
                        }
                    }
                });

                let content = data
                    .handlebars
                    .render(
                        "sections/outstanding_list",
                        &json!({
                            "section_rows": rows,
                        }),
                    )
                    .unwrap();

                let body = page::render_page(
                    Some(ctx),
                    &data,
                    dir::APP_NAME.to_owned() + " | Outstanding Achievements",
                    dir::EXTENDED_APP_NAME.to_owned(),
                    content,
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
    }
}
