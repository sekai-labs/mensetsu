pub mod hasher;
pub mod repository;
pub mod token;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::shared::error::{AppError, Result};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, ToSchema)]
pub struct UserId(pub String);

impl Default for UserId {
    fn default() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, ToSchema)]
pub struct SessionId(pub String);

impl Default for SessionId {
    fn default() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct Email(pub String);

impl Email {
    pub fn new(raw: impl Into<String>) -> Result<Self> {
        let s = raw.into().trim().to_lowercase();
        if !s.contains('@') || !s.contains('.') || s.len() < 5 {
            return Err(AppError::Validation(format!("Invalid email address: {s}")));
        }
        Ok(Self(s))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Email {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum UserStatus {
    #[default]
    Active,
    Suspended,
}

impl std::fmt::Display for UserStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Suspended => write!(f, "suspended"),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    Admin,
    #[default]
    Member,
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Admin => write!(f, "admin"),
            Self::Member => write!(f, "member"),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct User {
    pub id: UserId,
    pub email: String,
    pub name: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub status: UserStatus,
    pub role: UserRole,
    pub totp_enabled: bool,
    #[serde(skip_serializing)]
    pub totp_secret: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn new(
        email: Email,
        name: String,
        password_hash: String,
        role: UserRole,
    ) -> Result<Self> {
        let trimmed_name = name.trim().to_string();
        if trimmed_name.is_empty() {
            return Err(AppError::Validation("Name cannot be empty".into()));
        }

        let now = Utc::now();
        Ok(Self {
            id: UserId::default(),
            email: email.0,
            name: trimmed_name,
            password_hash,
            status: UserStatus::Active,
            role,
            totp_enabled: false,
            totp_secret: None,
            created_at: now,
            updated_at: now,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSession {
    pub id: SessionId,
    pub user_id: UserId,
    pub refresh_token_hash: String,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub last_used_at: DateTime<Utc>,
    pub revoked: bool,
}

impl AuthSession {
    pub fn new(
        user_id: UserId,
        refresh_token_hash: String,
        ttl_seconds: i64,
        user_agent: Option<String>,
        ip_address: Option<String>,
    ) -> Self {
        let now = Utc::now();
        let expires_at = now + chrono::Duration::seconds(ttl_seconds);
        Self {
            id: SessionId::default(),
            user_id,
            refresh_token_hash,
            user_agent,
            ip_address,
            expires_at,
            created_at: now,
            last_used_at: now,
            revoked: false,
        }
    }

    pub fn is_active(&self) -> bool {
        !self.revoked && self.expires_at > Utc::now()
    }
}
