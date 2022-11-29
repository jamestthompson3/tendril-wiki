#[cfg(not(debug_assertions))]
use ::persistance::fs::utils::get_data_dir_location;
use build::Titles;

use persistance::fs::{config::General, utils::normalize_wiki_location};
use wikitext::GlobalBacklinks;
use std::{path::PathBuf, sync::Arc};
use task_runners::JobQueue;
use warp::Filter;

pub mod handlers;
pub mod services;

use crate::handlers::*;

pub(crate) type RefHubParts = (GlobalBacklinks, Titles, Arc<JobQueue>);

pub async fn server(config: General, parts: RefHubParts) {
    let media_location = Arc::new(normalize_wiki_location(&config.media_location));
    let cloned = parts.clone();
    let static_page_router = StaticPageRouter::new(
        Arc::new(config.user),
        media_location.clone(),
        Arc::new(config.host),
        cloned.0,
        cloned.1
    );
    let wiki_router = WikiPageRouter::new(parts.clone());

    let task_router = TaskPageRouter::new();
    let static_files_router = StaticFileRouter::new(media_location.clone());
    let api_router = APIRouter::new(parts.1.clone());
    let bookmark_router = bookmark_handler::BookmarkPageRouter::new(parts.2.clone());
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
                .recover(handle_rejection)
                .boxed(),
        )
        .with(log)
        .boxed();
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
