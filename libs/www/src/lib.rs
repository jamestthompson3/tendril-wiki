use ::build::{config::General, RefBuilder};
use build::{get_config_location, get_data_dir_location};
use std::{path::PathBuf, sync::Arc};
use tasks::normalize_wiki_location;
use warp::Filter;

pub mod controllers;
pub mod handlers;
pub mod services;

use crate::handlers::*;

pub async fn server(config: General, ref_builder: RefBuilder) {
    let (config_dir, _) = get_config_location();
    let user_stylesheet = config_dir.join("userstyles.css");
    let wiki_location = Arc::new(config.wiki_location);
    let media_location = Arc::new(normalize_wiki_location(&config.media_location));
    let index = index(config.user);
    let wiki = wiki(ref_builder.clone(), wiki_location.clone());
    let nested = nested_file(ref_builder.clone(), wiki_location.clone());
    let new_page = new_page();
    let search_page = search_page();
    let user_styles = serve_user_styles();
    let update_user_styles = update_user_styles();
    let handle_search = search_handler(wiki_location.clone());
    let static_files = warp::path("static")
        .and(warp::fs::dir(get_static_dir()))
        .or(warp::path("config").and(warp::fs::file(user_stylesheet)));
    let user_files = warp::path("files").and(warp::fs::dir(PathBuf::from(media_location.as_str())));
    let user_file_list = file_list(media_location.clone());
    let edit = edit_handler(ref_builder.clone(), wiki_location.clone());
    let delete = delete_page(ref_builder, wiki_location);
    let upload_page = upload_page();
    let help = help_page();
    let img_uploader = img_upload(media_location.clone());
    let file_uploader = file_upload(media_location);
    // Order matters!!
    let routes = user_file_list
        .or(user_files)
        .or(static_files)
        .or(nested)
        .or(img_uploader)
        .or(file_uploader)
        .or(delete)
        .or(upload_page)
        .or(help)
        .or(edit)
        .or(user_styles)
        .or(update_user_styles)
        .or(new_page)
        .or(search_page)
        .or(handle_search)
        .or(wiki)
        .or(index)
        .or(login())
        .recover(handle_rejection);
    let port: u16 = config.port;
    println!("┌──────────────────────────────────────────────┐");
    println!("│Starting web backend @ http://127.0.0.1:{}  │", port);
    println!("└──────────────────────────────────────────────┘");
    warp::serve(routes).run(([0, 0, 0, 0], port)).await;
}

#[cfg(debug_assertions)]
fn get_static_dir() -> PathBuf {
    PathBuf::from("static")
}

#[cfg(not(debug_assertions))]
fn get_static_dir() -> PathBuf {
    let data_dir = get_data_dir_location();
    data_dir.join("static")
}
