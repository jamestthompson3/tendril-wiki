pub mod api;
pub mod bookmarks_page;
pub mod filters;
pub mod static_files;
pub mod static_pages;
pub mod tasks_page;
pub mod wiki_page;

pub use self::api::*;
pub use self::filters::*;
pub use self::static_files::*;
pub use self::static_pages::*;
pub use self::tasks_page::*;
pub use self::wiki_page::*;

use std::convert::Infallible;
use std::sync::Arc;

use render::{login_page::LoginPage, Render};
use task_runners::JobQueue;
use warp::body::BodyDeserializeError;
use warp::{http::StatusCode, Rejection, Reply};

// 40MB file limit
pub const MAX_BODY_SIZE: u64 = 40_000_000;

pub(crate) type QueueHandle = Arc<JobQueue>;

pub async fn handle_rejection(err: Rejection) -> std::result::Result<impl Reply, Infallible> {
    let (code, message) = if err.is_not_found() {
        (StatusCode::NOT_FOUND, "Not Found".to_string())
    } else if let Some(e) = err.find::<AuthError>() {
        match e {
            AuthError::AuthNotPresent => (StatusCode::UNAUTHORIZED, e.to_string()),
            AuthError::BadCredentials => (StatusCode::FORBIDDEN, e.to_string()),
            _ => (StatusCode::BAD_REQUEST, e.to_string()),
        }
    } else if err.find::<BodyDeserializeError>().is_some() {
        eprintln!("Serialization error: {:?}", err);
        (StatusCode::BAD_REQUEST, "Invalid body".to_string())
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
            .body(ctx.render().await)
            .unwrap();

        return Ok(response);
    }

    let response = warp::http::Response::builder()
        .status(code)
        .body(message)
        .unwrap();

    Ok(response)
}
