use actix_files::NamedFile;
use actix_web::http::header::{ContentDisposition, DispositionType, DispositionParam};
use actix_web::{body::Body, http, web, HttpRequest, HttpResponse};

use std::sync::Arc;

use serde_json::json;


use crate::data::SharedData;

use crate::dir;

use crate::page;
use crate::user;
use crate::link;
use crate::util;
use crate::login;

use actix_web::{get, post};
use crate::user::UserKey;

#[derive(Debug, serde::Deserialize)]
pub struct AccountsQuery {
    pub search: Option<String>,
    pub view_all: Option<bool>
}

#[get("/admin/accounts")]
pub async fn accounts_get(data: web::Data<Arc<SharedData>>, req: HttpRequest, query: web::Query<AccountsQuery>) -> HttpResponse {
    match data.authenticate_context_from_request(&req, true) {
        Ok(Some(ctx)) => {
            if ctx.user.user_agent.can_view_accounts() {
                let view_all: bool = query.view_all.unwrap_or(false);

                let mut rows: String = String::new();

                let mut unordered_users: Vec<(user::UserKey, user::User)> = Vec::new();
                let mut invalid_users: Vec<user::UserKey> = Vec::new();
                if view_all {
                    data.user_db.for_each_key(|user_id, user| {
                        if let Some(user) = user {
                            unordered_users.push((*user_id, user));
                        } else {
                            invalid_users.push(*user_id);
                        }
                        
                    });
                } else if let Some(search) = &query.search {
                    if !search.trim().is_empty() {
                        data.user_db.for_each(|user_id, user| {
                            // Perform lookup.
                            let name: String = (user.forename.clone() + " " + &user.surname).to_lowercase();
                            let search_span: String = search.to_lowercase();
                            if name.contains(&search_span) {
                                unordered_users.push((*user_id, user));
                            } else if user.email.contains(&search_span) {
                                unordered_users.push((*user_id, user));
                            }
                        });
                    } 
                }
                

                let mut users: Vec<(user::UserKey, user::User)> = Vec::with_capacity(unordered_users.len());
                // Order users by role.
                for (user_id, new_user) in unordered_users {
                    let mut insertion_index: usize = users.len();
                    'inner: for (i, (_, existing_user)) in users.iter().enumerate() {
                        if existing_user.user_agent.privilege().magnitude() < new_user.user_agent.privilege().magnitude() {
                            insertion_index = i;
                            break 'inner;
                        }
                    }
                    users.insert(insertion_index, (user_id, new_user));
                }

                for (user_id, user) in users.iter() {
                    let target_url: String = {
                        match user.user_agent {
                            user::UserAgent::Owner => dir::ORGS_PAGE.to_owned(),
                            user::UserAgent::Admin => dir::ORGS_PAGE.to_owned(),
                            user::UserAgent::Organisation(org_id) => dir::org_path(org_id),
                            user::UserAgent::Associate(org_id) => dir::org_path(org_id),
                            user::UserAgent::Client { org_id, .. } => dir::client_path(org_id, *user_id),
                        }
                    };

                    let role: String = match user.user_agent {
                        user::UserAgent::Owner => "Owner".to_owned(),
                        user::UserAgent::Admin => "Admin".to_owned(),
                        user::UserAgent::Organisation(_) => "Organisation Administrator".to_owned(),
                        user::UserAgent::Associate(_) => "Teacher".to_owned(),
                        user::UserAgent::Client { .. } => "Pupil".to_owned(),
                    };

                    let delete_user_hidden: String = {
                        if user_id == &ctx.user_id || user.user_agent.privilege().magnitude() >= ctx.user.user_agent.privilege().magnitude() {
                            "hidden".to_owned()
                        } else {
                            String::new()
                        }
                    };

                    rows += &data.handlebars.render("admin/account_row", &json!({
                        "target_url": target_url,
                        "user_url": dir::user_path(*user_id),
                        "user_id": user_id.to_string(),
                        "name": user.name(),
                        "role": role,
                        "email": user.email,
                        "delete_user_hidden": delete_user_hidden,
                    })).unwrap();
                }

                for user_id in invalid_users {
                    rows += &data.handlebars.render("admin/account_row", &json!({
                        "user_id": user_id.to_string(),
                        "name": "Invalid User",
                        "role": "N/A",
                        "email": "N/A",
                        "delete_user_hidden": "",
                        "invalid": true,
                    })).unwrap();
                }

                let content = data
                    .handlebars
                    .render(
                        "admin/accounts",
                        &json!({
                            "search_url": dir::ACCOUNTS_PATH,
                            "add_admin_url": dir::ADD_ADMIN_PATH,
                            "user_rows": rows,
                            "display_user_table": query.search.is_some() || view_all,
                            "delete_user_url": dir::DELETE_USER_PATH,
                            "not_viewing_all": !view_all || query.search.is_some(),
                            "can_add_admin": ctx.user.user_agent.can_add_admin()
                        }),
                    )
                    .unwrap();

                let body = page::render_page(
                    Some(ctx),
                    &data,
                    dir::APP_NAME.to_owned() + " | User Accounts",
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


pub fn add_admin_page(
    data: web::Data<Arc<SharedData>>,
    req: HttpRequest,
    err_msg: &str,
) -> HttpResponse {
    match data.authenticate_context_from_request(&req, true) {
        Ok(Some(ctx)) => {
            if ctx.user.user_agent.can_add_admin() {
                let content = data.handlebars.render("admin/add_admin", &json!({
                    "back_url": dir::ACCOUNTS_PATH,
                    "add_associate_url": dir::ADD_ADMIN_PATH,
                    "err_msg": err_msg,
                })).unwrap();

                let body = page::render_page(
                    Some(ctx),
                    &data,
                    dir::APP_NAME.to_owned() + " | Add Admin",
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

#[get("/admin/add_admin")]
pub async fn add_admin_get(
    data: web::Data<Arc<SharedData>>,
    req: HttpRequest,
) -> HttpResponse {
    add_admin_page(data, req, "")
}

#[derive(serde::Deserialize)]
pub struct AddAdminForm {
    forename: String,
    surname: String,
    email: String,
}

#[post("/admin/add_admin")]
pub async fn add_admin_post(
    data: web::Data<Arc<SharedData>>,
    req: HttpRequest,
    form: web::Form<AddAdminForm>,
) -> HttpResponse {
    match data.authenticate_context_from_request(&req, true) {
        Ok(Some(ctx)) => {
            if ctx.user.user_agent.can_add_admin() {
                if !util::is_string_server_valid(&form.forename) {
                    add_admin_page(
                        data,
                        req,
                        "Invalid forename provided",
                    )
                } else if !util::is_string_server_valid(&form.surname) {
                    add_admin_page(
                        data,
                        req,
                        "Invalid surname provided",
                    )
                } else if !util::is_email_valid(&form.email) {
                    add_admin_page(
                        data,
                        req,
                        "Invalid email provided",
                    )
                } else {
                    let user: user::User = user::User {
                        email: form.email.clone(),
                        forename: form.forename.clone(),
                        surname: form.surname.clone(),
                        notifications: true,
                        user_agent: user::UserAgent::Admin,
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
                                        "account_type": "global administrator",
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

                            let content = data.handlebars.render("admin/admin_added", &json!({
                                "back_url": dir::ACCOUNTS_PATH,
                                "add_admin_url": dir::ADD_ADMIN_PATH,
                                "attributes": attrs,
                            })).unwrap();

                            let body = page::render_page(Some(ctx), &data, dir::APP_NAME.to_owned() + " | " + "Admin Account Created", dir::EXTENDED_APP_NAME.to_owned(), content).unwrap();

                            HttpResponse::new(http::StatusCode::OK)
                                .set_body(Body::from(body))

                        },
                        Err(login::LoginEntryError::UsernameExists) =>  add_admin_page(data, req, "This email is associated with another account!"),
                        Err(e) =>  add_admin_page(data, req, &format!("Something went wrong: ensure that the email is unique: {}", e)),
                    }
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



pub fn delete_data_page(
    data: web::Data<Arc<SharedData>>,
    req: HttpRequest,
    err_msg: &str,
) -> HttpResponse {
    match data.authenticate_context_from_request(&req, true) {
        Ok(Some(ctx)) => {
            if ctx.user.user_agent.can_administrate() {
                let content = data.handlebars.render("admin/delete_data", &json!({
                    "back_url": dir::ADMIN_PATH,
                    "delete_url": dir::DELETE_PATH,
                    "err_msg": err_msg,
                })).unwrap();

                let body = page::render_page(
                    Some(ctx),
                    &data,
                    dir::APP_NAME.to_owned() + " | Delete Data",
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

#[get("/admin/delete")]
pub async fn delete_data_get(
    data: web::Data<Arc<SharedData>>,
    req: HttpRequest,
) -> HttpResponse {
    delete_data_page(data, req, "")
}

#[derive(serde::Deserialize)]
pub struct DeleteDataForm {
    delete_pupils: Option<String>,
    delete_credits: Option<String>,
    delete_orgs: Option<String>,
    password: String,
}

#[post("/admin/delete")]
pub async fn delete_data_post(
    data: web::Data<Arc<SharedData>>,
    req: HttpRequest,
    form: web::Form<DeleteDataForm>,
) -> HttpResponse {
    match data.authenticate_context_from_request(&req, true) {
        Ok(Some(ctx)) => {
            if ctx.user.user_agent.can_administrate() {
                if let Ok(_) = data.login_db.authenticate(&ctx.user.email, &form.password) {
                    // If nothing is selected, tell the user.
                    let mut info: String;
                    if form.delete_orgs.is_none() && form.delete_credits.is_none() && form.delete_credits.is_none() {
                        info = "You did not select any data to delete!".to_owned();
                    } else {
                        info = "You have deleted the following data: ".to_owned();
                        if let Some(_) = form.delete_orgs {
                            for org_id in data.org_db.keys() {
                                let _ = data.delete_org(&org_id);
                            }
                            info += "<b>organisations</b>";
                        } else {
                            if let Some(_) = form.delete_credits {
                                data.org_db.for_each_write(|mut org| { org.credits = 0; });
                                info += "<b>credits</b>"
                            }
                            if let Some(_) = form.delete_pupils {
                                let mut users: Vec<UserKey> = Vec::new();
                                data.user_db.for_each(|k, v| {
                                    if v.user_agent.is_client() {
                                        users.push(*k);
                                    }
                                });
                                for user_id in users {
                                    data.delete_user(&user_id);
                                }
                                if !info.is_empty() {
                                    info += ", <b>pupils</b>";
                                } else {
                                    info += "<b>pupils</b>";
                                }
                            }
                        }
                    }

                    
                    let content = data.handlebars.render("admin/data_deleted", &json!({
                                "back_url": dir::ADMIN_PATH,
                                "deletion_info": info,
                            })).unwrap();

                    let body = page::render_page(Some(ctx), &data, dir::APP_NAME.to_owned() + " | " + "Data Deleted", dir::EXTENDED_APP_NAME.to_owned(), content).unwrap();

                    HttpResponse::new(http::StatusCode::OK)
                        .set_body(Body::from(body))
                } else {
                    delete_data_page(data, req, "Incorrect confirmation password specified!")
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

#[get("/admin")]
pub async fn admin_get(data: web::Data<Arc<SharedData>>, req: HttpRequest) -> HttpResponse {
    match data.authenticate_context_from_request(&req, true) {
        Ok(Some(ctx)) => {
            if ctx.user.user_agent.can_administrate() {
                let disk = match sys_info::disk_info() {
                    Ok(info) => {
                        let free = info.free as f32;
                        let total = info.total as f32;
                        let p = ((total - free) / total) * 100.0;
                        let free_gb = free / 1_000_000.0;
                        let total_gb = total / 1_000_000.0;

                        format!("{:.precgb$}GB of {:.precgb$}GB ({:.prec$}% used)", free_gb, total_gb, p, precgb = 2, prec = 1)
                    },
                    Err(_) => "Error retrieving data!".to_owned(),
                };

                let memory = match sys_info::mem_info() {
                    Ok(info) => {
                        let free = info.free as f32;
                        let total = info.total as f32;
                        let p = ((total - free) / total) * 100.0;
                        let free_gb = free / 1_000_000.0;
                        let total_gb = total / 1_000_000.0;

                        format!("{:.precgb$}GB of {:.precgb$}GB ({:.prec$}% used)", free_gb, total_gb, p, precgb = 2, prec = 1)
                    },
                    Err(_) => "Error retrieving data!".to_owned(),
                };


                let content = data
                    .handlebars
                    .render(
                        "admin/admin",
                        &json!({
                            "disk": disk,
                            "memory": memory,
                            "log_url": dir::DOWNLOAD_LOG_PATH,
                            "delete_url": dir::DELETE_PATH,
                        }),
                    )
                    .unwrap();

                let body = page::render_page(
                    Some(ctx),
                    &data,
                    dir::APP_NAME.to_owned() + " | Admin",
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

#[get("/admin/log")]
pub async fn log_get(data: web::Data<Arc<SharedData>>, req: HttpRequest) -> HttpResponse {
    match data.authenticate_context_from_request(&req, true) {
        Ok(Some(ctx)) => {
            if ctx.user.user_agent.can_administrate() {
                match  web::block(|| NamedFile::open(dir::LOG_PATH)).await {
                    Ok(file) => file.set_content_disposition(ContentDisposition {
                        disposition: DispositionType::Attachment,
                        parameters: vec![
                            DispositionParam::Name(String::from("log")),
                            DispositionParam::Filename(String::from("log.txt")),
                        ],
                    }).into_response(&req).unwrap(),
                    Err(e) => HttpResponse::new(http::StatusCode::BAD_REQUEST)
                        .set_body(Body::from(format!("Failed to download log file: {}", e))),
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