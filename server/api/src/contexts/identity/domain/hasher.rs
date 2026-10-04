use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher as _, PasswordVerifier as _, SaltString},
    Argon2,
};
use crate::shared::error::{AppError, Result};

pub trait PasswordHasher: Send + Sync {
    fn hash_password(&self, password: &str) -> Result<String>;
    fn verify_password(&self, password: &str, hash: &str) -> Result<bool>;
}

#[derive(Clone, Default)]
pub struct Argon2PasswordHasher;

impl Argon2PasswordHasher {
    pub fn new() -> Self {
        Self
    }
}

impl PasswordHasher for Argon2PasswordHasher {
    fn hash_password(&self, password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| AppError::ExternalService(format!("Password hashing failed: {e}")))?
            .to_string();
        Ok(password_hash)
    }

    fn verify_password(&self, password: &str, hash: &str) -> Result<bool> {
        let parsed_hash = match PasswordHash::new(hash) {
            Ok(h) => h,
            Err(_) => return Ok(false),
        };
        let argon2 = Argon2::default();
        Ok(argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify_password() {
        let hasher = Argon2PasswordHasher::new();
        let password = "SuperSecretPassword123!";
        let hash = hasher.hash_password(password).expect("Hashing should succeed");
        assert!(hasher.verify_password(password, &hash).unwrap());
        assert!(!hasher.verify_password("WrongPassword!", &hash).unwrap());
    }
}
