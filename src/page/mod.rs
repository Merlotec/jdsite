pub mod login;
pub mod orgs;

use crate::{
    auth::AuthContext,
    data::SharedData,
};

use std::collections::HashMap;


pub fn render_page(ctx: Option<AuthContext>, data: &SharedData, title: String, heading: String, body: String) -> Result<String, handlebars::RenderError> {
    let nav_items = data.nav_items_for_context(ctx.clone());

    let mut nav_string: String = String::new();

    for (url, title) in nav_items {
        let mut nav_map: HashMap<String, String> = HashMap::new();
        nav_map.insert("nav_url".to_owned(), url);
        nav_map.insert("nav_title".to_owned(), title);
        nav_string += &data.handlebars.render("nav_item", &nav_map)?;
    }

    let mut map: HashMap<String, String> = HashMap::new();

    let (user_string, user_class): (String, String) = {
        match ctx {
            Some(ctx) => {
                (ctx.user.forename.to_owned() + " " +  &ctx.user.surname, "user-string".to_owned())
            },
            None => ("Not Logged In".to_owned(), "no-user-string".to_owned()),
        }
    };
    
    map.insert("page_title".to_owned(), title);
    map.insert("page_heading".to_owned(), heading);
    map.insert("page_user".to_owned(), user_string);
    map.insert("page_user_class".to_owned(), user_class);
    map.insert("page_nav".to_owned(), nav_string);
    map.insert("page_body".to_owned(), body);

    Ok(data.handlebars.render("page", &map)?)
}