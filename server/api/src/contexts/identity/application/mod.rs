pub mod dtos;

use std::sync::Arc;
use chrono::Utc;
use validator::Validate;

use dtos::{
    AuthResponseDto, Login2faRequestDto, LoginRequestDto, LoginResponseDto, RegisterRequestDto,
    TokenResponseDto, TotpSetupResponseDto, UserResponseDto,
};
use crate::contexts::identity::domain::{
    hasher::PasswordHasher,
    repository::{AuthSessionRepository, UserRepository},
    token::TokenService,
    AuthSession, Email, SessionId, User, UserId, UserStatus,
};
use crate::shared::error::{AppError, Result};

pub struct IdentityService {
    user_repo: Arc<dyn UserRepository>,
    session_repo: Arc<dyn AuthSessionRepository>,
    hasher: Arc<dyn PasswordHasher>,
    token_service: TokenService,
}

impl IdentityService {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        session_repo: Arc<dyn AuthSessionRepository>,
        hasher: Arc<dyn PasswordHasher>,
        token_service: TokenService,
    ) -> Self {
        Self {
            user_repo,
            session_repo,
            hasher,
            token_service,
        }
    }

    pub async fn register(
        &self,
        dto: RegisterRequestDto,
        user_agent: Option<String>,
        ip_address: Option<String>,
    ) -> Result<AuthResponseDto> {
        dto.validate()
            .map_err(|e| AppError::Validation(e.to_string()))?;

        let email = Email::new(dto.email)?;

        if self.user_repo.find_by_email(&email).await?.is_some() {
            return Err(AppError::Conflict(format!("Email {} already registered", email.0)));
        }

        let password_hash = self.hasher.hash_password(&dto.password)?;
        let admin_emails = std::env::var("ADMIN_EMAILS").unwrap_or_default();
        let is_admin = if !admin_emails.is_empty() {
            admin_emails
                .split(',')
                .any(|e| e.trim().eq_ignore_ascii_case(&email.0))
        } else {
            
            email.0.starts_with("admin@") || email.0.starts_with("admin_")
        };
        let role = if is_admin {
            crate::contexts::identity::domain::UserRole::Admin
        } else {
            crate::contexts::identity::domain::UserRole::Member
        };

        let user = User::new(email, dto.name, password_hash, role)?;
        self.user_repo.save(&user).await?;

        let refresh_token = TokenService::generate_refresh_token();
        let refresh_hash = TokenService::hash_refresh_token(&refresh_token);

        let session = AuthSession::new(
            user.id.clone(),
            refresh_hash,
            self.token_service.refresh_ttl_seconds(),
            user_agent,
            ip_address,
        );
        self.session_repo.save(&session).await?;

        let access_token = self
            .token_service
            .generate_access_token(&user.id, &session.id)?;

        Ok(AuthResponseDto {
            user: user.into(),
            access_token,
            refresh_token,
        })
    }

    pub async fn login(
        &self,
        dto: LoginRequestDto,
        user_agent: Option<String>,
        ip_address: Option<String>,
    ) -> Result<LoginResponseDto> {
        dto.validate()
            .map_err(|e| AppError::Validation(e.to_string()))?;

        let email = Email::new(dto.email)?;
        let user = self
            .user_repo
            .find_by_email(&email)
            .await?
            .ok_or_else(|| AppError::Unauthorized("Invalid email or password".into()))?;

        if user.status != UserStatus::Active {
            return Err(AppError::Unauthorized("Account is suspended".into()));
        }

        let valid = self.hasher.verify_password(&dto.password, &user.password_hash)?;
        if !valid {
            return Err(AppError::Unauthorized("Invalid email or password".into()));
        }

        if user.totp_enabled {
            let temp_token = self.token_service.generate_temp_2fa_token(&user.id)?;
            return Ok(LoginResponseDto {
                user: None,
                access_token: None,
                refresh_token: None,
                requires_2fa: true,
                temp_token: Some(temp_token),
            });
        }

        let refresh_token = TokenService::generate_refresh_token();
        let refresh_hash = TokenService::hash_refresh_token(&refresh_token);

        let session = AuthSession::new(
            user.id.clone(),
            refresh_hash,
            self.token_service.refresh_ttl_seconds(),
            user_agent,
            ip_address,
        );
        self.session_repo.save(&session).await?;

        let access_token = self
            .token_service
            .generate_access_token(&user.id, &session.id)?;

        Ok(LoginResponseDto {
            user: Some(user.into()),
            access_token: Some(access_token),
            refresh_token: Some(refresh_token),
            requires_2fa: false,
            temp_token: None,
        })
    }

    pub async fn login_2fa(
        &self,
        dto: Login2faRequestDto,
        user_agent: Option<String>,
        ip_address: Option<String>,
    ) -> Result<AuthResponseDto> {
        let user_id = self.token_service.verify_temp_2fa_token(&dto.temp_token)?;
        let user = self
            .user_repo
            .find_by_id(&user_id)
            .await?
            .ok_or_else(|| AppError::Unauthorized("User not found".into()))?;

        let totp_secret = user
            .totp_secret
            .as_deref()
            .ok_or_else(|| AppError::Unauthorized("2FA not configured".into()))?;

        let secret = totp_rs::Secret::try_from_base32(totp_secret)
            .map_err(|e| AppError::Internal(format!("Invalid base32 secret: {e}")))?;

        let totp = totp_rs::Builder::new()
            .with_secret(secret.as_bytes().to_vec())
            .with_issuer(Some("Mensetsu"))
            .with_account_name(user.email.as_str())
            .build()
            .map_err(|e| AppError::Internal(format!("TOTP init error: {e}")))?;

        if totp.check_current(&dto.code).is_none() {
            return Err(AppError::Unauthorized("Invalid 2FA authentication code".into()));
        }

        let refresh_token = TokenService::generate_refresh_token();
        let refresh_hash = TokenService::hash_refresh_token(&refresh_token);

        let session = AuthSession::new(
            user.id.clone(),
            refresh_hash,
            self.token_service.refresh_ttl_seconds(),
            user_agent,
            ip_address,
        );
        self.session_repo.save(&session).await?;

        let access_token = self
            .token_service
            .generate_access_token(&user.id, &session.id)?;

        Ok(AuthResponseDto {
            user: user.into(),
            access_token,
            refresh_token,
        })
    }

    pub async fn setup_totp(&self, user_id: &UserId) -> Result<TotpSetupResponseDto> {
        let mut user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("User {user_id} not found")))?;

        let secret = totp_rs::Secret::generate();
        let secret_str = secret.to_base32();

        let totp = totp_rs::Builder::new()
            .with_secret(secret.as_bytes().to_vec())
            .with_issuer(Some("Mensetsu"))
            .with_account_name(user.email.as_str())
            .build()
            .map_err(|e| AppError::Internal(format!("Failed to configure TOTP: {e}")))?;

        let qr_code_data_url = totp
            .to_qr_base64()
            .map_err(|e| AppError::Internal(format!("Failed to generate QR code: {e}")))?;
        let qr_code_data_url = format!("data:image/png;base64,{qr_code_data_url}");
        let otpauth_url = totp
            .to_url()
            .map_err(|e| AppError::Internal(format!("Failed to generate TOTP url: {e}")))?;

        user.totp_secret = Some(secret_str.clone());
        self.user_repo.save(&user).await?;

        Ok(TotpSetupResponseDto {
            secret: secret_str,
            qr_code_data_url,
            otpauth_url,
        })
    }

    pub async fn verify_and_enable_totp(&self, user_id: &UserId, code: &str) -> Result<UserResponseDto> {
        let mut user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("User {user_id} not found")))?;

        let secret_str = user
            .totp_secret
            .as_deref()
            .ok_or_else(|| AppError::Validation("TOTP setup not initiated. Call setup first.".into()))?;

        let secret = totp_rs::Secret::try_from_base32(secret_str)
            .map_err(|e| AppError::Internal(format!("Invalid base32 secret: {e}")))?;

        let totp = totp_rs::Builder::new()
            .with_secret(secret.as_bytes().to_vec())
            .with_issuer(Some("Mensetsu"))
            .with_account_name(user.email.as_str())
            .build()
            .map_err(|e| AppError::Internal(format!("TOTP error: {e}")))?;

        if totp.check_current(code).is_none() {
            return Err(AppError::Unauthorized("Invalid TOTP verification code".into()));
        }
        user.totp_enabled = true;
        user.updated_at = Utc::now();
        self.user_repo.save(&user).await?;
        Ok(user.into())
    }

    pub async fn disable_totp(&self, user_id: &UserId, code: &str) -> Result<UserResponseDto> {
        let mut user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("User {user_id} not found")))?;

        if !user.totp_enabled {
            return Ok(user.into());
        }

        let secret_str = user
            .totp_secret
            .as_deref()
            .ok_or_else(|| AppError::Validation("2FA secret missing".into()))?;

        let secret = totp_rs::Secret::try_from_base32(secret_str)
            .map_err(|e| AppError::Internal(format!("Invalid base32 secret: {e}")))?;

        let totp = totp_rs::Builder::new()
            .with_secret(secret.as_bytes().to_vec())
            .with_issuer(Some("Mensetsu"))
            .with_account_name(user.email.as_str())
            .build()
            .map_err(|e| AppError::Internal(format!("TOTP error: {e}")))?;

        if totp.check_current(code).is_none() {
            return Err(AppError::Unauthorized("Invalid 2FA verification code".into()));
        }
        user.totp_enabled = false;
        user.totp_secret = None;
        user.updated_at = Utc::now();
        self.user_repo.save(&user).await?;

        Ok(user.into())
    }

    pub async fn refresh(
        &self,
        raw_refresh_token: &str,
    ) -> Result<TokenResponseDto> {
        let token_hash = TokenService::hash_refresh_token(raw_refresh_token);
        let session = self
            .session_repo
            .find_by_token_hash(&token_hash)
            .await?
            .ok_or_else(|| AppError::Unauthorized("Invalid or expired refresh token".into()))?;

        if !session.is_active() {
            return Err(AppError::Unauthorized("Session expired or revoked".into()));
        }

        let user = self
            .user_repo
            .find_by_id(&session.user_id)
            .await?
            .ok_or_else(|| AppError::Unauthorized("User not found".into()))?;

        if user.status != UserStatus::Active {
            return Err(AppError::Unauthorized("Account is suspended".into()));
        }

        
        let new_refresh_token = TokenService::generate_refresh_token();
        let new_refresh_hash = TokenService::hash_refresh_token(&new_refresh_token);
        let new_expires_at = Utc::now() + chrono::Duration::seconds(self.token_service.refresh_ttl_seconds());

        self.session_repo
            .update_token_hash(&session.id, &new_refresh_hash, new_expires_at)
            .await?;

        let access_token = self
            .token_service
            .generate_access_token(&user.id, &session.id)?;

        Ok(TokenResponseDto {
            access_token,
            refresh_token: new_refresh_token,
        })
    }

    pub async fn logout(&self, session_id: &SessionId) -> Result<()> {
        self.session_repo.revoke(session_id).await
    }

    pub async fn logout_all(&self, user_id: &UserId) -> Result<()> {
        self.session_repo.revoke_all_for_user(user_id).await
    }

    pub async fn get_user_by_id(&self, user_id: &UserId) -> Result<UserResponseDto> {
        let user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("User {} not found", user_id.0)))?;

        Ok(user.into())
    }

    pub fn token_service(&self) -> &TokenService {
        &self.token_service
    }

    pub fn session_repo(&self) -> &Arc<dyn AuthSessionRepository> {
        &self.session_repo
    }
}
