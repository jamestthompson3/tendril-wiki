use bytes::BufMut;
use persistance::fs::{write, write_media};
use render::{
    search_results_context_page::SearchResultsContextPage, search_results_page::SearchResultsPage,
    uploaded_files_page::UploadedFilesPage, Render,
};
use std::{collections::HashMap, fs::read_dir, time::Instant};
use tasks::{context_search, search, CompileState};
use tokio::{fs::read_to_string, spawn};
use urlencoding::encode;

use markdown::parsers::EditPageData;

use logging::log;

use futures::TryStreamExt;

use build::{get_config_location, get_data_dir_location, RefBuilder};
use warp::{
    http::{header, HeaderValue, Response, StatusCode},
    hyper::{body::Bytes, Uri},
    multipart::{self, Part},
    Rejection, Reply,
};

use crate::{
    handlers::filters::AuthError,
    services::{create_jwt, MONTH},
};

pub async fn file(
    form: multipart::FormData,
    mut location: String,
) -> Result<impl Reply, Rejection> {
    let parts: Vec<Part> = form.try_collect().await.map_err(|e| {
        eprint!("Parsing form err: {}", e);
        warp::reject::reject()
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
    location.push_str(&filename);
    match write_media(&location, &data) {
        Ok(()) => Ok(warp::redirect(Uri::from_static("/"))),
        Err(e) => {
            eprintln!("Could not write media: {}", e);
            Err(warp::reject::reject())
        }
    }
}

pub async fn image(
    filename: String,
    bytes: Bytes,
    mut media_location: String,
) -> Result<impl Reply, Rejection> {
    media_location.push_str(&filename);
    write_media(&media_location, bytes.as_ref()).unwrap();
    Ok(warp::reply::with_status("ok", StatusCode::OK))
}

pub async fn edit(
    form_body: HashMap<String, String>,
    wiki_location: String,
    mut builder: RefBuilder,
    query_params: HashMap<String, String>,
) -> Result<impl Reply, Rejection> {
    let parsed_data = EditPageData::from(form_body);
    let redir_uri;
    if let Some(redirect_addition) = query_params.get("redir_to") {
        redir_uri = format!("/{}/{}", redirect_addition, encode(&parsed_data.title));
    } else {
        redir_uri = format!("/{}", encode(&parsed_data.title));
    }
    let now = Instant::now();
    let page_title = parsed_data.title.clone();
    match write(&wiki_location, parsed_data, builder.links()) {
        Ok(()) => {
            builder.build(&wiki_location);
            spawn(async {
                let mut data_dir = get_data_dir_location();
                data_dir.push("note_cache");
                match read_to_string(&data_dir).await {
                    Ok(content) => {
                        println!("--- {}", content);
                        let mut lines = content
                            .lines()
                            .filter(|&line| line != page_title)
                            .map(|l| l.to_string())
                            .collect::<Vec<String>>();
                        println!(">> {:?}", lines);
                        if lines.len() >= 5 {
                            lines.pop();
                        }
                        lines.insert(0, page_title);
                        tokio::fs::write(data_dir, lines.join("\n")).await.unwrap();
                    }
                    Err(_) => {
                        tokio::fs::write(data_dir, page_title).await.unwrap();
                    }
                }
            })
            .await
            .unwrap();
            log(format!("[Edit]: {:?}", now.elapsed()));
            Ok(warp::redirect(redir_uri.parse::<Uri>().unwrap()))
        }
        Err(e) => {
            eprintln!("{}", e);
            Ok(warp::redirect(Uri::from_static("/error")))
        }
    }
}

pub async fn note_search(
    form_body: HashMap<String, String>,
    wiki_location: String,
) -> Result<impl Reply, Rejection> {
    let term = form_body.get("term").unwrap();
    let include_context = form_body.get("context");
    match include_context {
        Some(_) => {
            let found_pages = context_search(term, &wiki_location).unwrap();
            // TODO: Maybe not a separate page here?
            let ctx = SearchResultsContextPage { pages: found_pages };
            Ok(warp::reply::html(ctx.render(&CompileState::Dynamic)))
        }
        None => {
            let found_pages = search(term, &wiki_location);
            let ctx = SearchResultsPage { pages: found_pages };
            Ok(warp::reply::html(ctx.render(&CompileState::Dynamic)))
        }
    }
}

pub async fn list_files(wiki_location: String) -> Result<impl Reply, Rejection> {
    // TODO: Make this async?
    let entries = read_dir(wiki_location).unwrap();
    let entries = entries
        .map(|entry| {
            let entry = entry.unwrap();
            entry.file_name().into_string().unwrap()
        })
        .collect::<Vec<String>>();
    let ctx = UploadedFilesPage { entries };
    Ok(warp::reply::html(ctx.render(&CompileState::Dynamic)))
}

pub async fn delete(
    mut builder: RefBuilder,
    wiki_location: String,
    form_body: HashMap<String, String>,
) -> Result<impl Reply, Rejection> {
    let title = form_body.get("title").unwrap();
    let now = Instant::now();
    match persistance::fs::delete(&wiki_location, title) {
        Ok(()) => {
            builder.build(&wiki_location);
            println!("[Delete] {}: {:?}", title, now.elapsed());

            Ok(warp::redirect(Uri::from_static("/")))
        }
        Err(e) => {
            eprint!("{}", e);
            Ok(warp::redirect(Uri::from_static("/error")))
        }
    }
}

pub async fn update_styles(form_body: HashMap<String, String>) -> Result<impl Reply, Rejection> {
    let (path, _) = get_config_location();
    let style_location = path.join("userstyles.css");
    let body = form_body.get("body").unwrap();
    std::fs::write(style_location, body).unwrap();
    Ok(warp::redirect(Uri::from_static("/")))
}

pub async fn authorize(form_body: HashMap<String, String>) -> Result<impl Reply, Rejection> {
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
            if let AuthError::JWTDecodeError = e {
                status = StatusCode::BAD_REQUEST;
            } else {
                status = StatusCode::FORBIDDEN;
            }
            // Response::builder().body("Bad creds".into())
            Ok(Response::builder()
                .status(status)
                .header(header::LOCATION, HeaderValue::from_static("/"))
                .body("ok"))
        }
    }
}
