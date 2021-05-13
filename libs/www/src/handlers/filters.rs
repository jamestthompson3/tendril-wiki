use std::sync::Arc;

use ::build::RefBuilder;
use ::markdown::ingestors::fs::read;
use build::read_config;
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use markdown::{
    ingestors::ReadPageError,
    parsers::{LinkPage, NewPage, TagIndex, TagPage},
};
use sailfish::TemplateOnce;
use serde::{Deserialize, Serialize};
use tasks::normalize_wiki_location;
use thiserror::Error;
use urlencoding::decode;
use warp::{Filter, Rejection};

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("missing authentication")]
    AuthNotPresent,
    #[error("authentication incorrect")]
    BadCredentials,
    #[error("unknown auth error")]
    Unknown,
    #[error("no jwt")]
    NoJWT,
    #[error("jwt decode error")]
    JWTDecodeError,
    #[error("could not create jwt")]
    JWTTokenCreationError,
}

impl warp::reject::Reject for AuthError {}

pub type AuthResult<T> = std::result::Result<T, Rejection>;

#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    pub exp: usize,
    pub sub: String,
}

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
    warp::any()
        .and(warp::filters::cookie::optional("token"))
        .and_then(authorize)
        .untuple_one()
}

pub async fn authorize(token: Option<String>) -> AuthResult<()> {
    let config = read_config();
    if config.general.pass.is_empty() {
        return Ok(());
    }
    if token.is_none() {
        return Err(warp::reject::custom(AuthError::AuthNotPresent));
    }
    let token = token.unwrap();
    jsonwebtoken::decode::<Claims>(
        &token,
        &DecodingKey::from_secret(config.general.pass.as_bytes()),
        &Validation::new(Algorithm::HS512),
    )
    .map_err(|e| {
        eprintln!("{}",e);
        warp::reject::custom(AuthError::JWTDecodeError)
    })?;
    Ok(())
}
