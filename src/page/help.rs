
use actix_web::{body::Body, http, web, HttpRequest, HttpResponse, get};
use std::sync::Arc;


use crate::data::SharedData;

use crate::dir;
use crate::page;

#[get("/help")]
pub async fn help_get(
    data: web::Data<Arc<SharedData>>,
    req: HttpRequest,
) -> HttpResponse {

    match data.authenticate_context_from_request(&req, true) {
        Ok(ctx) => {
            let org_page = data
            .handlebars
            .render(
                "shared/help",
                &(),
            )
            .unwrap();

            let body = page::render_page(
                ctx,
                &data,
                dir::APP_NAME.to_owned() + " | " + "Help",
                dir::APP_NAME.to_owned(),
                org_page,
            )
            .unwrap();

            HttpResponse::new(http::StatusCode::OK).set_body(Body::from(body))
        },
        Err(e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
            .set_body(Body::from(format!("Error: {}", e))),
    }
}
