use std::sync::Arc;

use actix_web::{
    web,
    http,
    body::Body,
    HttpRequest,
    HttpResponse,
    Result,
    dev::UrlEncoded,
};

use crate::data::SharedData;

use crate::page;
use crate::dir;
use crate::auth::AuthContext;

use actix_web::{post, get};

#[derive(serde::Deserialize)]
pub struct LoginQuery {
    msg: Option<String>,
}

#[get("/login")]
async fn login_get(data: web::Data<Arc<SharedData>>, req: HttpRequest) -> HttpResponse {
    match data.authenticate_context_from_request(&req) {
        Ok(ctx) => {
            login_template(ctx, &data, String::new())
        },
        Err(e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                .set_body(Body::from(format!("Error: {}", e))),
    }
}

pub fn login_template(ctx: Option<AuthContext>, data: &SharedData, msg: String) -> HttpResponse {
    let login_body: String = data.handlebars.render("login", &()).unwrap();

    match page::render_page(ctx, &data, dir::APP_NAME.to_string() + " Login", "Login".to_string(), login_body) {
        Ok(body) => HttpResponse::new(http::StatusCode::OK)
            .set_body(Body::from(body)),

        Err(e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
            .set_body(Body::from(format!("Error: {}", e))),
    }
}


#[derive(serde::Deserialize)]
pub struct LoginForm {
    username: String,
    password: String,
}

#[post("/login")]
async fn login_post(data: web::Data<Arc<SharedData>>, req: HttpRequest, form: web::Form<LoginForm>) -> HttpResponse {
    match data.authenticate_context_from_request(&req) {
        Ok(ctx) => {
            if !form.username.is_empty() {
                match data.login(&form.username, &form.password, std::time::Duration::from_secs(60 * 60)) {
                    Ok(ctx) => {
                        let mut r = HttpResponse::SeeOther();
                        r.header(http::header::LOCATION, "/download/".to_string());
                        r.body("")
                    },
                    Err(e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                        .set_body(Body::from(format!("Error: {}", e))),
                }
            } else {
                login_template(ctx, &data, "".to_string())
            }
        },
        Err(e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                .set_body(Body::from(format!("Error: {}", e))),
    }
}