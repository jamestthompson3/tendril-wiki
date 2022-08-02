use std::sync::Arc;

use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use persistance::fs::config::read_config;
use render::GlobalBacklinks;
use serde::{Deserialize, Serialize};
use tasks::JobQueue;
use thiserror::Error;
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
) -> impl Filter<Extract = (String,), Error = Rejection> + Clone {
    warp::any().map(move || (*wiki_location).to_owned()).boxed()
}

pub fn with_user(user: String) -> impl Filter<Extract = (String,), Error = Rejection> + Clone {
    warp::any().map(move || user.clone()).boxed()
}

pub fn with_host(host: String) -> impl Filter<Extract = (String,), Error = Rejection> + Clone {
    warp::any().map(move || host.clone()).boxed()
}

pub fn with_queue(
    queue: Arc<JobQueue>,
) -> impl Filter<Extract = (Arc<JobQueue>,), Error = Rejection> + Clone {
    warp::any().map(move || queue.clone()).boxed()
}

pub fn with_links(
    links: GlobalBacklinks,
) -> impl Filter<Extract = (GlobalBacklinks,), Error = Rejection> + Clone {
    warp::any().map(move || links.clone()).boxed()
}

pub fn with_auth() -> impl Filter<Extract = (), Error = Rejection> + Clone {
    warp::any()
        .and(warp::filters::cookie::optional("token"))
        .and_then(check_auth)
        .untuple_one()
        .boxed()
}

pub async fn check_auth(token: Option<String>) -> AuthResult<()> {
    let config = read_config();
    if config.general.pass.is_empty() {
        return Ok(());
    }
    if token.is_none() {
        return Err(warp::reject::custom(AuthError::AuthNotPresent));
    }
    let token = token.unwrap();
    if token.is_empty() {
        return Err(warp::reject::custom(AuthError::AuthNotPresent));
    }
    jsonwebtoken::decode::<Claims>(
        &token,
        &DecodingKey::from_secret(config.general.pass.as_bytes()),
        &Validation::new(Algorithm::HS512),
    )
    .map_err(|e| {
        eprintln!("{}", e);
        warp::reject::custom(AuthError::JWTDecodeError)
    })?;
    Ok(())
}
