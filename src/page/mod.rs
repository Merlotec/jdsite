pub mod login;

use crate::{
    auth::AuthContext,
    data::SharedData,
};

use std::collections::HashMap;

use actix_web::HttpRequest;

pub fn render_page(ctx: Option<AuthContext>, data: &SharedData, title: String, heading: String, body: String) -> Result<String, handlebars::RenderError> {
    let nav_items = data.nav_items_for_context(ctx);

    let mut nav_string: String = String::new();

    for (url, title) in nav_items {
        let mut nav_map: HashMap<String, String> = HashMap::new();
        nav_map.insert("nav_url".to_string(), url);
        nav_string += &data.handlebars.render("nav_item", &nav_map)?;
    }

    let mut map: HashMap<String, String> = HashMap::new();
    map.insert("page_title".to_string(), title);
    map.insert("page_nav".to_string(), nav_string);

    Ok(data.handlebars.render("page", &map)?)
}