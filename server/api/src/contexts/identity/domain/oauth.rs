use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chrono::{DateTime, Duration, Utc};
use rand::{distributions::Alphanumeric, Rng, RngCore};
use serde::{Deserialize, Serialize};
use subtle::ConstantTimeEq;
use utoipa::ToSchema;

use crate::contexts::identity::domain::UserId;
use crate::shared::error::{AppError, Result};

/// Marker stored in `user.password_hash` for accounts provisioned by an
/// external identity. Argon2 parsing fails for this value, so password login
/// is impossible until a real password is explicitly set.
pub const OAUTH_ONLY_PASSWORD_MARKER: &str = "!oauth-only-no-password!";

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum OAuthProviderType {
    Google,
    Github,
    Facebook,
    Oidc,
    #[default]
    #[serde(rename = "oauth2")]
    OAuth2,
}

impl OAuthProviderType {
    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "google" => Some(Self::Google),
            "github" => Some(Self::Github),
            "facebook" => Some(Self::Facebook),
            "oidc" => Some(Self::Oidc),
            "oauth2" => Some(Self::OAuth2),
            _ => None,
        }
    }

    /// Provider types that require OIDC discovery and ID-token validation.
    pub fn is_oidc(self) -> bool {
        matches!(self, Self::Google | Self::Oidc)
    }

    pub fn default_issuer(self) -> Option<&'static str> {
        match self {
            Self::Google => Some("https://accounts.google.com"),
            _ => None,
        }
    }

    pub fn default_scopes(self) -> Vec<String> {
        match self {
            Self::Google => vec!["openid", "email", "profile"],
            Self::Github => vec!["read:user", "user:email"],
            Self::Facebook => vec!["email", "public_profile"],
            Self::Oidc => vec!["openid", "email", "profile"],
            Self::OAuth2 => vec!["email", "profile"],
        }
        .into_iter()
        .map(str::to_string)
        .collect()
    }

    pub fn default_authorization_endpoint(self) -> Option<&'static str> {
        match self {
            Self::Github => Some("https://github.com/login/oauth/authorize"),
            Self::Facebook => Some("https://www.facebook.com/v19.0/dialog/oauth"),
            _ => None,
        }
    }

    pub fn default_token_endpoint(self) -> Option<&'static str> {
        match self {
            Self::Github => Some("https://github.com/login/oauth/access_token"),
            Self::Facebook => Some("https://graph.facebook.com/v19.0/oauth/access_token"),
            _ => None,
        }
    }

    pub fn default_userinfo_endpoint(self) -> Option<&'static str> {
        match self {
            Self::Github => Some("https://api.github.com/user"),
            Self::Facebook => Some("https://graph.facebook.com/me"),
            _ => None,
        }
    }

    pub fn default_email_endpoint(self) -> Option<&'static str> {
        match self {
            Self::Github => Some("https://api.github.com/user/emails"),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum OAuthIntent {
    #[default]
    Login,
    Link,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum OAuthFlowPurpose {
    #[default]
    Login,
    Link,
}

impl From<OAuthIntent> for OAuthFlowPurpose {
    fn from(intent: OAuthIntent) -> Self {
        match intent {
            OAuthIntent::Login => Self::Login,
            OAuthIntent::Link => Self::Link,
        }
    }
}

impl OAuthFlowPurpose {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Login => "login",
            Self::Link => "link",
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct OAuthProviderConfig {
    pub provider_key: String,
    pub name: String,
    pub provider_type: OAuthProviderType,
    pub issuer: Option<String>,
    pub client_id: String,
    /// AES-GCM ciphertext; never serialized or logged in plaintext.
    pub encrypted_client_secret: String,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
    pub authorization_endpoint: Option<String>,
    pub token_endpoint: Option<String>,
    pub userinfo_endpoint: Option<String>,
    pub email_endpoint: Option<String>,
    pub jwks_uri: Option<String>,
    pub subject_claim: Option<String>,
    pub email_claim: Option<String>,
    pub name_claim: Option<String>,
    pub email_verified_claim: Option<String>,
    pub is_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl std::fmt::Debug for OAuthProviderConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OAuthProviderConfig")
            .field("provider_key", &self.provider_key)
            .field("name", &self.name)
            .field("provider_type", &self.provider_type)
            .field("issuer", &self.issuer)
            .field("client_id", &self.client_id)
            .field("encrypted_client_secret", &"[REDACTED]")
            .field("redirect_uri", &self.redirect_uri)
            .field("scopes", &self.scopes)
            .field("authorization_endpoint", &self.authorization_endpoint)
            .field("token_endpoint", &self.token_endpoint)
            .field("userinfo_endpoint", &self.userinfo_endpoint)
            .field("email_endpoint", &self.email_endpoint)
            .field("jwks_uri", &self.jwks_uri)
            .field("subject_claim", &self.subject_claim)
            .field("email_claim", &self.email_claim)
            .field("name_claim", &self.name_claim)
            .field("email_verified_claim", &self.email_verified_claim)
            .field("is_enabled", &self.is_enabled)
            .field("created_at", &self.created_at)
            .field("updated_at", &self.updated_at)
            .finish()
    }
}

impl OAuthProviderConfig {
    pub fn is_oidc(&self) -> bool {
        self.provider_type.is_oidc()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthFlowState {
    pub id: String,
    pub state: String,
    pub provider_key: String,
    pub purpose: OAuthFlowPurpose,
    pub link_user_id: Option<UserId>,
    /// AES-GCM ciphertext of the PKCE verifier.
    pub encrypted_code_verifier: String,
    pub nonce: Option<String>,
    pub redirect_uri: String,
    pub expires_at: DateTime<Utc>,
    pub used: bool,
    pub created_at: DateTime<Utc>,
}

impl OAuthFlowState {
    pub fn new(
        provider_key: String,
        purpose: OAuthFlowPurpose,
        link_user_id: Option<UserId>,
        encrypted_code_verifier: String,
        nonce: Option<String>,
        redirect_uri: String,
        ttl_seconds: i64,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            state: generate_state_token(),
            provider_key,
            purpose,
            link_user_id,
            encrypted_code_verifier,
            nonce,
            redirect_uri,
            expires_at: now + Duration::seconds(ttl_seconds),
            used: false,
            created_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserOAuthIdentity {
    pub id: String,
    pub user_id: UserId,
    pub provider_key: String,
    pub provider_user_id: String,
    pub email: Option<String>,
    pub email_verified: bool,
    pub display_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_login_at: DateTime<Utc>,
}

impl UserOAuthIdentity {
    pub fn new(
        user_id: UserId,
        provider_key: String,
        provider_user_id: String,
        email: Option<String>,
        email_verified: bool,
        display_name: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            user_id,
            provider_key,
            provider_user_id,
            email,
            email_verified,
            display_name,
            created_at: now,
            last_login_at: now,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalIdentity {
    pub subject: String,
    pub email: Option<String>,
    pub email_verified: bool,
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthHandoff {
    pub id: String,
    pub code_hash: String,
    pub user_id: UserId,
    /// AES-GCM ciphertext of the serialized exchange payload.
    pub encrypted_payload: String,
    pub expires_at: DateTime<Utc>,
    pub used: bool,
    pub created_at: DateTime<Utc>,
}

impl AuthHandoff {
    pub fn new(
        user_id: UserId,
        code_hash: String,
        encrypted_payload: String,
        ttl_seconds: i64,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            code_hash,
            user_id,
            encrypted_payload,
            expires_at: now + Duration::seconds(ttl_seconds),
            used: false,
            created_at: now,
        }
    }
}

/// Generates a one-use handoff code with 256 bits of entropy.
pub fn generate_handoff_code() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

pub fn hash_handoff_code(code: &str) -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(code.as_bytes()))
}

/// Generates an opaque, URL-safe state token with 256 bits of entropy.
pub fn generate_state_token() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

/// RFC 7636 verifier: 64 unreserved characters.
pub fn generate_code_verifier() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(64)
        .map(char::from)
        .collect()
}

/// S256 challenge: base64url(sha256(verifier)) without padding.
pub fn code_challenge_s256(verifier: &str) -> String {
    use sha2::{Digest, Sha256};
    URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()))
}

pub fn verify_nonce(expected: &str, actual: Option<&str>) -> bool {
    match actual {
        Some(actual) if actual.len() == expected.len() => {
            expected.as_bytes().ct_eq(actual.as_bytes()).into()
        }
        _ => false,
    }
}

pub fn normalize_issuer(raw: &str) -> String {
    raw.trim().trim_end_matches('/').to_string()
}

/// Validates a redirect URI that is only ever used for front-channel browser
/// redirects (never fetched by the server).
pub fn validate_redirect_uri(raw: &str) -> Result<()> {
    let url = reqwest::Url::parse(raw)
        .map_err(|_| AppError::Validation("redirect_uri is not a valid URL".into()))?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(AppError::Validation(
            "redirect_uri must use http or https".into(),
        ));
    }
    if url.host_str().is_none() {
        return Err(AppError::Validation(
            "redirect_uri must include a host".into(),
        ));
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err(AppError::Validation(
            "redirect_uri must not embed credentials".into(),
        ));
    }
    if url.fragment().is_some() {
        return Err(AppError::Validation(
            "redirect_uri must not contain a fragment".into(),
        ));
    }
    Ok(())
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct IdTokenClaims {
    pub iss: String,
    pub aud: AudienceClaim,
    pub sub: String,
    pub exp: i64,
    #[serde(default)]
    pub iat: i64,
    #[serde(default)]
    pub nonce: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub email_verified: Option<serde_json::Value>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default, rename = "preferred_username")]
    pub preferred_username: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum AudienceClaim {
    Single(String),
    Multiple(Vec<String>),
}

impl AudienceClaim {
    pub fn contains(&self, client_id: &str) -> bool {
        match self {
            Self::Single(value) => value == client_id,
            Self::Multiple(values) => values.iter().any(|value| value == client_id),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedIdToken {
    pub subject: String,
    pub email: Option<String>,
    pub email_verified: bool,
    pub display_name: Option<String>,
}

/// Validates the security-critical claims of an OIDC ID token. Signature
/// verification happens separately against the provider JWKS.
pub fn validate_id_token_claims(
    claims: &IdTokenClaims,
    expected_issuer: &str,
    expected_audience: &str,
    expected_nonce: &str,
    now: DateTime<Utc>,
    leeway_seconds: i64,
) -> Result<ValidatedIdToken> {
    if normalize_issuer(&claims.iss) != normalize_issuer(expected_issuer) {
        return Err(AppError::Unauthorized(
            "OIDC ID token issuer mismatch".into(),
        ));
    }
    if !claims.aud.contains(expected_audience) {
        return Err(AppError::Unauthorized(
            "OIDC ID token audience mismatch".into(),
        ));
    }
    if claims.sub.trim().is_empty() {
        return Err(AppError::Unauthorized(
            "OIDC ID token subject is missing".into(),
        ));
    }
    if claims.exp <= (now - Duration::seconds(leeway_seconds)).timestamp() {
        return Err(AppError::Unauthorized("OIDC ID token has expired".into()));
    }
    if !verify_nonce(expected_nonce, claims.nonce.as_deref()) {
        return Err(AppError::Unauthorized(
            "OIDC ID token nonce mismatch".into(),
        ));
    }

    Ok(ValidatedIdToken {
        subject: claims.sub.clone(),
        email: claims
            .email
            .as_ref()
            .map(|email| email.trim().to_lowercase()),
        email_verified: json_bool(claims.email_verified.as_ref()),
        display_name: claims
            .name
            .clone()
            .or_else(|| claims.preferred_username.clone()),
    })
}

pub fn json_bool(value: Option<&serde_json::Value>) -> bool {
    match value {
        Some(serde_json::Value::Bool(value)) => *value,
        Some(serde_json::Value::String(value)) => value.eq_ignore_ascii_case("true"),
        _ => false,
    }
}

pub fn claim_string(value: Option<&serde_json::Value>) -> Option<String> {
    match value? {
        serde_json::Value::String(value) if !value.trim().is_empty() => {
            Some(value.trim().to_string())
        }
        serde_json::Value::Number(value) => Some(value.to_string()),
        _ => None,
    }
}

pub fn claim_bool(value: Option<&serde_json::Value>, default: bool) -> bool {
    match value {
        Some(serde_json::Value::Bool(value)) => *value,
        Some(serde_json::Value::String(value)) => value.eq_ignore_ascii_case("true"),
        Some(serde_json::Value::Number(value)) => value.as_i64().is_some_and(|v| v != 0),
        _ => default,
    }
}

/// Rejects control characters in claim-derived display values before they are
/// persisted or rendered.
pub fn sanitize_display_name(value: Option<String>) -> Option<String> {
    value
        .map(|value| {
            value
                .chars()
                .filter(|c| !c.is_control())
                .take(120)
                .collect::<String>()
                .trim()
                .to_string()
        })
        .filter(|value| !value.is_empty())
}

pub fn optional_trimmed(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}
