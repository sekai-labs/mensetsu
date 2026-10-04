use async_trait::async_trait;
use crate::contexts::identity::domain::{AuthSession, Email, SessionId, User, UserId};
use crate::shared::error::Result;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: &UserId) -> Result<Option<User>>;
    async fn find_by_email(&self, email: &Email) -> Result<Option<User>>;
    async fn save(&self, user: &User) -> Result<()>;
}

#[async_trait]
pub trait AuthSessionRepository: Send + Sync {
    async fn find_by_id(&self, id: &SessionId) -> Result<Option<AuthSession>>;
    async fn find_by_token_hash(&self, token_hash: &str) -> Result<Option<AuthSession>>;
    async fn save(&self, session: &AuthSession) -> Result<()>;
    async fn revoke(&self, id: &SessionId) -> Result<()>;
    async fn revoke_all_for_user(&self, user_id: &UserId) -> Result<()>;
    async fn update_token_hash(
        &self,
        id: &SessionId,
        new_token_hash: &str,
        new_expires_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<()>;
}
