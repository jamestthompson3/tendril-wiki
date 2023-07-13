use crate::services::{create_jwt, MONTH};
use bytes::BufMut;
use futures::TryStreamExt;
use persistance::fs::{get_note_titles, read_note_cache};
use std::collections::HashMap;
use task_runners::runners::api_runner::{APIRunner, FileError};
use urlencoding::encode;
use warp::{
    filters::BoxedFilter,
    http::{header, HeaderValue, Response},
    hyper::{StatusCode, Uri},
    multipart::{self, Part},
    Filter, Reply,
};

use super::{
    filters::{with_auth, AuthError},
    MAX_BODY_SIZE,
};

pub struct APIRouter {}

#[allow(clippy::new_without_default)]
impl APIRouter {
    pub fn new() -> Self {
        Self {}
    }
    pub fn routes(&self) -> BoxedFilter<(impl Reply,)> {
        self.login()
            .or(self.logout())
            .or(self.styles())
            .or(self.img())
            .or(self.files())
            .or(self.titles())
            .or(self.mru())
            .or(self.json_page())
            .or(self.search_from_qs())
            .or(self.version())
            .boxed()
    }
    fn json_page(&self) -> BoxedFilter<(impl Reply,)> {
        warp::get()
            .and(with_auth())
            .and(warp::path!("api" / String).then(|note: String| async {
                let note = APIRunner::get_note(note).await;
                Response::builder()
                    .status(200)
                    .header(
                        header::CACHE_CONTROL,
                        "max-age=60,stale-while-revalidate=60",
                    )
                    .body(serde_json::to_string(&note).unwrap())
            }))
            .with(warp::cors().allow_any_origin())
            .boxed()
    }
    fn titles(&self) -> BoxedFilter<(impl Reply,)> {
        warp::get()
            .and(with_auth())
            .and(warp::path("titles"))
            .then(|| async move {
                let titles = get_note_titles().unwrap();
                warp::reply::json(&titles)
            })
            .boxed()
    }
    fn mru(&self) -> BoxedFilter<(impl Reply,)> {
        warp::get()
            .and(with_auth())
            .and(warp::path!("api" / "mru"))
            .then(|| async move {
                let recent = read_note_cache().await;
                let recent = recent.split('\n').collect::<Vec<&str>>();
                warp::reply::json(&recent)
            })
            .boxed()
    }
    fn version(&self) -> BoxedFilter<(impl Reply,)> {
        warp::get()
            .and(with_auth())
            .and(warp::path("version").then(|| async {
                let version = APIRunner::get_version();
                warp::reply::json(&version)
            }))
            .boxed()
    }
    fn img(&self) -> BoxedFilter<(impl Reply,)> {
        warp::post()
            .and(with_auth())
            .and(
                warp::path("files").and(
                    warp::body::content_length_limit(MAX_BODY_SIZE)
                        .and(warp::header::<String>("filename"))
                        .and(warp::body::bytes())
                        .then(|filename, bytes| async {
                            match APIRunner::process_image(filename, bytes).await {
                                Ok(()) => warp::reply::with_status("ok", StatusCode::OK),
                                Err(e) => {
                                    eprintln!("{}", e);
                                    warp::reply::with_status(
                                        "internal server error",
                                        StatusCode::INTERNAL_SERVER_ERROR,
                                    )
                                }
                            }
                        }),
                ),
            )
            .boxed()
    }
    fn files(&self) -> BoxedFilter<(impl Reply,)> {
        warp::post()
            .and(with_auth())
            .and(warp::body::content_length_limit(MAX_BODY_SIZE))
            .and(warp::filters::multipart::form())
            .then(|form_body: multipart::FormData| async {
                let parts: Vec<Part> = form_body
                    .try_collect()
                    .await
                    .map_err(|e| {
                        eprint!("Parsing form err: {}", e);
                        FileError::FormBodyRead
                    })
                    .unwrap();
                let file = parts.into_iter().find(|p| p.name() == "file").unwrap();
                let filename = String::from(file.filename().unwrap());
                let data = file
                    .stream()
                    .try_fold(Vec::new(), |mut vec, data| {
                        vec.put(data);
                        async { Ok(vec) }
                    })
                    .await
                    .unwrap_or_default();
                match APIRunner::file(filename, data).await {
                    Ok(()) => warp::redirect(Uri::from_static("/")),
                    Err(e) => {
                        eprintln!("{}", e);
                        let redir_url = format!("/error?msg={}", encode(&format!("{:?}", e)));
                        warp::redirect(redir_url.parse::<Uri>().unwrap())
                    }
                }
            })
            .boxed()
    }
    fn login(&self) -> BoxedFilter<(impl Reply,)> {
        warp::post()
            .and(warp::path("login"))
            .and(
                warp::body::content_length_limit(MAX_BODY_SIZE)
                    .and(warp::body::form())
                    .then(|form_body: HashMap<String, String>| async move {
                        let username = form_body.get("username").unwrap();
                        let pwd = form_body.get("password").unwrap();
                        match create_jwt(username, pwd) {
                            Ok(token) => Ok(Response::builder()
                                .status(StatusCode::MOVED_PERMANENTLY)
                                .header(header::LOCATION, HeaderValue::from_static("/"))
                                .header(
                                    header::SET_COOKIE,
                                    format!(
                                        "token={}; Secure; HttpOnly; Max-Age={}; Path=/",
                                        token, MONTH
                                    ),
                                )
                                .header(
                                    header::SET_COOKIE,
                                    format!("login=true; Secure; Max-Age={}; Path=/", MONTH),
                                )
                                .body("ok")),
                            Err(e) => {
                                let status: StatusCode;
                                let body: &str;
                                if let AuthError::JWTDecodeError = e {
                                    status = StatusCode::BAD_REQUEST;
                                    body = "Could not process request";
                                } else {
                                    status = StatusCode::FORBIDDEN;
                                    body = "Invalid username or password";
                                }
                                Ok(Response::builder()
                                    .status(status)
                                    .header(header::LOCATION, HeaderValue::from_static("/"))
                                    .body(body))
                            }
                        }
                    }),
            )
            .boxed()
    }
    fn logout(&self) -> BoxedFilter<(impl Reply,)> {
        warp::get()
            .and(warp::path("logout"))
            .then(|| async {
                Ok(Response::builder()
                    .status(StatusCode::MOVED_PERMANENTLY)
                    .header(header::LOCATION, HeaderValue::from_static("/"))
                    .header(
                        header::SET_COOKIE,
                        "token=; Secure; HttpOnly; Max-Age=0; Path=/",
                    )
                    .body("ok"))
            })
            .boxed()
    }
    fn search_from_qs(&self) -> BoxedFilter<(impl Reply,)> {
        warp::path("search")
            .and(warp::get())
            .and(with_auth())
            .and(warp::query::<HashMap<String, String>>())
            .then(|query_params: HashMap<String, String>| async move {
                let term = query_params.get("term").unwrap();
                let results_page = APIRunner::note_search(term.clone()).await;
                warp::reply::html(results_page)
            })
            .boxed()
    }
    fn styles(&self) -> BoxedFilter<(impl Reply,)> {
        warp::path("styles")
            .and(warp::post().and(with_auth()).and(
                warp::body::content_length_limit(MAX_BODY_SIZE).and(warp::body::form().then(
                    |form_body| async {
                        match APIRunner::update_styles(form_body).await {
                            Ok(()) => warp::redirect(Uri::from_static("/")),
                            Err(e) => {
                                eprintln!("{}", e);
                                let redir_url =
                                    format!("/error?msg={}", encode(&format!("{:?}", e)));
                                warp::redirect(redir_url.parse::<Uri>().unwrap())
                            }
                        }
                    },
                )),
            ))
            .boxed()
    }
}
