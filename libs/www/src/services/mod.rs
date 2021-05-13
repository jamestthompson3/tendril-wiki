use chrono::prelude::*;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use std::time::Instant;

use build::read_config;
use tasks::verify_password;

use crate::handlers::filters::{AuthError, Claims};

const MONTH: usize = 2592000;

pub fn create_jwt(username: &str, password: &str) -> Result<String, AuthError> {
    println!("creating JWT");
    let config = read_config();

    if username != config.general.user {
        return Err(AuthError::BadCredentials);
    }
    let now = Instant::now();
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
                &EncodingKey::from_secret(&config.general.pass.as_bytes()),
            )
            .map_err(|_| AuthError::JWTTokenCreationError)
            .unwrap();
            println!(
                "verify_password & generate jwt took: {}ms",
                now.elapsed().as_millis()
            );
            Ok(encode)
        }
        Err(_) => Err(AuthError::BadCredentials),
    }
}
