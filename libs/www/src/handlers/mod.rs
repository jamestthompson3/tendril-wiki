pub mod api;
pub mod filters;
pub mod sinks;
pub mod static_files;
pub mod static_pages;
pub mod wiki_page;

pub use self::api::*;
pub use self::filters::*;
pub use self::sinks::*;
pub use self::static_files::*;
pub use self::static_pages::*;
pub use self::wiki_page::*;

use std::convert::Infallible;

use render::{login_page::LoginPage, Render};
use warp::{http::StatusCode, Rejection, Reply};

// 40MB file limit
pub const MAX_BODY_SIZE: u64 = 40_000_000;

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
            .body(ctx.render())
            .unwrap();

        return Ok(response);
    }

    let response = warp::http::Response::builder()
        .status(code)
        .body(message)
        .unwrap();

    Ok(response)
}
