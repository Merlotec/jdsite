#![feature(proc_macro_hygiene)]
use actix_web::{App, HttpRequest, HttpResponse, HttpServer, Responder, Result, body::Body, error::ErrorNotFound, body::PrivateHelper, get, http, http::{StatusCode, header::{ContentDisposition, DispositionParam, DispositionType}}, web};
use actix_files::{NamedFile, Files};
use std::path::PathBuf;
use std::sync::{Mutex, Arc};

#[macro_use]
pub mod db;

pub mod dir;

pub mod form;

pub mod data;
pub mod page;

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

async fn root(req: HttpRequest) -> impl Responder {
    let mut r = HttpResponse::PermanentRedirect();
    r.header(http::header::LOCATION, "/home.html");
    
    r.await
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    println!("Initialising...");

    let data: Arc<SharedData> = Arc::new(SharedData::load_from_disk("root".to_string()).expect("Failed to load database data!"));

    HttpServer::new(move || { 
        App::new()
            .data(data.clone())
            .route("/", web::get().to(root))
            .route("/{filename:.*}", web::get().to(static_file))
            .service(Files::new("/", "static").index_file("index.html"))
    })
        .bind("0.0.0.0:80")?
        .run()
        .await
}