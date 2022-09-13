use bytes::{BufMut, Bytes};
use futures::TryStreamExt;
use render::{search_results_page::SearchResultsPage, Render};
use search_engine::{semantic_search, Indicies};
use std::{collections::HashMap, io, time::Instant};
use thiserror::Error;
use urlencoding::encode;

use persistance::fs::{utils::get_config_location, write_media};
use warp::{
    filters::BoxedFilter,
    http::{header, HeaderValue, Response},
    hyper::{StatusCode, Uri},
    multipart::{self, Part},
    Filter, Reply,
};

use crate::services::{create_jwt, MONTH};

use super::{
    filters::{with_auth, AuthError},
    MAX_BODY_SIZE,
};

pub struct APIRouter {}

struct Runner {}

#[derive(Error, Debug)]
enum FileError {
    #[error("Could not parse form body")]
    FormBodyRead,
    #[error("Could not write media")]
    FileWrite,
}

impl Runner {
    pub async fn file(form: multipart::FormData) -> Result<(), FileError> {
        let parts: Vec<Part> = form.try_collect().await.map_err(|e| {
            eprint!("Parsing form err: {}", e);
            FileError::FormBodyRead
        })?;
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
        match write_media(&filename, &data).await {
            Ok(()) => Ok(()),
            Err(e) => {
                eprintln!("Could not write media: {}", e);
                Err(FileError::FileWrite)
            }
        }
    }

    pub async fn process_image(filename: String, bytes: Bytes) -> Result<(), io::Error> {
        write_media(&filename, bytes.as_ref()).await
    }

    pub async fn note_search(term: String) -> String {
        let now = Instant::now();
        let found_pages = semantic_search(&term).await;
        println!(
            "Search Time [{:?}]  Search Results [{}]",
            now.elapsed(),
            found_pages.len()
        );
        let ctx = SearchResultsPage { pages: found_pages };
        ctx.render().await
    }

    pub async fn dump_search_index() -> Indicies {
        search_engine::dump_search_index().await
    }

    pub async fn update_styles(form_body: HashMap<String, String>) -> Result<(), io::Error> {
        let (path, _) = get_config_location();
        let style_location = path.join("userstyles.css");
        let body = form_body.get("body").unwrap();
        tokio::fs::write(style_location, body).await
    }
}

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
            .or(self.search_from_qs())
            .or(self.search_indicies())
            .boxed()
    }
    fn search_indicies(&self) -> BoxedFilter<(impl Reply,)> {
        warp::get()
            .and(with_auth())
            .and(warp::path("search-idx").then(|| async {
                let indicies = Runner::dump_search_index().await;
                warp::reply::json(&indicies)
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
                            match Runner::process_image(filename, bytes).await {
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
            .then(|form_body| async {
                match Runner::file(form_body).await {
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
                let results_page = Runner::note_search(term.clone()).await;
                warp::reply::html(results_page)
            })
            .boxed()
    }
    fn styles(&self) -> BoxedFilter<(impl Reply,)> {
        warp::path("styles")
            .and(warp::post().and(with_auth()).and(
                warp::body::content_length_limit(MAX_BODY_SIZE).and(warp::body::form().then(
                    |form_body| async {
                        match Runner::update_styles(form_body).await {
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
