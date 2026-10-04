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
    pub intelligence_url: String,
    pub cors_origins: Vec<String>,
    pub jwt_secret: String,
    pub access_token_ttl_seconds: i64,
    pub refresh_token_ttl_seconds: i64,
    pub encryption_key: String,
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
            intelligence_url: env::var("INTELLIGENCE_URL").unwrap_or_else(|_| "http://127.0.0.1:8001".to_string()),
            cors_origins: env::var("CORS_ORIGIN")
                .unwrap_or_else(|_| "http://localhost:3000,http://localhost:3001".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            jwt_secret: env::var("JWT_SECRET").unwrap_or_else(|_| "mensetsu_dev_super_secret_jwt_key_at_least_32_bytes!".to_string()),
            access_token_ttl_seconds: env::var("ACCESS_TOKEN_TTL_SECONDS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(900), 
            refresh_token_ttl_seconds: env::var("REFRESH_TOKEN_TTL_SECONDS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(2_592_000), 
            encryption_key: env::var("ENCRYPTION_KEY")
                .unwrap_or_else(|_| "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string()),
        }
    }
}
