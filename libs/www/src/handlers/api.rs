use bytes::{BufMut, Bytes};
use futures::TryStreamExt;
use render::{search_results_page::SearchResultsPage, Render};
use search_engine::{semantic_search, Indicies};
use std::{collections::HashMap, io, sync::Arc, time::Instant};
use thiserror::Error;

use persistance::fs::{utils::get_config_location, write_media};
use warp::{
    http::{header, HeaderValue, Response},
    hyper::{StatusCode, Uri},
    multipart::{self, Part},
    Filter, Rejection, Reply,
};

use crate::services::{create_jwt, MONTH};

use super::{
    filters::{with_auth, AuthError},
    MAX_BODY_SIZE,
};

pub struct APIRouter {
    pub media_location: Arc<String>,
}

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

    pub async fn note_search(form_body: HashMap<String, String>) -> String {
        let term = form_body.get("term").unwrap();
        let now = Instant::now();
        let found_pages = semantic_search(term).await;
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
        Ok(tokio::fs::write(style_location, body).await?)
    }
}

impl APIRouter {
    pub fn routes(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        self.login()
            .or(self.logout())
            .or(self.styles())
            .or(self.img())
            .or(self.files())
            .or(self.search())
            .or(self.search_indicies())
    }
    fn search(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::post().and(with_auth()).and(
            warp::path("search").and(
                warp::body::content_length_limit(MAX_BODY_SIZE)
                    .and(warp::body::form())
                    .then(|form_body| async {
                        let results_page = Runner::note_search(form_body).await;
                        warp::reply::html(results_page)
                    }),
            ),
        )
    }
    fn search_indicies(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::get()
            .and(with_auth())
            .and(warp::path("search-idx").then(|| async {
                let indicies = Runner::dump_search_index().await;
                warp::reply::json(&indicies)
            }))
    }
    fn img(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::post().and(with_auth()).and(
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
    }
    fn files(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::post()
            .and(with_auth())
            .and(warp::body::content_length_limit(MAX_BODY_SIZE))
            .and(warp::filters::multipart::form())
            .then(|form_body| async {
                match Runner::file(form_body).await {
                    Ok(()) => warp::redirect(Uri::from_static("/")),
                    Err(e) => {
                        eprintln!("{}", e);
                        warp::redirect(Uri::from_static("/error"))
                    }
                }
            })
    }
    fn login(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::post().and(warp::path("login")).and(
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
    }
    fn logout(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::get().and(warp::path("logout")).then(|| async {
            Ok(Response::builder()
                .status(StatusCode::MOVED_PERMANENTLY)
                .header(header::LOCATION, HeaderValue::from_static("/"))
                .header(
                    header::SET_COOKIE,
                    "token=; Secure; HttpOnly; Max-Age=0; Path=/",
                )
                .body("ok"))
        })
    }
    fn styles(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::path("styles").and(warp::post().and(with_auth()).and(
            warp::body::content_length_limit(MAX_BODY_SIZE).and(warp::body::form().then(
                |form_body| async {
                    match Runner::update_styles(form_body).await {
                        Ok(()) => warp::redirect(Uri::from_static("/")),
                        Err(e) => {
                            eprintln!("{}", e);
                            warp::redirect(Uri::from_static("/error"))
                        }
                    }
                },
            )),
        ))
    }
}
