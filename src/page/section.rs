use std::sync::Arc;
use std::str::FromStr;
use std::io::Write;

use actix_web::{
    web,
    http,
    body::Body,
    HttpRequest,
    HttpResponse,
    Responder,
};

use actix_files::NamedFile;
use actix_multipart::Multipart;
use futures::{StreamExt, TryStreamExt};

use serde_json::json;

use crate::data::SharedData;

use crate::page;
use crate::dir;
use crate::org;
use crate::user;
use crate::util;
use crate::login;
use crate::auth;
use crate::section; 

use actix_web::{get, post};

pub fn choose_activities_page(data: &SharedData, ctx: auth::AuthContext, org_id: org::OrgKey, org: &org::Org, user_id: user::UserKey, user: &user::User, section_index: usize, section: &section::SectionInfo) -> HttpResponse {
    let mut activities: String = String::new();
        
    for (i, activity) in section.activities.iter().enumerate() {
        let desc: String = {
            match data.handlebars.render(&activity.activity_url, &()) {
                Ok(data) => data,
                Err(e) => format!("Failed to render: {}", e),
            }
        };

        activities += &data.handlebars.render("sections/activity_option", &json!({
            "index": i,
            "title": &activity.name,
            "subtitle": &activity.subtitle,
            "description": desc,
        })).unwrap();
    }

    let body: String = data.handlebars.render("sections/section_select", &json!({
        "section_name": &section.name,
        "back_url": dir::client_path(org_id, user_id),
        "section_image_url": section.image_url,
        "activities": activities,
        "select_activity_url": dir::client_path(org_id, user_id) + dir::SECTION_ROOT + "/" + &section_index.to_string() + dir::SELECT_ACTIVITY_PATH,
    })).unwrap();

    let header: String = page::path_header(&data, &[
        (dir::ORGS_PAGE.to_owned(), dir::ORGS_TITLE.to_owned()), 
        (dir::org_path(org_id), org.name.clone()),
        (dir::client_path(org_id, user_id), user.name())
    ]);

    let root: String = data.handlebars.render("client/client_root", &json!({
        "header": header,
        "body": body,
    })).unwrap();

    let body = page::render_page(Some(ctx), &data, dir::APP_NAME.to_owned() + " | " + "Pupil Dashboard", dir::APP_NAME.to_owned(), root).unwrap();

    HttpResponse::new(http::StatusCode::OK)
        .set_body(Body::from(body))
}

pub async fn section_page(data: &SharedData, ctx: auth::AuthContext, org_id: org::OrgKey, org: &org::Org, user_id: user::UserKey, user: &user::User, section_index: usize, section: &section::SectionInfo, section_id: section::SectionKey, section_instance: &section::Section) -> HttpResponse {
    let activity = &section.activities[section_instance.activity_index];

    let desc: String = {
        match data.handlebars.render(&activity.activity_url, &()) {
            Ok(data) => data,
            Err(e) => format!("Failed to render: {}", e),
        }
    };

    let mut files: String = String::new();

    let root_path: String = data.section_path(&section_id);
    let paths = web::block(|| std::fs::read_dir(root_path)).await.unwrap();
    
    let mut has_assets: bool = false;

    for path in paths {
        has_assets = true;
        let p = path.unwrap();
        let filename = p.file_name().to_owned().into_string().unwrap();
        let url = "/section/".to_owned() + &section_id.to_string() + "/asset/" + &filename;
        let media: String = {
            if p.path().extension() == Some(std::ffi::OsStr::new("png")) || p.path().extension() == Some(std::ffi::OsStr::new("jpg")) || p.path().extension() == Some(std::ffi::OsStr::new("jpeg")) {
                data.handlebars.render("sections/image_asset", &json!({
                    "asset_url": &url,
                })).unwrap()
        
            } else {
                String::new()
            }
        };
        files += &data.handlebars.render("sections/file_bubble", &json!({
            "filename": &filename,
            "download_url": url,
            "media": media,
        })).unwrap();
    }

    let files_info: String = {
        if !has_assets {
            "No files have been uploaded yet...".to_string()
        } else {
            String::new()
        }
    };

    let body: String = data.handlebars.render("sections/section", &json!({
        "section_name": &section.name,
        "back_url": dir::client_path(org_id, user_id),
        "section_image_url": section.image_url,
        "state": section_instance.state.to_string(),
        "state_class": section_instance.state.css_class(),
        "activity_name": &activity.name,
        "activity_subtitle": &activity.subtitle,
        "activity_description": desc,
        "plan": &section_instance.plan,
        "files": files,
        "files_info": files_info,
        "reflection": &section_instance.reflection,
        "upload_section_url": "/section/".to_owned() + &section_id.to_string() + "/upload",
        "select_activity_url": dir::client_path(org_id, user_id) + dir::SECTION_ROOT + "/" + &section_index.to_string() + dir::SELECT_ACTIVITY_PATH,
    })).unwrap();

    let header: String = page::path_header(&data, &[
        (dir::ORGS_PAGE.to_owned(), dir::ORGS_TITLE.to_owned()), 
        (dir::org_path(org_id), org.name.clone()),
        (dir::client_path(org_id, user_id), user.name())
    ]);

    let root: String = data.handlebars.render("client/client_root", &json!({
        "header": header,
        "body": body,
    })).unwrap();

    let body = page::render_page(Some(ctx), &data, dir::APP_NAME.to_owned() + " | " + "Pupil Dashboard", dir::APP_NAME.to_owned(), root).unwrap();

    HttpResponse::new(http::StatusCode::OK)
        .set_body(Body::from(body))
}


