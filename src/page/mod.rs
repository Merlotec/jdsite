pub mod associates;
pub mod clients;
pub mod login;
pub mod orgs;
pub mod outstanding;
pub mod section;
pub mod unreviewed;
pub mod user;
pub mod help;
pub mod admin;

use std::sync::Arc;

use actix_web::{body::Body, cookie::Cookie, get, http, web, HttpRequest, HttpResponse};
use serde_json::json;

use crate::{auth::AuthContext, data::SharedData, dir, org, user::Privilege};

use std::collections::HashMap;

pub fn render_page(
    ctx: Option<AuthContext>,
    data: &SharedData,
    title: String,
    heading: String,
    body: String,
) -> Result<String, handlebars::RenderError> {
    let nav_items = data.nav_items_for_context(ctx.clone());

    let mut nav_string: String = String::new();

    for (url, title) in nav_items {
        let mut nav_map: HashMap<String, String> = HashMap::new();
        nav_map.insert("nav_url".to_owned(), url);
        nav_map.insert("nav_title".to_owned(), title);
        nav_string += &data.handlebars.render("shared/nav_item", &nav_map)?;
    }

    let (user_string, user_class, user_link, auth_link, auth_action): (
        String,
        String,
        String,
        String,
        String,
    ) = {
        match ctx {
            Some(ctx) => (
                ctx.user.forename.to_owned() + " " + &ctx.user.surname,
                "user-string".to_owned(),
                dir::user_path(ctx.user_id),
                dir::LOGOUT_PATH.to_owned(),
                "Logout".to_owned(),
            ),
            None => (
                "Not Logged In".to_owned(),
                "no-user-string".to_owned(),
                "".to_owned(),
                dir::LOGIN_PAGE.to_owned(),
                "Login".to_owned(),
            ),
        }
    };

    Ok(data.handlebars.render(
        "shared/page",
        &json!({
            "page_title": title,
            "page_heading": heading,
            "page_user": user_string,
            "page_user_link": user_link,
            "page_user_class": user_class,
            "page_auth_link": auth_link,
            "page_auth_action": auth_action,
            "page_nav": nav_string,
            "page_body": body,
        }),
    )?)
}

pub fn org_nav(
    ctx: &AuthContext,
    data: &SharedData,
    org_id: org::OrgKey,
    org: &org::Org,
    path: String,
) -> String {
    let org_items = ctx.org_items(org_id, org);

    let mut org_nav: String = String::new();

    for (url, title) in org_items {
        let nav_class: &str = {
            if url == path {
                "org-nav-item-selected"
            } else {
                "org-nav-item"
            }
        };

        org_nav += &data
            .handlebars
            .render(
                "org/org_nav_item",
                &json!({
                    "nav_title": title,
                    "nav_url": url,
                    "nav_item_class": nav_class,
                }),
            )
            .unwrap()
    }

    org_nav
}

pub fn path_header(
    data: &SharedData,
    user_privilage: &Privilege,
    items: &[(String, String, Privilege)],
) -> String {
    let mut header: String = String::new();

    let mut is_first = true;

    for (url, title, privilege) in items.iter() {
        if user_privilage.magnitude() >= privilege.magnitude() {
            if !is_first {
                header += " > ";
            }

            header += &data
                .handlebars
                .render(
                    "shared/header_item",
                    &json!({
                        "text": title,
                        "url": url,
                    }),
                )
                .unwrap();

            is_first = false;
        }
    }

    return header;
}

pub fn not_authorized_page(ctx: Option<AuthContext>, data: &SharedData) -> HttpResponse {
    let page_body: String = data
        .handlebars
        .render("shared/not_authorized", &())
        .unwrap();

    let body = render_page(
        ctx,
        data,
        dir::APP_NAME.to_owned() + " | Not Authorised",
        dir::APP_NAME.to_owned(),
        page_body,
    )
    .unwrap();

    HttpResponse::new(http::StatusCode::UNAUTHORIZED).set_body(Body::from(body))
}

pub fn redirect_to_login(req: &HttpRequest) -> HttpResponse {
    let mut r = HttpResponse::SeeOther();
    r.cookie(Cookie::new(
        dir::LOGIN_REDIRECT_COOKIE,
        req.uri().to_string(),
    ));
    r.header(http::header::LOCATION, dir::LOGIN_PAGE);
    r.body("")
}

#[get("/")]
pub async fn root_get(data: web::Data<Arc<SharedData>>, req: HttpRequest) -> HttpResponse {
    match data.authenticate_context_from_request(&req, true) {
        Ok(Some(ctx)) => {
            let mut r = HttpResponse::SeeOther();
            r.header(http::header::LOCATION, ctx.root_page());
            r.body("")
        }
        Ok(None) => redirect_to_login(&req),

        Err(e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
            .set_body(Body::from(format!("Error: {}", e))),
    }
}
