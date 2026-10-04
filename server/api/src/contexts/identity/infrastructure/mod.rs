use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use surrealdb::Surreal;
use surrealdb::engine::remote::ws::Client;

use crate::contexts::identity::domain::{
    repository::{AuthSessionRepository, UserRepository},
    AuthSession, Email, SessionId, User, UserId, UserStatus,
};
use crate::shared::error::{AppError, Result};

pub type Db = Surreal<Client>;

#[derive(Clone)]
pub struct SurrealUserRepository {
    client: Arc<Db>,
}

impl SurrealUserRepository {
    pub fn new(client: Arc<Db>) -> Self {
        Self { client }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct UserDbRecord {
    id: serde_json::Value,
    email: String,
    name: String,
    password_hash: String,
    status: Option<String>,
    role: Option<String>,
    totp_enabled: Option<bool>,
    totp_secret: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl UserDbRecord {
    fn into_domain(self) -> User {
        let raw_id = match &self.id {
            serde_json::Value::String(s) => s.replace("user:", "").replace('`', ""),
            serde_json::Value::Object(map) => {
                if let Some(serde_json::Value::String(id_val)) = map.get("id") {
                    id_val.to_string()
                } else {
                    self.id.to_string().replace("user:", "").replace('`', "")
                }
            }
            _ => self.id.to_string().replace("user:", "").replace('`', ""),
        };

        let status = match self.status.as_deref() {
            Some("suspended") => UserStatus::Suspended,
            _ => UserStatus::Active,
        };

        let role = match self.role.as_deref() {
            Some("admin") => crate::contexts::identity::domain::UserRole::Admin,
            _ => crate::contexts::identity::domain::UserRole::Member,
        };

        User {
            id: UserId(raw_id),
            email: self.email,
            name: self.name,
            password_hash: self.password_hash,
            status,
            role,
            totp_enabled: self.totp_enabled.unwrap_or(false),
            totp_secret: self.totp_secret,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

#[async_trait]
impl UserRepository for SurrealUserRepository {
    async fn find_by_id(&self, id: &UserId) -> Result<Option<User>> {
        let clean_id = id.0.replace("user:", "").replace('`', "");
        let mut res = self
            .client
            .query("SELECT * FROM type::record('user', $id);")
            .bind(("id", clean_id))
            .await
            .map_err(|e| AppError::Database(format!("DB find user by id failed: {e}")))?;

        let record: Option<serde_json::Value> = res
            .take(0)
            .map_err(|e| AppError::Database(format!("Failed to parse user: {e}")))?;

        let parsed: Option<UserDbRecord> = record.and_then(|v| serde_json::from_value(v).ok());
        Ok(parsed.map(UserDbRecord::into_domain))
    }

    async fn find_by_email(&self, email: &Email) -> Result<Option<User>> {
        let mut res = self
            .client
            .query("SELECT * FROM user WHERE email = $email LIMIT 1;")
            .bind(("email", email.0.clone()))
            .await
            .map_err(|e| AppError::Database(format!("DB find user by email failed: {e}")))?;

        let records: Vec<serde_json::Value> = res
            .take(0)
            .map_err(|e| AppError::Database(format!("Failed to parse user by email: {e}")))?;

        let user = records
            .into_iter()
            .next()
            .and_then(|v| serde_json::from_value::<UserDbRecord>(v).ok())
            .map(UserDbRecord::into_domain);
        Ok(user)
    }

    async fn save(&self, user: &User) -> Result<()> {
        let user_id = user.id.0.as_str();
        let query = "
            UPSERT type::record('user', $id) SET
                email = $email,
                name = $name,
                password_hash = $password_hash,
                status = $status,
                role = $role,
                totp_enabled = $totp_enabled,
                totp_secret = $totp_secret,
                created_at = type::datetime($created_at),
                updated_at = type::datetime($updated_at);
        ";
        let _ = self
            .client
            .query(query)
            .bind(("id", user_id.to_string()))
            .bind(("email", user.email.clone()))
            .bind(("name", user.name.clone()))
            .bind(("password_hash", user.password_hash.clone()))
            .bind(("status", user.status.to_string()))
            .bind(("role", user.role.to_string()))
            .bind(("totp_enabled", user.totp_enabled))
            .bind(("totp_secret", user.totp_secret.clone()))
            .bind(("created_at", user.created_at.to_rfc3339()))
            .bind(("updated_at", user.updated_at.to_rfc3339()))
            .await
            .map_err(|e| AppError::Database(format!("Failed to save user {user_id}: {e}")))?;

        Ok(())
    }
}

#[derive(Clone)]
pub struct SurrealAuthSessionRepository {
    client: Arc<Db>,
}

impl SurrealAuthSessionRepository {
    pub fn new(client: Arc<Db>) -> Self {
        Self { client }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct AuthSessionDbRecord {
    id: serde_json::Value,
    user_id: serde_json::Value,
    refresh_token_hash: String,
    user_agent: Option<String>,
    ip_address: Option<String>,
    expires_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
    last_used_at: DateTime<Utc>,
    revoked: bool,
}

impl AuthSessionDbRecord {
    fn into_domain(self) -> AuthSession {
        let raw_id = match &self.id {
            serde_json::Value::String(s) => s.replace("auth_session:", "").replace('`', ""),
            _ => self.id.to_string().replace("auth_session:", "").replace('`', ""),
        };
        let raw_user_id = match &self.user_id {
            serde_json::Value::String(s) => s.replace("user:", "").replace('`', ""),
            _ => self.user_id.to_string().replace("user:", "").replace('`', ""),
        };

        AuthSession {
            id: SessionId(raw_id),
            user_id: UserId(raw_user_id),
            refresh_token_hash: self.refresh_token_hash,
            user_agent: self.user_agent,
            ip_address: self.ip_address,
            expires_at: self.expires_at,
            created_at: self.created_at,
            last_used_at: self.last_used_at,
            revoked: self.revoked,
        }
    }
}

#[async_trait]
impl AuthSessionRepository for SurrealAuthSessionRepository {
    async fn find_by_id(&self, id: &SessionId) -> Result<Option<AuthSession>> {
        let mut res = self
            .client
            .query("SELECT * FROM type::record('auth_session', $id);")
            .bind(("id", id.0.clone()))
            .await
            .map_err(|e| AppError::Database(format!("DB find auth_session by id failed: {e}")))?;

        let record: Option<serde_json::Value> = res
            .take(0)
            .map_err(|e| AppError::Database(format!("Failed to parse session: {e}")))?;

        let parsed: Option<AuthSessionDbRecord> = record.and_then(|v| serde_json::from_value(v).ok());
        Ok(parsed.map(AuthSessionDbRecord::into_domain))
    }

    async fn find_by_token_hash(&self, token_hash: &str) -> Result<Option<AuthSession>> {
        let mut res = self
            .client
            .query("SELECT * FROM auth_session WHERE refresh_token_hash = $hash AND revoked = false LIMIT 1;")
            .bind(("hash", token_hash.to_string()))
            .await
            .map_err(|e| AppError::Database(format!("DB find session by hash failed: {e}")))?;

        let records: Vec<serde_json::Value> = res
            .take(0)
            .map_err(|e| AppError::Database(format!("Failed to parse session by hash: {e}")))?;

        let session = records
            .into_iter()
            .next()
            .and_then(|v| serde_json::from_value::<AuthSessionDbRecord>(v).ok())
            .map(AuthSessionDbRecord::into_domain);
        Ok(session)
    }

    async fn save(&self, session: &AuthSession) -> Result<()> {
        let sid = session.id.0.as_str();
        let query = "
            UPSERT type::record('auth_session', $id) SET
                user_id = type::record('user', $user_id),
                refresh_token_hash = $refresh_token_hash,
                user_agent = $user_agent,
                ip_address = $ip_address,
                expires_at = type::datetime($expires_at),
                created_at = type::datetime($created_at),
                last_used_at = type::datetime($last_used_at),
                revoked = $revoked;
        ";
        let _ = self
            .client
            .query(query)
            .bind(("id", sid.to_string()))
            .bind(("user_id", session.user_id.0.clone()))
            .bind(("refresh_token_hash", session.refresh_token_hash.clone()))
            .bind(("user_agent", session.user_agent.clone()))
            .bind(("ip_address", session.ip_address.clone()))
            .bind(("expires_at", session.expires_at.to_rfc3339()))
            .bind(("created_at", session.created_at.to_rfc3339()))
            .bind(("last_used_at", session.last_used_at.to_rfc3339()))
            .bind(("revoked", session.revoked))
            .await
            .map_err(|e| AppError::Database(format!("Failed to save auth session {sid}: {e}")))?;

        Ok(())
    }

    async fn revoke(&self, id: &SessionId) -> Result<()> {
        let mut res = self
            .client
            .query("UPDATE type::record('auth_session', $id) SET revoked = true, updated_at = time::now();")
            .bind(("id", id.0.clone()))
            .await
            .map_err(|e| AppError::Database(format!("Failed to revoke auth session: {e}")))?;

        let _: Option<serde_json::Value> = res.take(0).ok().flatten();
        Ok(())
    }

    async fn revoke_all_for_user(&self, user_id: &UserId) -> Result<()> {
        let mut res = self
            .client
            .query("UPDATE auth_session SET revoked = true WHERE user_id = type::record('user', $user_id);")
            .bind(("user_id", user_id.0.clone()))
            .await
            .map_err(|e| AppError::Database(format!("Failed to revoke all sessions for user: {e}")))?;

        let _: Vec<serde_json::Value> = res.take(0).unwrap_or_default();
        Ok(())
    }

    async fn update_token_hash(
        &self,
        id: &SessionId,
        new_token_hash: &str,
        new_expires_at: DateTime<Utc>,
    ) -> Result<()> {
        let mut res = self
            .client
            .query("UPDATE type::record('auth_session', $id) SET refresh_token_hash = $hash, expires_at = $exp, last_used_at = time::now();")
            .bind(("id", id.0.clone()))
            .bind(("hash", new_token_hash.to_string()))
            .bind(("exp", new_expires_at))
            .await
            .map_err(|e| AppError::Database(format!("Failed to rotate session refresh token: {e}")))?;

        let _: Option<serde_json::Value> = res.take(0).ok().flatten();
        Ok(())
    }
}
