use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;

use crate::contexts::identity::domain::{
    oauth::{
        AuthHandoff, OAuthFlowPurpose, OAuthFlowState, OAuthProviderConfig, OAuthProviderType,
        UserOAuthIdentity,
    },
    repository::{
        AuthHandoffRepository, OAuthFlowRepository, OAuthProviderRepository,
        UserOAuthIdentityRepository,
    },
    UserId,
};
use crate::shared::error::{AppError, Result};
use crate::shared::record_id::clean_record_id;

pub type Db = Surreal<Client>;

fn record_id(value: &Value, table: &str) -> String {
    match value {
        Value::String(raw) => clean_record_id(raw),
        other => clean_record_id(&other.to_string()).replace(&format!("{table}:"), ""),
    }
}

fn opt_string(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(Value::as_str).map(str::to_string)
}

fn required_string(value: &Value, key: &str) -> Result<String> {
    opt_string(value, key)
        .ok_or_else(|| AppError::Database(format!("OAuth record is missing '{key}'")))
}

fn string_array(value: &Value, key: &str) -> Vec<String> {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

fn parse_datetime(value: &Value, key: &str) -> Result<DateTime<Utc>> {
    value
        .get(key)
        .and_then(|raw| serde_json::from_value::<DateTime<Utc>>(raw.clone()).ok())
        .ok_or_else(|| AppError::Database(format!("OAuth record has invalid '{key}'")))
}

fn parse_provider(row: Value) -> Result<OAuthProviderConfig> {
    let provider_type_raw = required_string(&row, "provider_type")?;
    let provider_type = OAuthProviderType::parse(&provider_type_raw).ok_or_else(|| {
        AppError::Database(format!("Unknown OAuth provider type '{provider_type_raw}'"))
    })?;

    Ok(OAuthProviderConfig {
        provider_key: required_string(&row, "provider_key")?,
        name: required_string(&row, "name")?,
        provider_type,
        issuer: opt_string(&row, "issuer"),
        client_id: required_string(&row, "client_id")?,
        encrypted_client_secret: required_string(&row, "encrypted_client_secret")?,
        redirect_uri: required_string(&row, "redirect_uri")?,
        scopes: string_array(&row, "scopes"),
        authorization_endpoint: opt_string(&row, "authorization_endpoint"),
        token_endpoint: opt_string(&row, "token_endpoint"),
        userinfo_endpoint: opt_string(&row, "userinfo_endpoint"),
        email_endpoint: opt_string(&row, "email_endpoint"),
        jwks_uri: opt_string(&row, "jwks_uri"),
        subject_claim: opt_string(&row, "subject_claim"),
        email_claim: opt_string(&row, "email_claim"),
        name_claim: opt_string(&row, "name_claim"),
        email_verified_claim: opt_string(&row, "email_verified_claim"),
        is_enabled: row
            .get("is_enabled")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        created_at: parse_datetime(&row, "created_at")?,
        updated_at: parse_datetime(&row, "updated_at")?,
    })
}

fn parse_identity(row: Value) -> Result<UserOAuthIdentity> {
    let user_id = row
        .get("user_id")
        .map(|value| record_id(value, "user"))
        .filter(|id| !id.is_empty())
        .ok_or_else(|| AppError::Database("OAuth identity is missing user_id".into()))?;

    Ok(UserOAuthIdentity {
        id: row
            .get("id")
            .map(|value| record_id(value, "user_oauth_account"))
            .ok_or_else(|| AppError::Database("OAuth identity is missing id".into()))?,
        user_id: UserId(user_id),
        provider_key: required_string(&row, "provider_key")?,
        provider_user_id: required_string(&row, "provider_user_id")?,
        email: opt_string(&row, "email"),
        email_verified: row
            .get("email_verified")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        display_name: opt_string(&row, "display_name"),
        created_at: parse_datetime(&row, "created_at")?,
        last_login_at: parse_datetime(&row, "last_login_at")?,
    })
}

fn parse_flow(row: Value) -> Result<OAuthFlowState> {
    let purpose = match required_string(&row, "purpose")?.as_str() {
        "login" => OAuthFlowPurpose::Login,
        "link" => OAuthFlowPurpose::Link,
        other => {
            return Err(AppError::Database(format!(
                "Unknown OAuth flow purpose '{other}'"
            )))
        }
    };
    let link_user_id = row
        .get("link_user_id")
        .filter(|value| !value.is_null())
        .map(|value| UserId(record_id(value, "user")))
        .filter(|id| !id.0.is_empty());

    Ok(OAuthFlowState {
        id: row
            .get("id")
            .map(|value| record_id(value, "oauth_state"))
            .ok_or_else(|| AppError::Database("OAuth state is missing id".into()))?,
        state: required_string(&row, "state")?,
        provider_key: required_string(&row, "provider_key")?,
        purpose,
        link_user_id,
        encrypted_code_verifier: required_string(&row, "code_verifier")?,
        nonce: opt_string(&row, "nonce"),
        redirect_uri: required_string(&row, "redirect_uri")?,
        expires_at: parse_datetime(&row, "expires_at")?,
        used: row.get("used").and_then(Value::as_bool).unwrap_or(false),
        created_at: parse_datetime(&row, "created_at")?,
    })
}

#[derive(Clone)]
pub struct SurrealOAuthProviderRepository {
    client: Arc<Db>,
}

impl SurrealOAuthProviderRepository {
    pub fn new(client: Arc<Db>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl OAuthProviderRepository for SurrealOAuthProviderRepository {
    async fn find_by_key(&self, provider_key: &str) -> Result<Option<OAuthProviderConfig>> {
        let mut response = self
            .client
            .query("SELECT * FROM oauth_provider WHERE provider_key = $provider_key LIMIT 1;")
            .bind(("provider_key", provider_key.to_string()))
            .await
            .map_err(|e| AppError::Database(format!("OAuth provider lookup failed: {e}")))?;

        let rows: Vec<Value> = response
            .take(0)
            .map_err(|e| AppError::Database(format!("OAuth provider decode failed: {e}")))?;

        rows.into_iter().next().map(parse_provider).transpose()
    }

    async fn list_all(&self) -> Result<Vec<OAuthProviderConfig>> {
        let mut response = self
            .client
            .query("SELECT * FROM oauth_provider ORDER BY provider_key;")
            .await
            .map_err(|e| AppError::Database(format!("OAuth provider list failed: {e}")))?;
        let rows: Vec<Value> = response
            .take(0)
            .map_err(|e| AppError::Database(format!("OAuth provider decode failed: {e}")))?;
        rows.into_iter().map(parse_provider).collect()
    }

    async fn list_enabled(&self) -> Result<Vec<OAuthProviderConfig>> {
        let mut response = self
            .client
            .query("SELECT * FROM oauth_provider WHERE is_enabled = true ORDER BY provider_key;")
            .await
            .map_err(|e| AppError::Database(format!("OAuth provider list failed: {e}")))?;
        let rows: Vec<Value> = response
            .take(0)
            .map_err(|e| AppError::Database(format!("OAuth provider decode failed: {e}")))?;
        rows.into_iter().map(parse_provider).collect()
    }

    async fn save(&self, provider: &OAuthProviderConfig) -> Result<()> {
        self.client
            .query(
                "UPSERT type::record('oauth_provider', $provider_key) SET
                    provider_key = $provider_key,
                    name = $name,
                    provider_type = $provider_type,
                    issuer = $issuer,
                    client_id = $client_id,
                    encrypted_client_secret = $encrypted_client_secret,
                    redirect_uri = $redirect_uri,
                    scopes = $scopes,
                    authorization_endpoint = $authorization_endpoint,
                    token_endpoint = $token_endpoint,
                    userinfo_endpoint = $userinfo_endpoint,
                    email_endpoint = $email_endpoint,
                    jwks_uri = $jwks_uri,
                    subject_claim = $subject_claim,
                    email_claim = $email_claim,
                    name_claim = $name_claim,
                    email_verified_claim = $email_verified_claim,
                    is_enabled = $is_enabled,
                    created_at = $created_at,
                    updated_at = time::now();",
            )
            .bind(("provider_key", provider.provider_key.clone()))
            .bind(("name", provider.name.clone()))
            .bind((
                "provider_type",
                match provider.provider_type {
                    OAuthProviderType::Google => "google",
                    OAuthProviderType::Github => "github",
                    OAuthProviderType::Facebook => "facebook",
                    OAuthProviderType::Oidc => "oidc",
                    OAuthProviderType::OAuth2 => "oauth2",
                },
            ))
            .bind(("issuer", provider.issuer.clone()))
            .bind(("client_id", provider.client_id.clone()))
            .bind((
                "encrypted_client_secret",
                provider.encrypted_client_secret.clone(),
            ))
            .bind(("redirect_uri", provider.redirect_uri.clone()))
            .bind(("scopes", provider.scopes.clone()))
            .bind((
                "authorization_endpoint",
                provider.authorization_endpoint.clone(),
            ))
            .bind(("token_endpoint", provider.token_endpoint.clone()))
            .bind(("userinfo_endpoint", provider.userinfo_endpoint.clone()))
            .bind(("email_endpoint", provider.email_endpoint.clone()))
            .bind(("jwks_uri", provider.jwks_uri.clone()))
            .bind(("subject_claim", provider.subject_claim.clone()))
            .bind(("email_claim", provider.email_claim.clone()))
            .bind(("name_claim", provider.name_claim.clone()))
            .bind((
                "email_verified_claim",
                provider.email_verified_claim.clone(),
            ))
            .bind(("is_enabled", provider.is_enabled))
            .bind(("created_at", provider.created_at))
            .await
            .map_err(|e| {
                AppError::Database(format!(
                    "Failed to save OAuth provider '{}': {e}",
                    provider.provider_key
                ))
            })?
            .check()
            .map_err(|e| {
                AppError::Database(format!(
                    "Failed to save OAuth provider '{}': {e}",
                    provider.provider_key
                ))
            })?;
        Ok(())
    }

    async fn delete(&self, provider_key: &str) -> Result<bool> {
        let mut response = self
            .client
            .query("DELETE oauth_provider WHERE provider_key = $provider_key RETURN BEFORE;")
            .bind(("provider_key", provider_key.to_string()))
            .await
            .map_err(|e| AppError::Database(format!("Failed to delete OAuth provider: {e}")))?;
        let rows: Vec<Value> = response.take(0).unwrap_or_default();
        Ok(!rows.is_empty())
    }
}

#[derive(Clone)]
pub struct SurrealUserOAuthIdentityRepository {
    client: Arc<Db>,
}

impl SurrealUserOAuthIdentityRepository {
    pub fn new(client: Arc<Db>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl UserOAuthIdentityRepository for SurrealUserOAuthIdentityRepository {
    async fn find_by_provider_subject(
        &self,
        provider_key: &str,
        provider_user_id: &str,
    ) -> Result<Option<UserOAuthIdentity>> {
        let mut response = self
            .client
            .query("SELECT * FROM user_oauth_account WHERE provider_key = $provider_key AND provider_user_id = $provider_user_id LIMIT 1;")
            .bind(("provider_key", provider_key.to_string()))
            .bind(("provider_user_id", provider_user_id.to_string()))
            .await
            .map_err(|e| AppError::Database(format!("OAuth identity lookup failed: {e}")))?;
        let rows: Vec<Value> = response
            .take(0)
            .map_err(|e| AppError::Database(format!("OAuth identity decode failed: {e}")))?;
        rows.into_iter().next().map(parse_identity).transpose()
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<UserOAuthIdentity>> {
        let mut response = self
            .client
            .query("SELECT * FROM type::record('user_oauth_account', $id);")
            .bind(("id", clean_record_id(id)))
            .await
            .map_err(|e| AppError::Database(format!("OAuth identity lookup failed: {e}")))?;
        let row: Option<Value> = response
            .take(0)
            .map_err(|e| AppError::Database(format!("OAuth identity decode failed: {e}")))?;
        row.map(parse_identity).transpose()
    }

    async fn list_for_user(&self, user_id: &UserId) -> Result<Vec<UserOAuthIdentity>> {
        let mut response = self
            .client
            .query("SELECT * FROM user_oauth_account WHERE user_id = type::record('user', $user_id) ORDER BY created_at;")
            .bind(("user_id", user_id.0.clone()))
            .await
            .map_err(|e| AppError::Database(format!("OAuth identity list failed: {e}")))?;
        let rows: Vec<Value> = response
            .take(0)
            .map_err(|e| AppError::Database(format!("OAuth identity decode failed: {e}")))?;
        rows.into_iter().map(parse_identity).collect()
    }

    async fn count_for_user(&self, user_id: &UserId) -> Result<usize> {
        let mut response = self
            .client
            .query("SELECT count() FROM user_oauth_account WHERE user_id = type::record('user', $user_id) GROUP ALL;")
            .bind(("user_id", user_id.0.clone()))
            .await
            .map_err(|e| AppError::Database(format!("OAuth identity count failed: {e}")))?;
        let rows: Vec<Value> = response.take(0).unwrap_or_default();
        Ok(rows
            .first()
            .and_then(|row| row.get("count"))
            .and_then(Value::as_u64)
            .unwrap_or(0) as usize)
    }

    async fn save(&self, identity: &UserOAuthIdentity) -> Result<()> {
        self.client
            .query(
                "UPSERT type::record('user_oauth_account', $id) SET
                    user_id = type::record('user', $user_id),
                    provider_key = $provider_key,
                    provider_user_id = $provider_user_id,
                    email = $email,
                    email_verified = $email_verified,
                    display_name = $display_name,
                    created_at = $created_at,
                    last_login_at = $last_login_at;",
            )
            .bind(("id", identity.id.clone()))
            .bind(("user_id", identity.user_id.0.clone()))
            .bind(("provider_key", identity.provider_key.clone()))
            .bind(("provider_user_id", identity.provider_user_id.clone()))
            .bind(("email", identity.email.clone()))
            .bind(("email_verified", identity.email_verified))
            .bind(("display_name", identity.display_name.clone()))
            .bind(("created_at", identity.created_at))
            .bind(("last_login_at", identity.last_login_at))
            .await
            .map_err(|e| AppError::Database(format!("Failed to save OAuth identity: {e}")))?
            .check()
            .map_err(|e| {
                if e.to_string().to_ascii_lowercase().contains("unique") {
                    AppError::Conflict("This provider account is already linked".into())
                } else {
                    AppError::Database(format!("Failed to save OAuth identity: {e}"))
                }
            })?;
        Ok(())
    }

    async fn touch_last_login(&self, id: &str) -> Result<()> {
        self.client
            .query(
                "UPDATE type::record('user_oauth_account', $id) SET last_login_at = time::now();",
            )
            .bind(("id", clean_record_id(id)))
            .await
            .map_err(|e| AppError::Database(format!("Failed to update OAuth identity: {e}")))?
            .check()
            .map_err(|e| AppError::Database(format!("Failed to update OAuth identity: {e}")))?;
        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<bool> {
        let mut response = self
            .client
            .query("DELETE type::record('user_oauth_account', $id) RETURN BEFORE;")
            .bind(("id", clean_record_id(id)))
            .await
            .map_err(|e| AppError::Database(format!("Failed to delete OAuth identity: {e}")))?;
        let rows: Vec<Value> = response.take(0).unwrap_or_default();
        Ok(!rows.is_empty())
    }
}

#[derive(Clone)]
pub struct SurrealOAuthFlowRepository {
    client: Arc<Db>,
}

impl SurrealOAuthFlowRepository {
    pub fn new(client: Arc<Db>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl OAuthFlowRepository for SurrealOAuthFlowRepository {
    async fn save(&self, flow: &OAuthFlowState) -> Result<()> {
        let link_user_id = flow.link_user_id.as_ref().map(|id| id.0.clone());
        self.client
            .query(
                "UPSERT type::record('oauth_state', $id) SET
                    state = $state,
                    provider_key = $provider_key,
                    purpose = $purpose,
                    link_user_id = (IF $link_user_id = NONE THEN NONE ELSE type::record('user', $link_user_id) END),
                    code_verifier = $code_verifier,
                    nonce = $nonce,
                    redirect_uri = $redirect_uri,
                    expires_at = type::datetime($expires_at),
                    used = false,
                    created_at = type::datetime($created_at);",
            )
            .bind(("id", flow.id.clone()))
            .bind(("state", flow.state.clone()))
            .bind(("provider_key", flow.provider_key.clone()))
            .bind(("purpose", flow.purpose.as_str()))
            .bind(("link_user_id", link_user_id))
            .bind((
                "code_verifier",
                flow.encrypted_code_verifier.clone(),
            ))
            .bind(("nonce", flow.nonce.clone()))
            .bind(("redirect_uri", flow.redirect_uri.clone()))
            .bind(("expires_at", flow.expires_at))
            .bind(("created_at", flow.created_at))
            .await
            .map_err(|e| AppError::Database(format!("Failed to store OAuth state: {e}")))?
            .check()
            .map_err(|e| AppError::Database(format!("Failed to store OAuth state: {e}")))?;
        Ok(())
    }

    async fn consume(&self, state: &str) -> Result<Option<OAuthFlowState>> {
        let mut response = self
            .client
            .query(
                "UPDATE oauth_state SET used = true
                 WHERE state = $state AND used = false AND expires_at > time::now()
                 RETURN AFTER;",
            )
            .bind(("state", state.to_string()))
            .await
            .map_err(|e| AppError::Database(format!("OAuth state consumption failed: {e}")))?;
        let rows: Vec<Value> = response
            .take(0)
            .map_err(|e| AppError::Database(format!("OAuth state decode failed: {e}")))?;
        rows.into_iter().next().map(parse_flow).transpose()
    }
}

fn parse_handoff(row: Value) -> Result<AuthHandoff> {
    let user_id = row
        .get("user_id")
        .map(|value| record_id(value, "user"))
        .filter(|id| !id.is_empty())
        .ok_or_else(|| AppError::Database("Auth handoff is missing user_id".into()))?;

    Ok(AuthHandoff {
        id: row
            .get("id")
            .map(|value| record_id(value, "auth_handoff"))
            .ok_or_else(|| AppError::Database("Auth handoff is missing id".into()))?,
        code_hash: required_string(&row, "code")?,
        user_id: UserId(user_id),
        encrypted_payload: required_string(&row, "encrypted_payload")?,
        expires_at: parse_datetime(&row, "expires_at")?,
        used: row.get("used").and_then(Value::as_bool).unwrap_or(false),
        created_at: parse_datetime(&row, "created_at")?,
    })
}

#[derive(Clone)]
pub struct SurrealAuthHandoffRepository {
    client: Arc<Db>,
}

impl SurrealAuthHandoffRepository {
    pub fn new(client: Arc<Db>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl AuthHandoffRepository for SurrealAuthHandoffRepository {
    async fn save(&self, handoff: &AuthHandoff) -> Result<()> {
        self.client
            .query(
                "UPSERT type::record('auth_handoff', $id) SET
                    code = $code,
                    user_id = type::record('user', $user_id),
                    encrypted_payload = $encrypted_payload,
                    expires_at = type::datetime($expires_at),
                    used = false,
                    created_at = type::datetime($created_at);",
            )
            .bind(("id", handoff.id.clone()))
            .bind(("code", handoff.code_hash.clone()))
            .bind(("user_id", handoff.user_id.0.clone()))
            .bind(("encrypted_payload", handoff.encrypted_payload.clone()))
            .bind(("expires_at", handoff.expires_at.to_rfc3339()))
            .bind(("created_at", handoff.created_at.to_rfc3339()))
            .await
            .map_err(|e| AppError::Database(format!("Failed to store auth handoff: {e}")))?
            .check()
            .map_err(|e| AppError::Database(format!("Failed to store auth handoff: {e}")))?;
        Ok(())
    }

    async fn consume_by_hash(&self, code_hash: &str) -> Result<Option<AuthHandoff>> {
        let mut response = self
            .client
            .query(
                "UPDATE auth_handoff SET used = true
                 WHERE code = $code AND used = false AND expires_at > time::now()
                 RETURN AFTER;",
            )
            .bind(("code", code_hash.to_string()))
            .await
            .map_err(|e| AppError::Database(format!("Auth handoff consumption failed: {e}")))?;
        let rows: Vec<Value> = response
            .take(0)
            .map_err(|e| AppError::Database(format!("Auth handoff decode failed: {e}")))?;
        rows.into_iter().next().map(parse_handoff).transpose()
    }
}
