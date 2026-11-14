use crate::contexts::identity::domain::{
    oauth::{AuthHandoff, OAuthFlowState, OAuthProviderConfig, UserOAuthIdentity},
    AuthSession, Email, SessionId, User, UserId,
};
use crate::shared::error::Result;
use async_trait::async_trait;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: &UserId) -> Result<Option<User>>;
    async fn find_by_email(&self, email: &Email) -> Result<Option<User>>;
    async fn save(&self, user: &User) -> Result<()>;
}

#[async_trait]
pub trait OAuthProviderRepository: Send + Sync {
    async fn find_by_key(&self, provider_key: &str) -> Result<Option<OAuthProviderConfig>>;
    async fn list_all(&self) -> Result<Vec<OAuthProviderConfig>>;
    async fn list_enabled(&self) -> Result<Vec<OAuthProviderConfig>>;
    async fn save(&self, provider: &OAuthProviderConfig) -> Result<()>;
    async fn delete(&self, provider_key: &str) -> Result<bool>;
}

#[async_trait]
pub trait UserOAuthIdentityRepository: Send + Sync {
    async fn find_by_provider_subject(
        &self,
        provider_key: &str,
        provider_user_id: &str,
    ) -> Result<Option<UserOAuthIdentity>>;
    async fn find_by_id(&self, id: &str) -> Result<Option<UserOAuthIdentity>>;
    async fn list_for_user(&self, user_id: &UserId) -> Result<Vec<UserOAuthIdentity>>;
    async fn count_for_user(&self, user_id: &UserId) -> Result<usize>;
    async fn save(&self, identity: &UserOAuthIdentity) -> Result<()>;
    async fn touch_last_login(&self, id: &str) -> Result<()>;
    async fn delete(&self, id: &str) -> Result<bool>;
}

#[async_trait]
pub trait OAuthFlowRepository: Send + Sync {
    async fn save(&self, flow: &OAuthFlowState) -> Result<()>;
    /// Atomically consumes the flow. Returns `None` when the state is unknown,
    /// already used, or expired.
    async fn consume(&self, state: &str) -> Result<Option<OAuthFlowState>>;
}

#[async_trait]
pub trait AuthHandoffRepository: Send + Sync {
    async fn save(&self, handoff: &AuthHandoff) -> Result<()>;
    /// Atomically consumes the one-use exchange code.
    async fn consume_by_hash(&self, code_hash: &str) -> Result<Option<AuthHandoff>>;
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
