use chrono::prelude::*;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};

use persistance::fs::config::read_config;
use task_runners::verify_password;

use crate::handlers::filters::{AuthError, Claims};

pub const MONTH: usize = 2629800;

pub fn create_jwt(username: &str, password: &str) -> Result<String, AuthError> {
    let config = read_config();

    if username != config.general.user {
        return Err(AuthError::BadCredentials);
    }
    match verify_password(password.into(), config.general.pass.clone()) {
        Ok(()) => {
            let expiration = Utc::now()
                .checked_add_signed(chrono::Duration::seconds(MONTH as i64))
                .expect("valid timestamp")
                .timestamp();
            let claims = Claims {
                sub: username.into(),
                exp: expiration as usize,
            };
            let header = Header::new(Algorithm::HS512);
            let encode = encode(
                &header,
                &claims,
                &EncodingKey::from_secret(config.general.pass.as_bytes()),
            )
            .map_err(|_| AuthError::JWTTokenCreationError)
            .unwrap();
            Ok(encode)
        }
        Err(_) => Err(AuthError::BadCredentials),
    }
}
