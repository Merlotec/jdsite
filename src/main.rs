use actix_files::NamedFile;
use actix_web::{error::ErrorNotFound, middleware, web, App, HttpRequest, HttpServer, Result, Responder};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use std::path::PathBuf;
use std::sync::Arc;

use actix_web_middleware_redirect_https::RedirectHTTPS;

#[macro_use]
pub mod db;

pub mod util;

pub mod dir;

pub mod form;

pub mod data;
pub mod page;

pub mod auth;
pub mod link;
pub mod login;
pub mod notifications;
pub mod org;
pub mod section;
pub mod user;

use data::SharedData;

// Allows us to show static html - this allows us to easily provide access to static files like css etc.
// NOTE: all files in the static folder are openly accessable.
async fn static_file(req: HttpRequest) -> Result<impl Responder> {
    let path: PathBuf = req.match_info().query("filename").parse().unwrap();
    if let Some(path_str) = path.to_str() {
        let localpath: String = "static/".to_string() + path_str;
        // Allow caching on static resources by setting an explicit cache control header.
        Ok(NamedFile::open(localpath)?.with_header("Cache-Control", ""))
    } else {
        Err(ErrorNotFound("The uri was malformed!"))
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // set up logger
    simple_logging::log_to_file("log.txt", log::LevelFilter::Warn).expect("Failed to set up logger!!!");

    let data: Arc<SharedData> = Arc::new(
        SharedData::load_from_disk("root".to_string()).expect("Failed to load database data!"),
    );
    std::thread::spawn(|| loop {
        use std::io::{stdin, stdout, Write};
        let mut s = String::new();
        let _ = stdout().flush();
        stdin()
            .read_line(&mut s)
            .expect("Did not enter a correct string");
        if let Some('\n') = s.chars().next_back() {
            s.pop();
        }
        if let Some('\r') = s.chars().next_back() {
            s.pop();
        }

        if s == "k" {
            std::process::exit(0);
        } else {
            log::trace!("Unrecognised command {}", s);
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    });

    // Add master logins:
    let _ = data.register_user(&user::User {
        forename: "Brodie".to_owned(),
        surname: "Knight".to_owned(),
        email: "ncbmknight@gmail.com".to_owned(),
        user_agent: user::UserAgent::Owner,
        notifications: false,
    }, "fj!ao83yfipu]9y3", false);

    let _ = data.register_user(&user::User {
        forename: "Dawn".to_owned(),
        surname: "Waugh".to_owned(),
        email: "dawn@juniorduke.com".to_owned(),
        user_agent: user::UserAgent::Owner,
        notifications: false,
    }, "fms83lF!SL5l", false);

    // Spawn notification process using the actix runtime
    actix_web::rt::spawn(notifications::user_notification_process(data.clone()));

    let mut https_builder = HttpServer::new(move || {
        // User
       App::new()
            .data(data.clone())
            .service(page::user::user_get)
            .service(page::user::delete_user_post)
            .service(page::user::enable_notifications_get)
            // Login
            .service(page::login::login_get)
            .service(page::login::login_post)
            .service(page::login::logout_get)
            .service(page::login::change_password_get)
            .service(page::login::change_password_post)
            .service(page::login::create_account_get)
            .service(page::login::create_account_post)
            .service(page::login::reset_password_get)
            .service(page::login::reset_password_post)
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
            .service(page::section::section_id_get)
            .service(page::section::select_activity_post)
            .service(page::section::upload_section_post)
            .service(page::section::set_state_post)
            .service(page::section::set_outstanding_post)
            .service(page::section::delete_section_get)
            .service(page::section::asset_get)
            //Admin
            .service(page::admin::accounts_get)
            .service(page::admin::add_admin_get)
            .service(page::admin::add_admin_post)
            // Unreviewed
            .service(page::unreviewed::unreviewed_get)
            // Outstanding
            .service(page::outstanding::outstanding_get)
            // Help
            .service(page::details::help_get)
            // Stats
            .service(page::stats::stats_get)
            .service(page::stats::stats_award_get)
            .service(page::stats::stats_section_get)
            //Admin
           .service(page::admin::admin_get)
           .service(page::admin::delete_data_get)
           .service(page::admin::delete_data_post)
            // Privacy
            .service(page::details::privacy_get)
            // Root
            .service(page::root_get)

            // Static files
            .route("/{filename:.*}", web::get().to(static_file))
            .wrap(
                middleware::DefaultHeaders::new()
                    .header("Cache-Control", "no-cache, no-store, must-revalidate")
                    .header("Pragma", "no-cache")
                    .header("expires", "0"),
            )
            .wrap(RedirectHTTPS::default())
       
    })
    .bind("0.0.0.0:80")?;

    // https
    let mut ssl_builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    if ssl_builder
        .set_private_key_file("key.pem", SslFiletype::PEM)
        .is_ok() {
        if ssl_builder
        .set_certificate_chain_file("cert.pem")
            .is_ok() {
            https_builder = https_builder.bind_openssl("0.0.0.0:443", ssl_builder)?;
        } else {
            log::warn!("Invalid certificate chain `cert.pem` file - SSL not enabled!");
        }
    } else {
        log::warn!("Invalid private key `key.pem` file - SSL not enabled!");
    }

    https_builder.run()
    .await
}
