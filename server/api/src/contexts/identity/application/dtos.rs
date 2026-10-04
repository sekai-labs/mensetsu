use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::contexts::identity::domain::{User, UserRole, UserStatus};

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct RegisterRequestDto {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
    #[validate(length(min = 2, message = "Name must be at least 2 characters"))]
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct LoginRequestDto {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RefreshTokenRequestDto {
    pub refresh_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserResponseDto {
    pub id: String,
    pub email: String,
    pub name: String,
    pub status: UserStatus,
    pub role: UserRole,
    pub totp_enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<User> for UserResponseDto {
    fn from(user: User) -> Self {
        Self {
            id: user.id.0,
            email: user.email,
            name: user.name,
            status: user.status,
            role: user.role,
            totp_enabled: user.totp_enabled,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LoginResponseDto {
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
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct Login2faRequestDto {
    pub temp_token: String,
    #[validate(length(min = 6, max = 8, message = "Code must be 6 digits"))]
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TotpSetupResponseDto {
    pub secret: String,
    pub qr_code_data_url: String,
    pub otpauth_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct VerifyTotpRequestDto {
    #[validate(length(min = 6, max = 8, message = "Code must be 6 digits"))]
    pub code: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AuthResponseDto {
    pub user: UserResponseDto,
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TokenResponseDto {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MessageResponseDto {
    pub message: String,
}
