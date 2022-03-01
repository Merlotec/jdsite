use actix_web::{body::Body, http, web, HttpRequest, HttpResponse};
use std::str::FromStr;
use std::sync::Arc;

use serde_json::json;

use crate::data::SharedData;

use crate::dir;
use crate::link;
use crate::login;
use crate::org;
use crate::page;
use crate::user;
use crate::util;

use user::Privilege;

use actix_web::{get, post};

#[get("/stats/{award}/{section}")]
pub async fn stats_section_get(
    data: web::Data<Arc<SharedData>>,
    req: HttpRequest,
    path: web::Path<(String, usize)>,
) -> HttpResponse {
    match data.authenticate_context_from_request(&req, true) {
        Ok(Some(ctx)) => {
            if ctx.user.user_agent.can_view_stats() {
                let award = path.0.0;
                let section_idx = path.0.1;
                let mut activities: String = String::new();
                if let Some(aw) = data.awards.get(&award) {
                    let section = &aw.sections[path.0.1];
                    let stats = data.get_activity_stats();

                    let section_point = &stats.awards.get(&award).unwrap().sections[section_idx];
                    let total_section: usize = section_point.total;

                    let mut total_completions: usize = 0;

                    for (id, activity) in section.activities.iter() {
                        let desc: String = {
                            match data.handlebars.render(&activity.activity_url, &()) {
                                Ok(data) => data,
                                Err(e) => format!("Failed to render: {}", e),
                            }
                        };

                        let activity_point = section_point.activities.get(id).unwrap();

                        total_completions += activity_point.completed;

                        let share: String = {
                            if total_section == 0 {
                                "N/A".to_string()
                            } else {
                                let p = (activity_point.selected as f32 / total_section as f32) * 100.0;
                                format!("{:.prec$}%", p, prec = 1)
                            }
                        };

                        let completion_rate: String =  {
                            if activity_point.selected == 0 {
                                "N/A".to_string()
                            } else {
                                let p = (activity_point.completed as f32 / activity_point.selected as f32) * 100.0;
                                format!("{:.prec$}%", p, prec = 1)
                            }
                        };

                        activities += &data
                            .handlebars
                            .render(
                                "stats/activity_option_stat",
                                &json!({
                                "activity": id,
                                "title": &activity.name,
                                "subtitle": &activity.subtitle,
                                "choices": activity_point.selected,
                                "share": share,
                                "completions": activity_point.completed,
                                "completion_rate": completion_rate,
                        }),
                            )
                            .unwrap();
                    }

                    let total_rate: String = {
                        if section_point.total == 0 {
                            "N/A".to_owned()
                        } else {
                            let p = (total_completions as f32 / section_point.total as f32) * 100.0;
                            format!("{:.prec$}%", p, prec = 1)
                        }
                    };

                    let body: String = data.handlebars.render("stats/section_stats", &json!({
                        "back_url": format!("/stats/{}", &award),
                        "section_name": &section.name,
                        "section_image_url": section.image_url,
                        "activities": activities,

                        "choices": section_point.total,
                        "completions": total_completions,
                        "completion_rate": total_rate,
                     })).unwrap();


                    let body = page::render_page(
                        Some(ctx),
                        &data,
                        dir::APP_NAME.to_owned() + " | Activity Stats",
                        dir::EXTENDED_APP_NAME.to_owned(),
                        body,
                    ).unwrap();

                    HttpResponse::new(http::StatusCode::OK).set_body(Body::from(body))
                } else {
                    HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                        .set_body(Body::from(format!("Failed to fetch section!")))
                }


            } else {
                page::not_authorized_page(Some(ctx), &data)
            }
        }
        Ok(None) => page::redirect_to_login(&req),

        Err(e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
            .set_body(Body::from(format!("Error: {}", e))),
    }
}

#[get("/stats")]
pub async fn stats_get(
    data: web::Data<Arc<SharedData>>,
    req: HttpRequest,
) -> HttpResponse {
    match data.authenticate_context_from_request(&req, true) {
        Ok(Some(ctx)) => {
            if ctx.user.user_agent.can_view_stats() {
                let mut awards = String::new();
                for (award_id, award) in data.awards.iter() {
                    awards += &data
                        .handlebars
                        .render(
                            "stats/stats_award_option",
                            &json!({
                                "award_stat_path": format!("/stats/{}", award_id),
                                "award_image_url": &award.image_url,
                                "title": &award.name,
                        }),
                        )
                        .unwrap();
                }

                let body: String = data.handlebars.render("stats/stats", &json!({
                        "awards": awards,
                     })).unwrap();


                let body = page::render_page(
                    Some(ctx),
                    &data,
                    dir::APP_NAME.to_owned() + " | Stats",
                    dir::EXTENDED_APP_NAME.to_owned(),
                    body,
                ).unwrap();

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

#[get("/stats/{award}")]
pub async fn stats_award_get(
    data: web::Data<Arc<SharedData>>,
    req: HttpRequest,
    path: web::Path<String>,
) -> HttpResponse {
    match data.authenticate_context_from_request(&req, true) {
        Ok(Some(ctx)) => {
            if ctx.user.user_agent.can_view_stats() {
                let award_id = &path.0;
                if let Some(award) = data.awards.get(award_id) {
                    let mut sections_body: String = String::new();
                    let mut completed_count: usize = 0;
                    for (i, section) in award.sections.iter().enumerate() {

                        sections_body += &data.handlebars.render("client/client_section_bubble", &json!({
                                                        "section_url": format!("/stats/{}/{}", award_id, i),
                                                        "section_image_url": &section.image_url,
                                                        "section_title": &section.name,
                                                        "activity_title": "Click to view stats",
                                                        "activity_title_class": "activity-chosen",
                                                        "state": "",
                                                        "state_class": "",
                                                    })).unwrap();
                    }

                    let body: String = data
                        .handlebars
                        .render(
                            "stats/stats_sections",
                            &json!({
                                "back_url": "/stats",
                                "award_icon": &award.image_url,
                                "award": &award.name,
                                "sections": sections_body,
                            }),
                        )
                        .unwrap();

                    let body = page::render_page(
                        Some(ctx),
                        &data,
                        format!("{} | {} Stats", dir::APP_NAME.to_owned(), &award.short_name),
                        dir::EXTENDED_APP_NAME.to_owned(),
                        body,
                    ).unwrap();

                    HttpResponse::new(http::StatusCode::OK).set_body(Body::from(body))
                } else {
                    HttpResponse::new(http::StatusCode::BAD_REQUEST)
                        .set_body(Body::from(format!("Invalid award!")))
                }


            } else {
                page::not_authorized_page(Some(ctx), &data)
            }
        }
        Ok(None) => page::redirect_to_login(&req),

        Err(e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
            .set_body(Body::from(format!("Error: {}", e))),
    }
}