use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, VerifyError},
    Argon2,
};
use rand_core::OsRng;

pub fn hash_password(password: &[u8]) -> String {
    let salt = SaltString::generate(&mut OsRng);
    // Argon2 with default params (Argon2id v19)
    let argon2 = Argon2::default();

    // Hash password to PHC string ($argon2id$v=19$...)
    let password_hash = argon2
        .hash_password_simple(password, salt.as_ref())
        .unwrap()
        .to_string();
    password_hash
}

pub fn verify_password(password: String, hash: String) -> Result<(), VerifyError> {
    let argon2 = Argon2::default();
    // Verify password against PHC string
    let parsed_hash = PasswordHash::new(&hash).unwrap();
    argon2.verify_password(&password.as_bytes(), &parsed_hash)
}