#[get("/org/{org}/client/{user}/section/{section}")]
pub async fn section_get(data: web::Data<Arc<SharedData>>, req: HttpRequest, path: web::Path<(String, String, usize)>) -> HttpResponse {
    if let Ok(org_id) = org::OrgKey::from_str(&path.0.0) {
        if let Ok(user_id) = user::UserKey::from_str(&path.0.1) {
            let section_index: usize = path.0.2;
            if section_index < 6 {
                let section = &data.sections[section_index];
                match data.authenticate_context_from_request(&req, true) {
                    Ok(Some(ctx)) => {
                        match data.user_db.fetch(&user_id) {
                            Ok(Some(user)) => {
                                if user.user_agent.is_client() {
                                    if ctx.user.user_agent.can_view_user(&user.user_agent) || ctx.user_id == user_id {
                                        match data.org_db.fetch(&org_id) {
                                            Ok(Some(org)) => {
                                                if let user::UserAgent::Client { sections, .. } = &user.user_agent {
                                                    match sections[section_index] {
                                                        Some(section_id) => {
                                                            match data.section_db.fetch(&section_id) {
                                                                Ok(Some(ref section_instance)) => section_page(&data, ctx, org_id, &org, user_id, &user, section_index, &section, section_id, section_instance).await,
                                                                Ok(None) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .set_body(Body::from("Section doesnt exist!")),
                                                                Err(e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .set_body(Body::from(format!("Error: {}", e))),
                                                            }

                                                        },
                                                        None => choose_activities_page(&data, ctx, org_id, &org, user_id, &user, section_index, &section)
                                                    }
                                                } else {
                                                    panic!("Urneachable code!!!");
                                                }
                                            },
                                            _ => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                                                .set_body(Body::from("Failed to fetch org!")),
                                        }
                                    } else {
                                        page::not_authorized_page(Some(ctx), &data)
                                    }
                                } else {
                                    HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                                                .set_body(Body::from("User is not a client!"))
                                }
                                
                            },
                            Ok(None) => HttpResponse::new(http::StatusCode::BAD_REQUEST)
                                .set_body(Body::from("Could not find user!")),
                            Err(e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                                .set_body(Body::from(format!("Error: {}", e))),
                        }
                    },
                    Ok(None) => page::redirect_to_login(&req),
    
                    Err(e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                        .set_body(Body::from(format!("Error: {}", e))),
                        
                } 
            } else {
                HttpResponse::new(http::StatusCode::BAD_REQUEST)
                    .set_body(Body::from("Invalid section index"))
            }
        } else {
            HttpResponse::new(http::StatusCode::BAD_REQUEST)
                .set_body(Body::from("Invalid user_id"))
        }
    } else {
        HttpResponse::new(http::StatusCode::BAD_REQUEST)
            .set_body(Body::from("Invalid org_id"))
    }
}

#[derive(serde::Deserialize)]
pub struct SelectSectionOptionForm {
    index: usize,
}

#[post("/org/{org}/client/{user}/section/{section}/select_activity")]
pub async fn select_activity_post(data: web::Data<Arc<SharedData>>, req: HttpRequest, form: web::Form<SelectSectionOptionForm>, path: web::Path<(String, String, usize)>) -> HttpResponse {
    if let Ok(org_id) = org::OrgKey::from_str(&path.0.0) {
        if let Ok(user_id) = user::UserKey::from_str(&path.0.1) {
            let section_index: usize = path.0.2;
            if section_index < 6 {
                let section = &data.sections[section_index];
                match data.authenticate_context_from_request(&req, true) {
                    Ok(Some(ctx)) => {
                        match data.user_db.fetch(&user_id) {
                            Ok(Some(mut user)) => {
                                if user.user_agent.is_client() {
                                    if ctx.user.user_agent.can_view_user(&user.user_agent) || ctx.user_id == user_id {
                                        match data.org_db.fetch(&org_id) {
                                            Ok(Some(org)) => {
                                                if form.index < section.activities.len() {
    
                                                    let section_instance = section::Section::new(form.index, user_id);
                                                    
                                                    match data.add_section(&section_instance) {
                                                        Ok(section_id) => {
                                                            if let user::UserAgent::Client { sections, .. } = &mut user.user_agent {
                                                                sections[section_index] = Some(section_id);
                                                                let _ = data.user_db.insert(&user_id, &user);

                                                                let mut r = HttpResponse::SeeOther();
                                                                r.header(http::header::LOCATION, dir::client_path(org_id, user_id) + dir::SECTION_ROOT + "/" + &section_index.to_string());
                                                                r.body("")
                                                            } else {
                                                                panic!("Unreachable code!!!");
                                                            }
                                                        },
                                                        Err(e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                                                            .set_body(Body::from(format!("Error: {}", e))),
                                                    }
                                                } else {
                                                    HttpResponse::new(http::StatusCode::BAD_REQUEST)
                                                        .set_body(Body::from("Invalid activity index!"))
                                                }       
                                            },
                                            _ => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                                                .set_body(Body::from("Failed to fetch org!")),
                                        }
                                    } else {
                                        page::not_authorized_page(Some(ctx), &data)
                                    }
                                } else {
                                    HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                                                .set_body(Body::from("User is not a client!"))
                                }
                                
                            },
                            Ok(None) => HttpResponse::new(http::StatusCode::BAD_REQUEST)
                                .set_body(Body::from("Could not find user!")),
                            Err(e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                                .set_body(Body::from(format!("Error: {}", e))),
                        }
                    },
                    Ok(None) => page::redirect_to_login(&req),
    
                    Err(e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                        .set_body(Body::from(format!("Error: {}", e))),
                        
                } 
            } else {
                HttpResponse::new(http::StatusCode::BAD_REQUEST)
                    .set_body(Body::from("Invalid section index"))
            }
        } else {
            HttpResponse::new(http::StatusCode::BAD_REQUEST)
                .set_body(Body::from("Invalid user_id"))
        }
    } else {
        HttpResponse::new(http::StatusCode::BAD_REQUEST)
            .set_body(Body::from("Invalid org_id"))
    }
}



#[post("/section/{section}/upload")]
async fn upload_section_post(data: web::Data<Arc<SharedData>>, req: HttpRequest, mut payload: Multipart, section: web::Path<String>) -> HttpResponse {
    if let Ok(section_id) = section::SectionKey::from_str(&section) {
            // iterate over multipart stream
        match data.authenticate_context_from_request(&req, true) {
            Ok(Some(ctx)) => {
                match data.section_db.fetch(&section_id) {
                    Ok(Some(mut section_instance)) => {
                        match data.user_db.fetch(&section_instance.user_id) {
                            Ok(Some(user)) => {
                                if ctx.user.user_agent.can_view_user(&user.user_agent) || ctx.user_id == section_instance.user_id {
                                    while let Ok(Some(mut field)) = payload.try_next().await {
                                        let content_type = field.content_disposition().unwrap();
                                        if let Some(fname) = content_type.get_filename() {
                                            if !fname.trim().is_empty() {
                                                let mut filename: String = sanitize_filename::sanitize(&fname);
                                                let path = std::path::Path::new(&filename).to_owned();
                                                let mut filepath = data.path_for_asset(&section_id, &filename);
                                
                                                let mut i: i32 = 0;
                                
                                                while std::path::Path::new(&filepath).exists() { 
                                                    filename = path.file_stem().or(Some(std::ffi::OsStr::new("file"))).unwrap().to_owned().into_string().unwrap() + &i.to_string() + "." + path.extension().or(Some(&std::ffi::OsString::new())).unwrap().to_str().unwrap();
                                                    filepath = data.path_for_asset(&section_id, &filename);
                                                    i += 1;
                                                }

                                                let prefix = std::path::Path::new(&filepath).parent().unwrap();
                                                std::fs::create_dir_all(prefix).unwrap();

                                                if let Ok(mut f) = web::block(|| std::fs::File::create(filepath)).await {
                                                    // Field in turn is stream of *Bytes* object
                                                    while let Some(chunk) = field.next().await {
                                                        let data = chunk.unwrap();
                                                        // filesystem operations are blocking, we have to use threadpool
                                                        f = web::block(move || f.write_all(&data).map(|_| f)).await.unwrap();
                                                    }
                                                } 
                                                
                                            }
                                        } else if let Some(name) = content_type.get_name() {
                                            let mut buffer: Vec<u8> = Vec::new();
                                            while let Some(chunk) = field.next().await {
                                                let data = chunk.unwrap();
                                                buffer.append(&mut data.to_vec());
                                            }
                                            let value = String::from_utf8(buffer).unwrap().trim().to_owned();
                                            match name {
                                                "plan" => section_instance.plan = value,
                                                "reflection" => section_instance.reflection = value,
                                                "delete" => {
                                                    if !value.trim().is_empty() {
                                                        let delete_filepath = data.path_for_asset(&section_id, &value);
                                                        let _ = web::block(move || std::fs::remove_file(&delete_filepath)).await;
                                                    }
                                                },
                                                _ => {},
                                            }
                                            
                                        }
                                    }
                                    let _ = data.section_db.insert(&section_id, &section_instance);
                                    let mut r = HttpResponse::SeeOther();        
                                    if let Some(referer) = req.headers().get("Referer") {
                                        r.header(http::header::LOCATION, referer.clone());
                                    }
                                    r.body("")
                                } else {
                                    page::not_authorized_page(Some(ctx), &data)
                                }
                            },
                            Ok(None) => HttpResponse::new(http::StatusCode::BAD_REQUEST)
                                .set_body(Body::from("Could not find user!")),
                            Err(e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                                .set_body(Body::from(format!("Error: {}", e))),
                        }
                        
                    },
                    Ok(None) => HttpResponse::new(http::StatusCode::BAD_REQUEST)
                        .set_body(Body::from("No matching section")),
        
                    Err(e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                        .set_body(Body::from(format!("Error: {}", e))),
                }
            },
            Ok(None) => page::redirect_to_login(&req),
    
            Err(e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                .set_body(Body::from(format!("Error: {}", e))),
        }
        
    } else {
        HttpResponse::new(http::StatusCode::BAD_REQUEST)
            .set_body(Body::from("Invalid section_id"))
    }
   
}



#[get("/section/{section}/asset/{asset}")]
async fn asset_get(data: web::Data<Arc<SharedData>>, req: HttpRequest, path: web::Path<(String, String)>) -> HttpResponse {
    if let Ok(section_id) = section::SectionKey::from_str(&path.0.0) {
            // iterate over multipart stream
        match data.authenticate_context_from_request(&req, true) {
            Ok(Some(ctx)) => {
                match data.section_db.fetch(&section_id) {
                    Ok(Some(section_instance)) => {
                        match data.user_db.fetch(&section_instance.user_id) {
                            Ok(Some(user)) => {
                                if ctx.user.user_agent.can_view_user(&user.user_agent) || ctx.user_id == section_instance.user_id {
                                    let filename = path.0.1;
                                    let filepath = data.path_for_asset(&section_id, &filename);
                                    if let Ok(file) = web::block(|| NamedFile::open(filepath)).await {
                                        file.into_response(&req).unwrap()
                                    } else {
                                        HttpResponse::new(http::StatusCode::BAD_REQUEST)
                                            .set_body(Body::from("Asset not found!"))
                                    }
                                } else {
                                    page::not_authorized_page(Some(ctx), &data)
                                }
                            },
                            Ok(None) => HttpResponse::new(http::StatusCode::BAD_REQUEST)
                                .set_body(Body::from("Could not find user!")),
                            Err(e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                                .set_body(Body::from(format!("Error: {}", e))),
                        }
                        
                    },
                    Ok(None) => HttpResponse::new(http::StatusCode::BAD_REQUEST)
                        .set_body(Body::from("No matching section")),
        
                    Err(e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                        .set_body(Body::from(format!("Error: {}", e))),
                }
            },
            Ok(None) => page::redirect_to_login(&req),
    
            Err(e) => HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                .set_body(Body::from(format!("Error: {}", e))),
        }
        
    } else {
        HttpResponse::new(http::StatusCode::BAD_REQUEST)
            .set_body(Body::from("Invalid section_id"))
    }
   
}