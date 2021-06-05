pub mod filters;
pub mod static_pages;
pub mod user_styles;
pub mod wiki_page;

pub use self::filters::*;
pub use self::static_pages::*;
pub use self::user_styles::*;
pub use self::wiki_page::*;

use build::RefBuilder;
use markdown::parsers::{LoginPage, NewPage};
use sailfish::TemplateOnce;
use std::{collections::HashMap, convert::Infallible, sync::Arc};

use warp::{http::StatusCode, Filter, Rejection, Reply};

use crate::controllers::{self, *};

// 40MB file limit
pub const MAX_BODY_SIZE: u64 = 40_000_000;

pub fn login() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::post().and(warp::path("login")).and(
        warp::body::content_length_limit(MAX_BODY_SIZE)
            .and(warp::body::form())
            .and_then(controllers::authorize),
    )
}

pub fn new_page() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get().and(with_auth()).and(
        warp::path("new")
            .and(warp::query::<HashMap<String, String>>())
            .map(|query_params: HashMap<String, String>| {
                let ctx = NewPage {
                    title: None,
                    linkto: query_params.get("linkto"),
                };
                warp::reply::html(ctx.render_once().unwrap())
            }),
    )
}

pub fn delete_page(
    ref_builder: RefBuilder,
    location: Arc<String>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::post()
        .and(with_auth())
        .and(warp::path("delete"))
        .and(with_refs(ref_builder))
        .and(with_location(location))
        .and(warp::body::content_length_limit(MAX_BODY_SIZE))
        .and(warp::body::form())
        .and_then(delete)
}

pub fn search_handler(
    location: Arc<String>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::post().and(with_auth()).and(
        warp::path("search").and(
            warp::body::content_length_limit(MAX_BODY_SIZE)
                .and(warp::body::form())
                .and(with_location(location))
                .and_then(note_search),
        ),
    )
}

pub fn img_upload(
    media_location: Arc<String>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::post().and(with_auth()).and(
        warp::path("files").and(
            warp::body::content_length_limit(MAX_BODY_SIZE)
                .and(warp::header::<String>("filename"))
                .and(warp::body::bytes())
                .and(with_location(media_location))
                .and_then(image),
        ),
    )
}

pub fn file_upload(
    media_location: Arc<String>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::post()
        .and(with_auth())
        .and(warp::body::content_length_limit(MAX_BODY_SIZE))
        .and(warp::filters::multipart::form())
        .and(with_location(media_location))
        .and_then(file)
}

pub fn edit_handler(
    ref_builder: RefBuilder,
    location: Arc<String>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::post().and(with_auth()).and(
        warp::path("edit").and(
            warp::body::content_length_limit(MAX_BODY_SIZE)
                .and(warp::body::form())
                .and(with_location(location))
                .and(with_refs(ref_builder))
                .and_then(edit),
        ),
    )
}

pub async fn handle_rejection(err: Rejection) -> std::result::Result<impl Reply, Infallible> {
    let (code, message) = if err.is_not_found() {
        (StatusCode::NOT_FOUND, "Not Found".to_string())
    } else if let Some(e) = err.find::<AuthError>() {
        match e {
            AuthError::AuthNotPresent => (StatusCode::UNAUTHORIZED, e.to_string()),
            AuthError::BadCredentials => (StatusCode::FORBIDDEN, e.to_string()),
            _ => (StatusCode::BAD_REQUEST, e.to_string()),
        }
    } else if err.find::<warp::reject::MethodNotAllowed>().is_some() {
        (
            StatusCode::METHOD_NOT_ALLOWED,
            "Method Not Allowed".to_string(),
        )
    } else {
        eprintln!("unhandled error: {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal Server Error".to_string(),
        )
    };

    // Redirect to users to the login page if not authenticated.
    if code == StatusCode::UNAUTHORIZED {
        let ctx = LoginPage {};
        let response = warp::http::Response::builder()
            .status(StatusCode::OK)
            .body(ctx.render_once().unwrap())
            .unwrap();

        return Ok(response);
    }

    let response = warp::http::Response::builder()
        .status(code)
        .body(message)
        .unwrap();

    Ok(response)
}
