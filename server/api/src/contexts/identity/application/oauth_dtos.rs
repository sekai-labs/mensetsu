use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::contexts::identity::application::dtos::UserResponseDto;
use crate::contexts::identity::domain::oauth::{OAuthProviderType, UserOAuthIdentity};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OAuthProviderSummaryDto {
    pub provider_key: String,
    pub name: String,
    pub provider_type: OAuthProviderType,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OAuthProviderAdminDto {
    pub provider_key: String,
    pub name: String,
    pub provider_type: OAuthProviderType,
    pub issuer: Option<String>,
    pub client_id: String,
    /// Never contains the client secret; only whether one is configured.
    pub has_client_secret: bool,
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

/// Admin provider update. Every field is optional: omitted fields preserve the
/// stored value, while an explicit empty string clears an optional field.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct OAuthProviderUpsertRequestDto {
    pub name: Option<String>,
    pub provider_type: Option<OAuthProviderType>,
    pub issuer: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub redirect_uri: Option<String>,
    pub scopes: Option<Vec<String>>,
    pub authorization_endpoint: Option<String>,
    pub token_endpoint: Option<String>,
    pub userinfo_endpoint: Option<String>,
    pub email_endpoint: Option<String>,
    pub jwks_uri: Option<String>,
    pub subject_claim: Option<String>,
    pub email_claim: Option<String>,
    pub name_claim: Option<String>,
    pub email_verified_claim: Option<String>,
    pub is_enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OAuthIdentityDto {
    pub id: String,
    pub provider_key: String,
    pub subject: String,
    pub email: Option<String>,
    pub email_verified: bool,
    pub display_name: Option<String>,
    pub linked_at: DateTime<Utc>,
    pub last_login_at: DateTime<Utc>,
}

impl From<UserOAuthIdentity> for OAuthIdentityDto {
    fn from(identity: UserOAuthIdentity) -> Self {
        Self {
            id: identity.id,
            provider_key: identity.provider_key,
            subject: identity.provider_user_id,
            email: identity.email,
            email_verified: identity.email_verified,
            display_name: identity.display_name,
            linked_at: identity.created_at,
            last_login_at: identity.last_login_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OAuthLinkStartResponseDto {
    pub authorization_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OAuthExchangeRequestDto {
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OAuthExchangeResponseDto {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<UserResponseDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub requires_2fa: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temp_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linked_identity: Option<OAuthIdentityDto>,
}

#[derive(Debug, Clone, Default, Deserialize, ToSchema, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct OAuthCallbackQuery {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
}
