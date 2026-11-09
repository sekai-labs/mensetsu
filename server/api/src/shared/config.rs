use std::env;

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub surreal_url: String,
    pub surreal_user: String,
    pub surreal_pass: String,
    pub surreal_ns: String,
    pub surreal_db: String,
    pub intelligence_grpc_url: String,
    pub cors_origins: Vec<String>,
    pub jwt_secret: String,
    pub access_token_ttl_seconds: i64,
    pub refresh_token_ttl_seconds: i64,
    pub encryption_key: String,
    pub internal_service_token: String,
    pub s3_bucket: String,
    pub aws_region: String,
    pub s3_endpoint_internal: Option<String>,
    pub s3_endpoint_public: Option<String>,
    pub api_grpc_bind: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3000),
            surreal_url: env::var("SURREAL_URL").unwrap_or_else(|_| "ws://127.0.0.1:8000".to_string()),
            surreal_user: env::var("SURREAL_USER").unwrap_or_else(|_| "root".to_string()),
            surreal_pass: env::var("SURREAL_PASS").unwrap_or_else(|_| "root".to_string()),
            surreal_ns: env::var("SURREAL_NS").unwrap_or_else(|_| "mensetsu".to_string()),
            surreal_db: env::var("SURREAL_DB").unwrap_or_else(|_| "production".to_string()),
            intelligence_grpc_url: env::var("INTELLIGENCE_GRPC_ENDPOINT")
                .unwrap_or_else(|_| "http://127.0.0.1:50051".to_string()),
            cors_origins: env::var("CORS_ORIGIN")
                .unwrap_or_else(|_| "http://localhost:3000,http://localhost:3001".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            jwt_secret: env::var("JWT_SECRET").unwrap_or_default(),
            access_token_ttl_seconds: env::var("ACCESS_TOKEN_TTL_SECONDS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(900), 
            refresh_token_ttl_seconds: env::var("REFRESH_TOKEN_TTL_SECONDS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(2_592_000), 
            encryption_key: env::var("ENCRYPTION_KEY").unwrap_or_default(),
            internal_service_token: env::var("INTERNAL_SERVICE_TOKEN")
                .map(|t| t.trim().to_string())
                .unwrap_or_default(),
            s3_bucket: env::var("S3_BUCKET").unwrap_or_else(|_| "mensetsu-resumes".to_string()),
            aws_region: env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
            s3_endpoint_internal: env::var("S3_ENDPOINT_INTERNAL").ok().filter(|s| !s.is_empty()),
            s3_endpoint_public: env::var("S3_ENDPOINT_PUBLIC").ok().filter(|s| !s.is_empty()),
            api_grpc_bind: env::var("API_GRPC_BIND").unwrap_or_else(|_| "127.0.0.1:50052".to_string()),
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        for (name, value) in [
            ("JWT_SECRET", self.jwt_secret.as_str()),
            ("ENCRYPTION_KEY", self.encryption_key.as_str()),
        ] {
            if value.trim().is_empty() {
                return Err(format!("{name} is unset or blank; refusing to start"));
            }
            if value.as_bytes().len() < 32 {
                return Err(format!("{name} must be at least 32 bytes; refusing to start"));
            }
        }

        let former_fallbacks = [
            "mensetsu_dev_super_secret_jwt_key_at_least_32_bytes!",
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        ];
        if former_fallbacks
            .iter()
            .any(|fallback| self.jwt_secret == *fallback || self.encryption_key == *fallback)
        {
            return Err("A known insecure fallback secret is configured; refusing to start".into());
        }
        if self.jwt_secret == self.encryption_key {
            return Err("JWT_SECRET and ENCRYPTION_KEY must be different; refusing to start".into());
        }

        if self.internal_service_token.trim().is_empty() {
            return Err("INTERNAL_SERVICE_TOKEN is unset or empty; refusing to start".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::AppConfig;

    fn config(jwt_secret: &str, encryption_key: &str) -> AppConfig {
        let mut config = AppConfig::from_env();
        config.jwt_secret = jwt_secret.to_string();
        config.encryption_key = encryption_key.to_string();
        config.internal_service_token = "test-internal-token".to_string();
        config
    }

    #[test]
    fn validate_rejects_missing_or_blank_secrets() {
        assert!(config("", "distinct-encryption-key-123456789").validate().is_err());
        assert!(config("jwt-secret-value-123456789012345", " \t ").validate().is_err());
    }

    #[test]
    fn validate_rejects_secrets_shorter_than_32_bytes() {
        assert!(config("short", "distinct-encryption-key-123456789").validate().is_err());
        assert!(config("jwt-secret-value-123456789012345", "short").validate().is_err());
    }

    #[test]
    fn validate_rejects_known_fallback_secrets() {
        assert!(config(
            "mensetsu_dev_super_secret_jwt_key_at_least_32_bytes!",
            "different-encryption-secret-value-123456789"
        )
        .validate()
        .is_err());
        assert!(config(
            "different-jwt-secret-value-123456789012",
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
        )
        .validate()
        .is_err());
    }

    #[test]
    fn validate_rejects_reused_secrets() {
        let secret = "same-secret-value-with-at-least-32-bytes";
        assert!(config(secret, secret).validate().is_err());
    }

    #[test]
    fn validate_accepts_distinct_secrets_and_requires_internal_token() {
        assert!(config(
            "a-valid-jwt-secret-value-with-32-bytes",
            "a-valid-encryption-key-with-32-bytes"
        )
        .validate()
        .is_ok());
        let mut missing_token = config(
            "a-valid-jwt-secret-value-with-32-bytes",
            "a-valid-encryption-key-with-32-bytes",
        );
        missing_token.internal_service_token = "  ".to_string();
        assert!(missing_token.validate().is_err());
    }
}
