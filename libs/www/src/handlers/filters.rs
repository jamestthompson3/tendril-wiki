use std::sync::Arc;

use ::build::RefBuilder;
use build::read_config;
use ::markdown::ingestors::fs::read;
use markdown::{
    ingestors::ReadPageError,
    parsers::{LinkPage, NewPage, TagIndex, TagPage},
};
use sailfish::TemplateOnce;
use tasks::{normalize_wiki_location, verify_password};
use urlencoding::decode;
use warp::{
    header::headers_cloned,
    http::HeaderValue,
    hyper::{header::AUTHORIZATION, HeaderMap},
    Filter, Rejection,
};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("missing authentication")]
    AuthNotPresent,
    #[error("authentication incorrect")]
    BadCredentials,
    #[error("unknown auth error")]
    Unknown,
}

impl warp::reject::Reject for AuthError {}

type AuthResult<T> = std::result::Result<T, Rejection>;

pub fn with_location(
    wiki_location: Arc<String>,
) -> impl Filter<Extract = (String,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || normalize_wiki_location(&wiki_location))
}

pub async fn with_file(
    path: String,
    refs: RefBuilder,
    wiki_location: String,
) -> Result<impl warp::Reply, warp::Rejection> {
    match path.as_str() {
        "links" => {
            let ref_links = refs.links();
            let links = ref_links.lock().unwrap();
            let ctx = LinkPage {
                links: links.clone(),
            };
            Ok(warp::reply::html(ctx.render_once().unwrap()))
        }
        "tags" => {
            let ref_tags = refs.tags();
            let tags = ref_tags.lock().unwrap();
            let ctx = TagIndex { tags: tags.clone() };
            Ok(warp::reply::html(ctx.render_once().unwrap()))
        }
        _ => {
            let links = refs.links();
            let tags = refs.tags();
            match read(&wiki_location, path.clone(), tags, links) {
                Ok(page) => Ok(warp::reply::html(page)),
                Err(ReadPageError::PageNotFoundError) => {
                    // TODO: Ideally, I want to redirect, but I'm not sure how to do this with
                    // warp's filter system where some branches return HTML, and others redirect...
                    let ctx = NewPage {
                        title: Some(decode(&path).unwrap()),
                    };

                    Ok(warp::reply::html(ctx.render_once().unwrap()))
                }
                _ => Err(warp::reject()),
            }
        }
    }
}

// TODO: Not repeat this the same as file
pub async fn with_nested_file(
    mut main_path: String,
    sub_path: String,
    refs: RefBuilder,
    wiki_location: String,
) -> Result<impl warp::Reply, warp::Rejection> {
    match main_path.as_str() {
        "tags" => {
            let ref_tags = refs.tags();
            let tags = ref_tags.lock().unwrap();
            // I don't know why warp doesn't decode the sub path here...
            let sub_path_decoded = decode(&sub_path).unwrap();
            match tags.get(&sub_path_decoded) {
                Some(tags) => {
                    let ctx = TagPage {
                        title: sub_path_decoded,
                        tags: tags.to_owned(),
                    };
                    Ok(warp::reply::html(ctx.render_once().unwrap()))
                }
                None => Err(warp::reject()),
            }
        }
        _ => {
            // I don't know why warp doesn't decode the sub path here...
            let sub_path_decoded = decode(&sub_path).unwrap();
            let links = refs.links();
            let tags = refs.tags();
            main_path.push_str(&sub_path_decoded.as_str());
            let page = read(&wiki_location, main_path, tags, links).map_err(|_| warp::reject())?;
            Ok(warp::reply::html(page))
        }
    }
}

pub fn with_user(
    user: String,
) -> impl Filter<Extract = (String,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || user.clone())
}

pub fn with_refs(
    refs: RefBuilder,
) -> impl Filter<Extract = (RefBuilder,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || refs.clone())
}

pub fn with_auth() -> impl Filter<Extract = (), Error = Rejection> + Clone {
    headers_cloned()
        .map(move |headers: HeaderMap<HeaderValue>| headers)
        .and_then(authorize)
        .untuple_one()
}

pub async fn authorize(headers: HeaderMap<HeaderValue>) -> AuthResult<()> {
    let config = read_config();
    if config.general.pass.is_empty() {
        return Ok(())
    }
    match headers.get(AUTHORIZATION) {
        Some(auth) => {
            let uname_pwd = auth.to_str().unwrap().strip_prefix("Basic ").unwrap();
            let decoded = String::from_utf8(base64::decode(&uname_pwd).unwrap()).unwrap();
            let auth_info: Vec<&str> = decoded.split(':').collect();
            if auth_info[0] != config.general.user {
                return Err(warp::reject::custom(AuthError::BadCredentials))
            }
            match verify_password(auth_info[1].into(), config.general.pass) {
                Ok(()) => {
                    Ok(())
                },
                Err(_) => Err(warp::reject::custom(AuthError::BadCredentials))
            }


        }
        None => return Err(warp::reject::custom(AuthError::AuthNotPresent)),
    }
}
