#[cfg(not(debug_assertions))]
use ::persistance::fs::utils::get_data_dir_location;

use persistance::fs::{config::General, utils::normalize_wiki_location};
use render::GlobalBacklinks;
use std::{path::PathBuf, sync::Arc};
use tasks::JobQueue;
use warp::Filter;

#[macro_use]
extern crate lazy_static;

pub mod handlers;
pub mod services;

use crate::handlers::*;

pub(crate) type RefHubParts = (GlobalBacklinks, Arc<JobQueue>);

pub async fn server(config: General, parts: RefHubParts) {
    let media_location = Arc::new(normalize_wiki_location(&config.media_location));
    let static_page_router = StaticPageRouter::new(
        Arc::new(config.user),
        media_location.clone(),
        Arc::new(config.host),
    );
    let wiki_router = WikiPageRouter::new(parts.clone());

    let task_router = TaskPageRouter::new();
    let static_files_router = StaticFileRouter::new(media_location.clone());
    let api_router = APIRouter::new();
    let bookmark_router = bookmarks_page::BookmarkPageRouter::new(parts.1.clone());
    pretty_env_logger::init();
    // Order matters!!
    let log = warp::log("toplevel");
    let routes = warp::any()
        .and(
            static_files_router
                .routes()
                .or(static_page_router.routes())
                .or(bookmark_router.routes())
                .or(api_router.routes())
                .or(task_router.routes())
                .or(wiki_router.routes())
                .or(static_page_router.index())
                .recover(handle_rejection),
        )
        .with(log);
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
