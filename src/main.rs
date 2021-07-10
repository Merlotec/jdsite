#![feature(proc_macro_hygiene)]
use actix_web::{App, HttpRequest, HttpResponse, HttpServer, Responder, Result,  error::ErrorNotFound, http, web};
use actix_files::{NamedFile, Files};
use std::path::PathBuf;
use std::sync::Arc;

#[macro_use]
pub mod db;

pub mod util;

pub mod dir;

pub mod form;

pub mod data;
pub mod page;

pub mod link;
pub mod login;
pub mod auth;
pub mod user;
pub mod org;
pub mod section;

use data::SharedData;

// Allows us to show static html - this allows us to easily provide access to static files like css etc.
// NOTE: all files in the static folder are openly accessable.
async fn static_file(req: HttpRequest) -> Result<NamedFile> {
    let path: PathBuf = req.match_info().query("filename").parse().unwrap();
    if let Some(path_str) = path.to_str() {
        let localpath: String = "static/".to_string() + path_str;
        Ok(NamedFile::open(localpath)?)
    } else {
        Err(ErrorNotFound("The uri was malformed!"))
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let data: Arc<SharedData> = Arc::new(SharedData::load_from_disk("root".to_string()).expect("Failed to load database data!"));
    
    
    /*
    data.register_user(&user::User {
        email: "ncbmknight@gmail.com".to_owned(),
        forename: "Brodie".to_owned(),
        surname: "Knight".to_owned(),
        user_agent: user::UserAgent::Owner,
    }, "Nemisite", false);
    */

    //println!("{}", data.link_manager.create_link(link::Link::ChangePassword(data.login_db.db().fetch("ncbmknight@gmail.com").unwrap().unwrap().user_id), std::time::Duration::from_secs(1000)).unwrap().to_string());

    std::thread::spawn(|| {
        loop {
            use std::io::{stdin,stdout,Write};
            let mut s = String::new();
            let _ = stdout().flush();
            stdin().read_line(&mut s).expect("Did not enter a correct string");
            if let Some('\n')=s.chars().next_back() {
                s.pop();
            }
            if let Some('\r')=s.chars().next_back() {
                s.pop();
            }
            
            if s == "k" {
                std::process::exit(0);
            } else {
                println!("Unrecognised command {}", s);
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    });

    HttpServer::new(move || { 
        App::new()
            .data(data.clone())
            //User
            .service(page::user::user_get)
            .service(page::user::delete_user_post)
            // Login
            .service(page::login::login_get)
            .service(page::login::login_post)
            .service(page::login::logout_get)
            .service(page::login::change_password_get)
            .service(page::login::change_password_post)
            // Org
            .service(page::orgs::org_get)
            .service(page::orgs::orgs_get)
            .service(page::orgs::add_org_post)
            .service(page::orgs::delete_org_post)
            .service(page::orgs::assign_admin_post)
            .service(page::orgs::add_credits_post)
            // Clients
            .service(page::clients::clients_get)
            .service(page::clients::add_client_get)
            .service(page::clients::add_client_post)
            .service(page::clients::client_dashboard_get)
            // Associates
            .service(page::associates::associates_get)
            .service(page::associates::add_associate_get)
            .service(page::associates::add_associate_post)
            // Sections
            .service(page::section::section_get)
            .service(page::section::select_activity_post)
            .service(page::section::upload_section_post)
            .service(page::section::asset_get)
            // Root
            .service(page::root_get)
            // Static files
            .route("/{filename:.*}", web::get().to(static_file))
            //.service(Files::new("/", "static").index_file("index.html"))
            
    })
        .bind("0.0.0.0:80")?
        .run()
        .await
}