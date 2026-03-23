use argon2::{
    Argon2,
    PasswordHash,
    PasswordVerifier,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng}, // ← thêm PasswordHasher
};

pub struct HashString;

impl HashString {
    pub fn hash_password(password: &str) -> String {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .unwrap()
            .to_string()
    }

    pub fn verify_password(password: &str, hash: &str) -> bool {
        let Ok(password_hash) = PasswordHash::new(hash) else {
            return false;
        };
        Argon2::default()
            .verify_password(password.as_bytes(), &password_hash)
            .is_ok()
    }
}
