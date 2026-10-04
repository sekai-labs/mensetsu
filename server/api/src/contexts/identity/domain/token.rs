use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};

use crate::contexts::identity::domain::{SessionId, UserId};
use crate::shared::error::{AppError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub sid: String,
    pub exp: i64,
    pub iat: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Temp2faClaims {
    pub sub: String,
    pub purpose: String,
    pub exp: i64,
    pub iat: i64,
}

#[derive(Clone)]
pub struct TokenService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    access_ttl: i64,
    refresh_ttl: i64,
}

impl TokenService {
    pub fn new(secret: &str, access_ttl: i64, refresh_ttl: i64) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            access_ttl,
            refresh_ttl,
        }
    }

    pub fn generate_access_token(&self, user_id: &UserId, session_id: &SessionId) -> Result<String> {
        let now = Utc::now();
        let exp = now + Duration::seconds(self.access_ttl);
        let claims = Claims {
            sub: user_id.0.clone(),
            sid: session_id.0.clone(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| AppError::Unauthorized(format!("Failed to sign JWT: {e}")))
    }

    pub fn verify_access_token(&self, token: &str) -> Result<Claims> {
        let validation = Validation::default();
        decode::<Claims>(token, &self.decoding_key, &validation)
            .map(|data| data.claims)
            .map_err(|e| AppError::Unauthorized(format!("Invalid token: {e}")))
    }
    pub fn generate_temp_2fa_token(&self, user_id: &UserId) -> Result<String> {
        let now = Utc::now();
        let exp = now + Duration::seconds(300); // 5 minutes validity
        let claims = Temp2faClaims {
            sub: user_id.0.clone(),
            purpose: "2fa_challenge".to_string(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| AppError::Internal(format!("Failed to generate 2FA token: {e}")))
    }

    pub fn verify_temp_2fa_token(&self, token: &str) -> Result<UserId> {
        let validation = Validation::default();
        let token_data = decode::<Temp2faClaims>(token, &self.decoding_key, &validation)
            .map_err(|_| AppError::Unauthorized("Invalid or expired 2FA session".into()))?;
        if token_data.claims.purpose != "2fa_challenge" {
            return Err(AppError::Unauthorized("Invalid token purpose".into()));
        }
        Ok(UserId(token_data.claims.sub))
    }

    
    pub fn generate_refresh_token() -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(64)
            .map(char::from)
            .collect()
    }

    
    pub fn hash_refresh_token(token: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn refresh_ttl_seconds(&self) -> i64 {
        self.refresh_ttl
    }

    pub fn access_ttl_seconds(&self) -> i64 {
        self.access_ttl
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_generation_and_verification() {
        let service = TokenService::new("my_super_secret_test_key_which_is_long!", 900, 2592000);
        let uid = UserId("u1".into());
        let sid = SessionId("s1".into());

        let token = service.generate_access_token(&uid, &sid).expect("Token generation");
        let claims = service.verify_access_token(&token).expect("Token verification");

        assert_eq!(claims.sub, "u1");
        assert_eq!(claims.sid, "s1");
    }

    #[test]
    fn test_refresh_token_generation() {
        let t1 = TokenService::generate_refresh_token();
        let t2 = TokenService::generate_refresh_token();
        assert_eq!(t1.len(), 64);
        assert_ne!(t1, t2);

        let h1 = TokenService::hash_refresh_token(&t1);
        let h2 = TokenService::hash_refresh_token(&t1);
        assert_eq!(h1, h2);
    }
}
