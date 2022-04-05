use bytes::BufMut;
use persistance::fs::{write, write_media};
use render::{search_results_page::SearchResultsPage, Render};
use search_engine::semantic_search;
use std::{collections::HashMap, time::Instant};
use urlencoding::encode;

use markdown::parsers::EditPageData;

use futures::TryStreamExt;

use build::{get_config_location, get_data_dir_location, RefHubTx};
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
    match write_media(&location, &data).await {
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
    write_media(&media_location, bytes.as_ref()).await.unwrap();
    Ok(warp::reply::with_status("ok", StatusCode::OK))
}

pub async fn edit(
    form_body: HashMap<String, String>,
    wiki_location: String,
    sender: RefHubTx,
    query_params: HashMap<String, String>,
) -> Result<impl Reply, Rejection> {
    let parsed_data = EditPageData::from(form_body);
    let redir_uri = if let Some(redirect_addition) = query_params.get("redir_to") {
        format!("/{}/{}", redirect_addition, encode(&parsed_data.title))
    } else {
        format!("/{}", encode(&parsed_data.title))
    };
    let page_title = parsed_data.title.clone();
    let update_msg = format!("{}~~{}", parsed_data.old_title, page_title);
    match write(&wiki_location, parsed_data).await {
        Ok(()) => {
            sender.send(("update".into(), update_msg)).await.unwrap();
            Ok(warp::redirect(redir_uri.parse::<Uri>().unwrap()))
        }
        Err(e) => {
            eprintln!("{}", e);
            Ok(warp::redirect(Uri::from_static("/error")))
        }
    }
}

pub async fn note_search(form_body: HashMap<String, String>) -> Result<impl Reply, Rejection> {
    let term = form_body.get("term").unwrap();
    let now = Instant::now();
    let found_pages = semantic_search(term, get_data_dir_location()).await;
    println!(
        "Search Time [{:?}]  Search Results [{}]",
        now.elapsed(),
        found_pages.len()
    );
    let ctx = SearchResultsPage { pages: found_pages };
    Ok(warp::reply::html(ctx.render().await))
}

pub async fn update_styles(form_body: HashMap<String, String>) -> Result<impl Reply, Rejection> {
    let (path, _) = get_config_location();
    let style_location = path.join("userstyles.css");
    let body = form_body.get("body").unwrap();
    tokio::fs::write(style_location, body).await.unwrap();
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
}

pub async fn unauthorize() -> Result<impl Reply, Rejection> {
    Ok(Response::builder()
        .status(StatusCode::MOVED_PERMANENTLY)
        .header(header::LOCATION, HeaderValue::from_static("/"))
        .header(
            header::SET_COOKIE,
            "token=; Secure; HttpOnly; Max-Age=0; Path=/",
        )
        .body("ok"))
}
