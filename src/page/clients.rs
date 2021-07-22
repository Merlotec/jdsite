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
use crate::section;
use crate::link;

use user::Privilege;

use actix_web::{get, post};

#[get("/org/{org}/clients")]
pub async fn clients_get(data: web::Data<Arc<SharedData>>, req: HttpRequest, org_path_str: web::Path<String>) -> HttpResponse {
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
                                        if let user::UserAgent::Client { org_id: client_org_id, class, award_index, sections } = &user.user_agent {
                                            if client_org_id == &org_id {
                                                if let Some(award) = data.awards.get(*award_index) {
                                                     // Get section info
                                                    let mut unreviewed: u32 = 0;
                                                    //let mut completed: u32 = 0;

                                                    let section_styles: Vec<String> = sections.iter().map(|x| {
                                                        if let Some(section_id) = x {
                                                            if let Ok(Some(section)) = data.section_db.fetch(&section_id) {
                                                                if let section::SectionState::InReview(_) = section.state {
                                                                    unreviewed += 1;
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
                                                    })).unwrap();
                                                } else {
                                                    println!("[clients_get] Error - no award for index: {}", award_index);
                                                }
                                            }
                                        }
                                    },
                                    _ => {},
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

                            let content = data.handlebars.render("client/client_list", &json!({
                                "credits": org.credits,
                                "add_client_button": add_client_button,
                                "client_rows": rows,
                                "delete_user_url": dir::DELETE_USER_PATH.to_owned(),
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

                            let body = page::render_page(Some(ctx), &data, dir::APP_NAME.to_owned() + " | " + &org.name + " - Pupils", dir::APP_NAME.to_owned(), org_page).unwrap();

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

pub fn add_client_page(data: web::Data<Arc<SharedData>>, req: HttpRequest, org_path_str: web::Path<String>, err_msg: &str) -> HttpResponse {
    if let Ok(org_id) = org::OrgKey::from_str(&org_path_str) {
        match data.authenticate_context_from_request(&req, true) {
            Ok(Some(ctx)) => {
                if ctx.user.user_agent.can_view_org(&org_id) {
                    match data.org_db.fetch(&org_id) {
                        Ok(Some(org)) => {
                            
                            let awards: Vec<String> = data.awards.iter().map(|x| x.name.clone()).collect();

                            let content = data.handlebars.render("client/add_client", &json!({
                                "back_url": dir::org_path(org_id) + dir::CLIENTS_PAGE,
                                "awards": awards,
                                "add_client_url": dir::org_path(org_id) + dir::ADD_CLIENT_PATH,
                                "err_msg": err_msg,
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


#[get("/org/{org}/add_client")]
pub async fn add_client_get(data: web::Data<Arc<SharedData>>, req: HttpRequest, org: web::Path<String>) -> HttpResponse {
   add_client_page(data, req, org, "")
}

#[derive(serde::Deserialize)]
pub struct AddClientForm {
    forename: String,
    surname: String,
    email: String,
    class: String,
    award_index: usize,
}

#[post("/org/{org}/add_client")]
pub async fn add_client_post(data: web::Data<Arc<SharedData>>, req: HttpRequest, form: web::Form<AddClientForm>, org_path_str: web::Path<String>) -> HttpResponse {
    if let Ok(org_id) = org::OrgKey::from_str(&org_path_str) {
        match data.authenticate_context_from_request(&req, true) {
            Ok(Some(ctx)) => {
                if ctx.user.user_agent.can_view_org(&org_id) {
                    match data.org_db.fetch(&org_id) {
                        Ok(Some(org)) => {
                            if org.credits > 0 {
                                if util::is_string_server_valid(&form.forename) && 
                                util::is_string_server_valid(&form.surname) &&
                                util::is_string_server_valid(&form.email) &&
                                util::is_string_server_valid(&form.class) {

                                    let user: user::User = user::User {
                                        email: form.email.clone(),
                                        forename: form.forename.clone(),
                                        surname: form.surname.clone(),
                                        notifications: true,
                                        user_agent: user::UserAgent::Client {
                                            org_id,
                                            class: form.class.clone(),
                                            award_index: form.award_index,
                                            sections: [None; 6],
                                        }
                                    };

                                    let password: String = util::gen_password(8);

                                    match data.register_user(&user, &password, true)  {
                                        Ok(user_id) => {
                                            if let Ok(link_token) = data.link_manager.create_link(link::Link::ChangePassword(user_id), std::time::Duration::from_secs(dir::CHANE_PASSWORD_LINK_TIMEOUT_SECS)) {
                                                // send email.
                                                let link: String = "/user/change_password/".to_string() + &link_token.to_string();
                                                let addr: String = form.email.clone();
                        
                                                let subtitle: String = "<a href=\"".to_owned() + &link + "\">" + "Click here</a> to change your account password. Your default password is: " + &password;
                        
                                                if let Err(e) = data.send_email(
                                                    &addr, 
                                                    "Senior Duke - Change Your Password", 
                                                    "Change Password",
                                                    &subtitle, 
                                                    ""
                                                ) {
                                                    println!("Failed to send email: {}", e);
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
                
                                            let body = page::render_page(Some(ctx), &data, dir::APP_NAME.to_owned() + " | " + "Pupil Account Created", dir::APP_NAME.to_owned(), org_page).unwrap();
                
                                            HttpResponse::new(http::StatusCode::OK)
                                                .set_body(Body::from(body))

                                        },
                                        Err(login::LoginEntryError::UsernameExists) =>  add_client_page(data, req, org_path_str, "This email is associated with another account!"),
                                        Err(e) =>  add_client_page(data, req, org_path_str, &format!("Something went wrong: ensure that the email is unique: {}", e)),
                                    }   
                                } else {
                                    add_client_page(data, req, org_path_str, "Invalid pupil details provided!")
                                }
                            } else {
                                add_client_page(data, req, org_path_str, "No more pupil credits remaining! Please contact support to purchase more.")
                            }
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


#[get("/org/{org}/client/{user}")]
pub async fn client_dashboard_get(data: web::Data<Arc<SharedData>>, req: HttpRequest, path: web::Path<(String, String)>) -> HttpResponse {
    if let Ok(org_id) = org::OrgKey::from_str(&(path.0).0) {
        if let Ok(user_id) = user::UserKey::from_str(&(path.0).1) {
            match data.authenticate_context_from_request(&req, true) {
                Ok(Some(ctx)) => {
                    match data.user_db.fetch(&user_id) {
                        Ok(Some(user)) => {
                            if let user::UserAgent::Client { award_index, sections, .. } = &user.user_agent {
                                if ctx.user.user_agent.can_view_user(&user.user_agent) || ctx.user_id == user_id {
                                    match data.org_db.fetch(&org_id) {
                                        Ok(Some(org)) => {
                                            if let Some(award) = data.awards.get(*award_index) {
                                                let mut sections_body: String = String::new();
    
                                                for (i, section) in award.sections.iter().enumerate() {
                                                    let (activity_title, activity_title_class, state, state_class): (String, String, String, String) = {
                                                        if let Some(section_id) = sections[i] {
                                                            if let Ok(Some(section_instance)) = data.section_db.fetch(&section_id) {
                                                                let outstanding: &str = {
                                                                    if section_instance.outstanding {
                                                                        " - <span style=\"color: pink;\">Outstanding</span>"
                                                                    } else {
                                                                        ""
                                                                    }
                                                                };

                                                                (
                                                                section.activities[section_instance.activity_index].name.clone(), 
                                                                    "activity-chosen".to_owned(),
                                                                    section_instance.state.to_string() + outstanding,
                                                                    section_instance.state.css_class(),
                                                                )
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
        
                                                let body: String = data.handlebars.render("client/client_dashboard", &json!({
                                                    "award": &award.name,
                                                    "sections": sections_body,
                                                })).unwrap();
        
                                                let header: String = page::path_header(&data, &ctx.user.user_agent.privilege(), &[
                                                    (dir::ORGS_PAGE.to_owned(), dir::ORGS_TITLE.to_owned(), Privilege::RootLevel), 
                                                    (dir::org_path(org_id), org.name.clone(), Privilege::OrgLevel),
                                                    (dir::client_path(org_id, user_id), user.name(), Privilege::ClientLevel)
                                                ]);

                                                let header_properties: String = {
                                                    if ctx.user.user_agent.privilege() == Privilege::ClientLevel {
                                                        "hidden=\"true\"".to_owned()
                                                    } else {
                                                        String::new()
                                                    }
                                                };
        
                                                let root: String = data.handlebars.render("client/client_root", &json!({
                                                    "header": header,
                                                    "body": body,
                                                    "header_properties": header_properties,
                                                })).unwrap();
        
                                                let body = page::render_page(Some(ctx), &data, dir::APP_NAME.to_owned() + " | " + "Pupil Dashboard", dir::APP_NAME.to_owned(), root).unwrap();
                                
                                                HttpResponse::new(http::StatusCode::OK)
                                                    .set_body(Body::from(body))
                                            } else {
                                                HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                                                    .set_body(Body::from("Award index out of range!"))
                                            }
                                            
                                        },
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
                .set_body(Body::from("Invalid user_id"))
        }
    } else {
        HttpResponse::new(http::StatusCode::BAD_REQUEST)
            .set_body(Body::from("Invalid org_id"))
    }
}
