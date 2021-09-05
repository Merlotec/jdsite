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

                if view_all {
                    data.user_db.for_each(|user_id, user| {
                        unordered_users.push((*user_id, user));
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
                    dir::APP_NAME.to_owned() + " | Outstanding Sections",
                    dir::APP_NAME.to_owned(),
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
                    dir::APP_NAME.to_owned() + " | Add Admin Account",
                    dir::APP_NAME.to_owned(),
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
                if util::is_string_server_valid(&form.forename)
                            && util::is_string_server_valid(&form.surname)
                            && util::is_email_valid(&form.email)
                        {
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
                                        let subtitle: String = "You have successfully been registered for a Senior Duke admin account! ".to_owned() + "<a href=\"" + &link + "\">" + "Click here</a> to change your account password. Your default password is: " + &password;
                
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
        
                                    let body = page::render_page(Some(ctx), &data, dir::APP_NAME.to_owned() + " | " + "Associate Account Created", dir::APP_NAME.to_owned(), content).unwrap();
        
                                    HttpResponse::new(http::StatusCode::OK)
                                        .set_body(Body::from(body))

                                },
                                Err(login::LoginEntryError::UsernameExists) =>  add_admin_page(data, req, "This email is associated with another account!"),
                                Err(e) =>  add_admin_page(data, req, &format!("Something went wrong: ensure that the email is unique: {}", e)),
                            }
                        } else {
                            add_admin_page(
                                data,
                                req,
                                "Invalid teacher details provided!",
                            )
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
