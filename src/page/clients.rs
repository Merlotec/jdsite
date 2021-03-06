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
use crate::section;
use crate::user;
use crate::util;

use user::Privilege;

use actix_web::{get, post};

#[get("/org/{org}/clients")]
pub async fn clients_get(
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

                            for user_id in org.clients.iter() {
                                match data.user_db.fetch(user_id) {
                                    Ok(Some(user)) => {
                                        if let user::UserAgent::Client {
                                            org_id: client_org_id,
                                            class,
                                            award,
                                            sections,
                                        } = &user.user_agent
                                        {
                                            if client_org_id == &org_id {
                                                if let Some(award) = data.awards.get(award) {
                                                    // Get section info
                                                    let mut unreviewed: u32 = 0;
                                                    let mut completed_count: usize = 0;

                                                    let section_styles: Vec<String> = sections.iter().map(|x| {
                                                        if let Some(section_id) = x {
                                                            if let Ok(Some(section)) = data.section_db.fetch(&section_id) {
                                                                if let section::SectionState::InReview(_) = section.state {
                                                                    unreviewed += 1;
                                                                } else if let section::SectionState::Completed = section.state {
                                                                    completed_count += 1;
                                                                }
                                                                let border: &str = {
                                                                    if section.outstanding {
                                                                        "border: 1px solid pink;"
                                                                    } else {
                                                                        "border: none;"
                                                                    }
                                                                };
                                                                "background-color: ".to_owned() + &section.state.css_color() + "; " + border
                                                            } else {
                                                                "".to_owned()
                                                            }
                                                        } else {
                                                            "".to_owned()
                                                        }
                                                    }).collect();

                                                    let completed: bool = completed_count == sections.len();

                                                    rows += &data.handlebars.render("client/client_row", &json!({
                                                        "client_url": dir::client_path(org_id, *user_id),
                                                        "user_url": dir::user_path(*user_id),
                                                        "user_id": user_id,
                                                        "name": user.name(),
                                                        "class": class,
                                                        "email": user.email,
                                                        "award": &award.short_name,
                                                        "section_styles": section_styles,
                                                        "unreviewed_sections": unreviewed.to_string(),
                                                        "completed": completed,
                                                    })).unwrap();
                                                } else {
                                                    log::error!("[clients_get] Error - no award for id: {}", award);
                                                }
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }

                            let add_client_button: String = {
                                if org.credits > 0 {
                                    data.handlebars.render("client/add_client_button", &json!({
                                        "add_client_url": dir::org_path(org_id) + dir::ADD_CLIENT_PATH,
                                    })).unwrap()
                                } else {
                                    String::new()
                                }
                            };

                            let content = data
                                .handlebars
                                .render(
                                    "client/client_list",
                                    &json!({
                                        "credits": org.credits,
                                        "add_client_button": add_client_button,
                                        "client_rows": rows,
                                        "delete_user_url": dir::DELETE_USER_PATH.to_owned(),
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
                                dir::org_path(org_id) + dir::CLIENTS_PAGE,
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
                                dir::APP_NAME.to_owned() + " | Pupils - " + &org.name,
                                dir::EXTENDED_APP_NAME.to_owned(),
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

pub fn add_client_page(
    data: web::Data<Arc<SharedData>>,
    req: HttpRequest,
    org_path_str: web::Path<String>,
    err_msg: &str,
) -> HttpResponse {
    if let Ok(org_id) = org::OrgKey::from_str(&org_path_str) {
        match data.authenticate_context_from_request(&req, true) {
            Ok(Some(ctx)) => {
                if ctx.user.user_agent.can_view_org(&org_id) {
                    match data.org_db.fetch(&org_id) {
                        Ok(Some(org)) => {
                            let awards: Vec<_> =
                                data.awards.iter().map(|(id, x)| { 
                                    json!({
                                        "title": x.name.clone(),
                                        "award": id,
                                    })
                                }).collect();

                            let content = data.handlebars.render("client/add_client", &json!({
                                "back_url": dir::org_path(org_id) + dir::CLIENTS_PAGE,
                                "awards": awards,
                                "add_client_url": dir::org_path(org_id) + dir::ADD_CLIENT_PATH,
                                "err_msg": err_msg,
                            })).unwrap();

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
                                dir::org_path(org_id) + dir::CLIENTS_PAGE,
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
                                dir::APP_NAME.to_owned() + " | Add Pupil - " + &org.name,
                                dir::EXTENDED_APP_NAME.to_owned(),
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

#[get("/org/{org}/add_client")]
pub async fn add_client_get(
    data: web::Data<Arc<SharedData>>,
    req: HttpRequest,
    org: web::Path<String>,
) -> HttpResponse {
    add_client_page(data, req, org, "")
}

#[derive(serde::Deserialize)]
pub struct AddClientForm {
    forename: String,
    surname: String,
    email: String,
    class: String,
    award: String,
}

#[post("/org/{org}/add_client")]
pub async fn add_client_post(
    data: web::Data<Arc<SharedData>>,
    req: HttpRequest,
    form: web::Form<AddClientForm>,
    org_path_str: web::Path<String>,
) -> HttpResponse {
    if let Ok(org_id) = org::OrgKey::from_str(&org_path_str) {
        match data.authenticate_context_from_request(&req, true) {
            Ok(Some(ctx)) => {
                if ctx.user.user_agent.can_view_org(&org_id) {
                    match data.org_db.fetch(&org_id) {
                        Ok(Some(org)) => {
                            if org.credits > 0 {

                                if !util::is_string_server_valid(&form.forename){
                                    add_client_page(
                                        data,
                                        req,
                                        org_path_str,
                                        "Invalid pupil forname provided",
                                    )
                                } else if !util::is_string_server_valid(&form.surname) {
                                    add_client_page(
                                        data,
                                        req,
                                        org_path_str,
                                        "Invalid pupil surname provided",
                                    )
                                } else if !util::is_email_valid(&form.email) {
                                    add_client_page(
                                        data,
                                        req,
                                        org_path_str,
                                        "Invalid pupil email provided",
                                    )
                                } else if !util::is_optional_string_server_valid(&form.class) {
                                    add_client_page(
                                        data,
                                        req,
                                        org_path_str,
                                        "Invalid class name provided",
                                    )
                                } else {
                                    if !data.awards.contains_key(&form.award) {
                                        return add_client_page(
                                            data,
                                            req,
                                            org_path_str,
                                            "Invalid award provided",
                                        );
                                    }
                                    let user: user::User = user::User {
                                        email: form.email.clone(),
                                        forename: form.forename.clone(),
                                        surname: form.surname.clone(),
                                        notifications: true,
                                        user_agent: user::UserAgent::Client {
                                            org_id,
                                            class: form.class.clone(),
                                            award: form.award.clone(),
                                            sections: [None; 6],
                                        },
                                    };

                                    let password: String = util::gen_password(8);

                                    match data.register_user(&user, &password, true)  {
                                        Ok(user_id) => {
                                            if let Ok(link_token) = data.link_manager.create_link(link::Link::ChangePassword(user_id), std::time::Duration::from_secs(dir::CHANE_PASSWORD_LINK_TIMEOUT_SECS)) {
                                                // send email.
                                                let link: String = dir::make_absolute_url(&("/user/change_password/".to_string() + &link_token.to_string()));
                                                let addr: String = form.email.clone();
                                                
                                                let subtitle: String = data
                                                .handlebars
                                                .render(
                                                    "email/account_created",
                                                    &json!({
                                                        "name": user.name(),
                                                        "account_type": "pupil",
                                                        "org_name": &org.name,
                                                        "username": &user.email,
                                                        "password": &password,
                                                        "link": link,
                                                    }),
                                                )
                                            .unwrap();

                                                if data.send_email(
                                                    &addr, 
                                                    "Senior Duke - Welcome & Password Info", 
                                                    "Senior Duke - Welcome & Password Info",
                                                    &subtitle, 
                                                    ""
                                                ).is_none() {
                                                    log::error!("Failed to send email!");
                                                }
                                            }
                                            
                                            let mut attrs: String = String::new();

                                            attrs += &data.handlebars.render("user/user_attribute", &json!({
                                                "attribute_name": "Username",
                                                "attribute_value": user.email,
                                            })).unwrap();
                
                                            attrs += "<br><br>";
                                            attrs += &data.handlebars.render("user/user_attribute", &json!({
                                                "attribute_name": "Password",
                                                "attribute_value": password,
                                            })).unwrap();

                                            let content = data.handlebars.render("client/client_added", &json!({
                                                "back_url": dir::org_path(org_id) + dir::CLIENTS_PAGE,
                                                "add_client_url": dir::org_path(org_id) + dir::ADD_CLIENT_PATH,
                                                "attributes": attrs,
                                            })).unwrap();
                
                                            let header: String = page::path_header(&data, &ctx.user.user_agent.privilege(), &[
                                                (dir::ORGS_PAGE.to_owned(), dir::ORGS_TITLE.to_owned(), Privilege::RootLevel), 
                                                (dir::org_path(org_id), org.name.clone(), Privilege::OrgLevel),
                                            ]);
                
                                            let nav = page::org_nav(&ctx, &data, org_id, &org, dir::org_path(org_id) + dir::CLIENTS_PAGE);
                
                                            let org_page = data.handlebars.render("org/org_root", &json!({
                                                "header": header,
                                                "org_nav": nav,
                                                "body": content,
                                            })).unwrap();
                
                                            let body = page::render_page(Some(ctx), &data, dir::APP_NAME.to_owned() + " | Pupil Account Created - " + &org.name, dir::EXTENDED_APP_NAME.to_owned(), org_page).unwrap();
                
                                            HttpResponse::new(http::StatusCode::OK)
                                                .set_body(Body::from(body))

                                        },
                                        Err(login::LoginEntryError::UsernameExists) =>  add_client_page(data, req, org_path_str, "This email is associated with another account!"),
                                        Err(e) =>  add_client_page(data, req, org_path_str, &format!("Something went wrong: ensure that the email is unique: {}", e)),
                                    }
                                }
                            } else {
                                add_client_page(data, req, org_path_str, "No more pupil credits remaining! Please contact support to purchase more.")
                            }
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

#[get("/org/{org}/client/{user}")]
pub async fn client_dashboard_get(
    data: web::Data<Arc<SharedData>>,
    req: HttpRequest,
    path: web::Path<(String, String)>,
) -> HttpResponse {
    if let Ok(org_id) = org::OrgKey::from_str(&(path.0).0) {
        if let Ok(user_id) = user::UserKey::from_str(&(path.0).1) {
            match data.authenticate_context_from_request(&req, true) {
                Ok(Some(ctx)) => match data.user_db.fetch(&user_id) {
                    Ok(Some(user)) => {
                        if let user::UserAgent::Client {
                            award,
                            sections,
                            ..
                        } = &user.user_agent
                        {
                            if ctx.user.user_agent.can_view_user(&user.user_agent)
                                || ctx.user_id == user_id
                            {
                                match data.org_db.fetch(&org_id) {
                                    Ok(Some(org)) => {
                                        if let Some(award) = data.awards.get(award) {
                                            let mut sections_body: String = String::new();
                                            let mut completed_count: usize = 0;
                                            for (i, section) in award.sections.iter().enumerate() {
                                                let (activity_title, activity_title_class, state, state_class): (String, String, String, String) = {
                                                        if let Some(section_id) = sections[i] {
                                                            if let Ok(Some(section_instance)) = data.section_db.fetch(&section_id) {
                                                                if section_instance.state.is_completed() {
                                                                    completed_count += 1;
                                                                }

                                                                let outstanding: &str = {
                                                                    if section_instance.outstanding {
                                                                        " - <span style=\"color: pink;\">Outstanding</span>"
                                                                    } else {
                                                                        ""
                                                                    }
                                                                };
                                                                if let Some(activity) = section.activities.get(&section_instance.activity) {
                                                                    (
                                                                        activity.name.clone(), 
                                                                        "activity-chosen".to_owned(),
                                                                        section_instance.state.to_string() + outstanding,
                                                                        section_instance.state.css_class(),
                                                                    )
                                                                } else {
                                                                    (
                                                                        "Click to Select Challenge".to_owned(), 
                                                                        "activity-not-chosen".to_owned(),
                                                                        "Invalid Activity".to_owned(),
                                                                        String::new(),
                                                                    )
                                                                }
                                                            } else {
                                                                (
                                                                    "ERROR".to_owned(), 
                                                                    "activity-not-chosen".to_owned(),
                                                                    String::new(),
                                                                    "".to_owned(),
                                                                )
                                                            }
                                                        } else {
                                                            (
                                                                "Click to Select Challenge".to_owned(), 
                                                                "activity-not-chosen".to_owned(),
                                                                "Not Started".to_owned(),
                                                                String::new(),
                                                            )
                                                        }
                                                    };

                                                sections_body += &data.handlebars.render("client/client_section_bubble", &json!({
                                                        "section_url": dir::client_path(org_id, user_id) + dir::SECTION_ROOT + "/" + &i.to_string(),
                                                        "section_image_url": &section.image_url,
                                                        "section_title": &section.name,
                                                        "activity_title": &activity_title,
                                                        "activity_title_class": &activity_title_class,
                                                        "state": &state,
                                                        "state_class": &state_class,
                                                    })).unwrap();
                                            }

                                            let completed = completed_count == award.sections.len();

                                            let body: String = data
                                                .handlebars
                                                .render(
                                                    "client/client_dashboard",
                                                    &json!({
                                                        "award_icon": &award.image_url,
                                                        "award": &award.name,
                                                        "sections": sections_body,
                                                        "completed": completed,
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
                                                    (
                                                        dir::org_path(org_id),
                                                        org.name.clone(),
                                                        Privilege::OrgLevel,
                                                    ),
                                                    (
                                                        dir::client_path(org_id, user_id),
                                                        user.name(),
                                                        Privilege::ClientLevel,
                                                    ),
                                                ],
                                            );

                                            let header_properties: String = {
                                                if ctx.user.user_agent.privilege()
                                                    == Privilege::ClientLevel
                                                {
                                                    "hidden=\"true\"".to_owned()
                                                } else {
                                                    String::new()
                                                }
                                            };

                                            let root: String = data
                                                .handlebars
                                                .render(
                                                    "client/client_root",
                                                    &json!({
                                                        "header": header,
                                                        "body": body,
                                                        "header_properties": header_properties,
                                                    }),
                                                )
                                                .unwrap();

                                            let body = page::render_page(
                                                Some(ctx),
                                                &data,
                                                dir::APP_NAME.to_owned()
                                                    + " | "
                                                    + "Pupil Dashboard",
                                                dir::EXTENDED_APP_NAME.to_owned(),
                                                root,
                                            )
                                            .unwrap();

                                            HttpResponse::new(http::StatusCode::OK)
                                                .set_body(Body::from(body))
                                        } else {
                                            HttpResponse::new(
                                                http::StatusCode::INTERNAL_SERVER_ERROR,
                                            )
                                            .set_body(Body::from("Award index out of range!"))
                                        }
                                    }
                                    _ => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                                        .set_body(Body::from("Failed to fetch org!")),
                                }
                            } else {
                                page::not_authorized_page(Some(ctx), &data)
                            }
                        } else {
                            HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                                .set_body(Body::from("User is not a client!"))
                        }
                    }
                    Ok(None) => HttpResponse::new(http::StatusCode::BAD_REQUEST)
                        .set_body(Body::from("Could not find user!")),
                    Err(e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                        .set_body(Body::from(format!("Error: {}", e))),
                },
                Ok(None) => page::redirect_to_login(&req),

                Err(e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                    .set_body(Body::from(format!("Error: {}", e))),
            }
        } else {
            HttpResponse::new(http::StatusCode::BAD_REQUEST).set_body(Body::from("Invalid user_id"))
        }
    } else {
        HttpResponse::new(http::StatusCode::BAD_REQUEST).set_body(Body::from("Invalid org_id"))
    }
}
