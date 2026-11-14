use std::sync::Arc;

use chrono::Utc;
use subtle::ConstantTimeEq;

use crate::contexts::identity::application::dtos::UserResponseDto;
use crate::contexts::identity::application::oauth_dtos::{
    OAuthExchangeResponseDto, OAuthIdentityDto, OAuthProviderAdminDto, OAuthProviderSummaryDto,
    OAuthProviderUpsertRequestDto,
};
use crate::contexts::identity::domain::oauth::{
    claim_bool, claim_string, generate_code_verifier, generate_handoff_code, hash_handoff_code,
    optional_trimmed, sanitize_display_name, validate_redirect_uri, AuthHandoff, ExternalIdentity,
    OAuthFlowPurpose, OAuthFlowState, OAuthIntent, OAuthProviderConfig, OAuthProviderType,
    UserOAuthIdentity, OAUTH_ONLY_PASSWORD_MARKER,
};
use crate::contexts::identity::domain::repository::{
    AuthHandoffRepository, AuthSessionRepository, OAuthFlowRepository, OAuthProviderRepository,
    UserOAuthIdentityRepository, UserRepository,
};
use crate::contexts::identity::domain::{
    token::TokenService, AuthSession, Email, User, UserId, UserStatus,
};
use crate::contexts::identity::infrastructure::oauth_http::{
    OAuthProviderClient, ResolvedOAuthEndpoints, TokenEndpointResponse,
};
use crate::shared::encryption::CryptoService;
use crate::shared::error::{AppError, Result};

pub struct AuthorizationStart {
    pub authorization_url: String,
    pub state: String,
    pub expires_in_seconds: i64,
}

pub struct CallbackRedirect {
    pub location: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallbackErrorCode {
    Denied,
    StateInvalid,
    EmailUnverified,
    AccountExists,
    LastSignInMethod,
    OAuthFailed,
}

impl CallbackErrorCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Denied => "oauth_denied",
            Self::StateInvalid => "state_invalid",
            Self::EmailUnverified => "email_unverified",
            Self::AccountExists => "account_exists",
            Self::LastSignInMethod => "last_sign_in_method",
            Self::OAuthFailed => "oauth_failed",
        }
    }
}

pub struct OAuthService {
    provider_repo: Arc<dyn OAuthProviderRepository>,
    identity_repo: Arc<dyn UserOAuthIdentityRepository>,
    flow_repo: Arc<dyn OAuthFlowRepository>,
    handoff_repo: Arc<dyn AuthHandoffRepository>,
    user_repo: Arc<dyn UserRepository>,
    session_repo: Arc<dyn AuthSessionRepository>,
    token_service: TokenService,
    crypto: CryptoService,
    provider_client: Arc<dyn OAuthProviderClient>,
    state_ttl_seconds: i64,
    handoff_ttl_seconds: i64,
    web_callback_url: Option<String>,
}

impl OAuthService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        provider_repo: Arc<dyn OAuthProviderRepository>,
        identity_repo: Arc<dyn UserOAuthIdentityRepository>,
        flow_repo: Arc<dyn OAuthFlowRepository>,
        handoff_repo: Arc<dyn AuthHandoffRepository>,
        user_repo: Arc<dyn UserRepository>,
        session_repo: Arc<dyn AuthSessionRepository>,
        token_service: TokenService,
        crypto: CryptoService,
        provider_client: Arc<dyn OAuthProviderClient>,
        state_ttl_seconds: i64,
        handoff_ttl_seconds: i64,
        web_callback_url: Option<String>,
    ) -> Self {
        Self {
            provider_repo,
            identity_repo,
            flow_repo,
            handoff_repo,
            user_repo,
            session_repo,
            token_service,
            crypto,
            provider_client,
            state_ttl_seconds,
            handoff_ttl_seconds,
            web_callback_url,
        }
    }

    pub async fn list_public_providers(&self) -> Result<Vec<OAuthProviderSummaryDto>> {
        Ok(self
            .provider_repo
            .list_enabled()
            .await?
            .into_iter()
            .map(|provider| OAuthProviderSummaryDto {
                provider_key: provider.provider_key,
                name: provider.name,
                provider_type: provider.provider_type,
            })
            .collect())
    }

    pub async fn begin_login(&self, provider_key: &str) -> Result<AuthorizationStart> {
        self.begin_flow(provider_key, OAuthIntent::Login, None)
            .await
    }

    pub async fn begin_link(
        &self,
        provider_key: &str,
        user_id: &UserId,
    ) -> Result<AuthorizationStart> {
        self.begin_flow(provider_key, OAuthIntent::Link, Some(user_id.clone()))
            .await
    }

    async fn begin_flow(
        &self,
        provider_key: &str,
        intent: OAuthIntent,
        link_user_id: Option<UserId>,
    ) -> Result<AuthorizationStart> {
        let provider = self.require_enabled_provider(provider_key).await?;
        if matches!(intent, OAuthIntent::Link) && link_user_id.is_none() {
            return Err(AppError::Unauthorized(
                "Linking requires an authenticated user".into(),
            ));
        }

        validate_redirect_uri(&provider.redirect_uri)?;
        let endpoints = self.provider_client.resolve_endpoints(&provider).await?;

        let verifier = generate_code_verifier();
        let challenge = crate::contexts::identity::domain::oauth::code_challenge_s256(&verifier);
        let encrypted_verifier = self.crypto.encrypt(&verifier)?;
        let nonce = if provider.is_oidc() {
            Some(crate::contexts::identity::domain::oauth::generate_state_token())
        } else {
            None
        };

        let flow = OAuthFlowState::new(
            provider.provider_key.clone(),
            intent.into(),
            link_user_id,
            encrypted_verifier,
            nonce.clone(),
            provider.redirect_uri.clone(),
            self.state_ttl_seconds,
        );

        let mut url = reqwest::Url::parse(&endpoints.authorization_endpoint)
            .map_err(|_| AppError::Validation("authorization_endpoint is invalid".into()))?;
        {
            let mut query = url.query_pairs_mut();
            query.append_pair("response_type", "code");
            query.append_pair("client_id", &provider.client_id);
            query.append_pair("redirect_uri", &provider.redirect_uri);
            query.append_pair("scope", &provider.scopes.join(" "));
            query.append_pair("state", &flow.state);
            query.append_pair("code_challenge", &challenge);
            query.append_pair("code_challenge_method", "S256");
            if let Some(nonce) = &nonce {
                query.append_pair("nonce", nonce);
            }
        }

        self.flow_repo.save(&flow).await?;

        Ok(AuthorizationStart {
            authorization_url: url.to_string(),
            state: flow.state,
            expires_in_seconds: self.state_ttl_seconds,
        })
    }

    /// Handles the provider's browser callback. Returns a redirect to the web
    /// BFF carrying either a single-use exchange code or a stable error code.
    /// Access tokens and API JWTs are never placed in the URL.
    pub async fn handle_callback(
        &self,
        code: Option<&str>,
        state: Option<&str>,
        provider_error: Option<&str>,
        state_cookie: Option<&str>,
        user_agent: Option<String>,
        ip_address: Option<String>,
    ) -> Result<CallbackRedirect> {
        let state = match state {
            Some(state) if !state.is_empty() => state,
            _ => return self.error_redirect(CallbackErrorCode::StateInvalid),
        };

        let flow = match self.flow_repo.consume(state).await? {
            Some(flow) => flow,
            None => return self.error_redirect(CallbackErrorCode::StateInvalid),
        };

        if flow.provider_key.is_empty() {
            return self.error_redirect(CallbackErrorCode::StateInvalid);
        }

        if flow.purpose == OAuthFlowPurpose::Login {
            let cookie = match state_cookie {
                Some(cookie) if !cookie.is_empty() => cookie,
                _ => return self.error_redirect(CallbackErrorCode::StateInvalid),
            };
            if !constant_time_eq(cookie, state) {
                return self.error_redirect(CallbackErrorCode::StateInvalid);
            }
        }

        if let Some(error) = provider_error.filter(|error| !error.is_empty()) {
            let code = if error == "access_denied" {
                CallbackErrorCode::Denied
            } else {
                CallbackErrorCode::OAuthFailed
            };
            return self.error_redirect(code);
        }

        let code = match code.filter(|code| !code.is_empty()) {
            Some(code) => code,
            None => return self.error_redirect(CallbackErrorCode::OAuthFailed),
        };

        let provider = match self.provider_repo.find_by_key(&flow.provider_key).await? {
            Some(provider) if provider.is_enabled => provider,
            _ => return self.error_redirect(CallbackErrorCode::OAuthFailed),
        };

        let (payload, user_id) = match self
            .complete_callback(&provider, &flow, code, user_agent, ip_address)
            .await
        {
            Ok(result) => result,
            Err(error) => {
                if let Some(code) = callback_error_code(&error) {
                    tracing::warn!(
                        provider = %provider.provider_key,
                        error = %error,
                        "OAuth callback rejected"
                    );
                    return self.error_redirect(code);
                }
                return Err(error);
            }
        };

        let handoff_code = generate_handoff_code();
        let serialized = serde_json::to_vec(&payload)
            .map_err(|e| AppError::Internal(format!("Failed to serialize auth handoff: {e}")))?;
        let encrypted_payload = self.crypto.encrypt(
            std::str::from_utf8(&serialized)
                .map_err(|_| AppError::Internal("Auth handoff encoding failed".into()))?,
        )?;
        let handoff = AuthHandoff::new(
            user_id,
            hash_handoff_code(&handoff_code),
            encrypted_payload,
            self.handoff_ttl_seconds,
        );
        self.handoff_repo.save(&handoff).await?;

        let mut redirect = self.web_callback_url()?;
        redirect
            .query_pairs_mut()
            .append_pair("code", &handoff_code);
        Ok(CallbackRedirect {
            location: redirect.to_string(),
        })
    }

    async fn complete_callback(
        &self,
        provider: &OAuthProviderConfig,
        flow: &OAuthFlowState,
        code: &str,
        user_agent: Option<String>,
        ip_address: Option<String>,
    ) -> Result<(OAuthExchangeResponseDto, UserId)> {
        let verifier = self.crypto.decrypt(&flow.encrypted_code_verifier)?;
        let client_secret = self.crypto.decrypt(&provider.encrypted_client_secret)?;
        let endpoints = self.provider_client.resolve_endpoints(provider).await?;
        let token_response = self
            .provider_client
            .exchange_code(
                provider,
                &client_secret,
                &endpoints,
                code,
                &verifier,
                &flow.redirect_uri,
            )
            .await?;

        let external = self
            .external_identity(provider, &endpoints, &token_response, flow.nonce.as_deref())
            .await?;

        match flow.purpose {
            OAuthFlowPurpose::Link => {
                let link_user_id = flow
                    .link_user_id
                    .clone()
                    .ok_or_else(|| AppError::Unauthorized("OAuth link state has no user".into()))?;
                let identity = self
                    .finish_link(&link_user_id, &provider.provider_key, external)
                    .await?;
                Ok((
                    OAuthExchangeResponseDto {
                        user: None,
                        access_token: None,
                        refresh_token: None,
                        requires_2fa: false,
                        temp_token: None,
                        linked_identity: Some(identity),
                    },
                    link_user_id,
                ))
            }
            OAuthFlowPurpose::Login => {
                self.finish_login(&provider.provider_key, external, user_agent, ip_address)
                    .await
            }
        }
    }

    async fn external_identity(
        &self,
        provider: &OAuthProviderConfig,
        endpoints: &ResolvedOAuthEndpoints,
        token_response: &TokenEndpointResponse,
        nonce: Option<&str>,
    ) -> Result<ExternalIdentity> {
        if provider.is_oidc() {
            let id_token = token_response.id_token.as_deref().ok_or_else(|| {
                AppError::ExternalService("OIDC token response did not include an ID token".into())
            })?;
            let nonce = nonce
                .ok_or_else(|| AppError::Unauthorized("OIDC flow is missing its nonce".into()))?;
            let validated = self
                .provider_client
                .validate_id_token(provider, endpoints, id_token, nonce)
                .await?;
            let userinfo = self
                .provider_client
                .fetch_userinfo(provider, endpoints, &token_response.access_token)
                .await?;

            let userinfo_subject = userinfo
                .get("sub")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| {
                    AppError::ExternalService("OIDC userinfo did not include a subject".into())
                })?;
            if userinfo_subject != validated.subject {
                return Err(AppError::Unauthorized(
                    "OIDC userinfo subject does not match the ID token".into(),
                ));
            }

            let email_claim = provider.email_claim.as_deref().unwrap_or("email");
            let name_claim = provider.name_claim.as_deref().unwrap_or("name");
            let verified_claim = provider
                .email_verified_claim
                .as_deref()
                .unwrap_or("email_verified");

            let email = validated
                .email
                .or_else(|| claim_string(userinfo.get(email_claim)))
                .map(|email| email.trim().to_lowercase())
                .filter(|email| !email.is_empty());
            let email_verified =
                validated.email_verified || claim_bool(userinfo.get(verified_claim), false);

            return Ok(ExternalIdentity {
                subject: validated.subject,
                email,
                email_verified,
                display_name: sanitize_display_name(
                    validated
                        .display_name
                        .or_else(|| claim_string(userinfo.get(name_claim))),
                ),
            });
        }

        let userinfo = self
            .provider_client
            .fetch_userinfo(provider, endpoints, &token_response.access_token)
            .await?;

        match provider.provider_type {
            OAuthProviderType::Github => {
                let subject = claim_string(userinfo.get("id"))
                    .ok_or_else(|| AppError::ExternalService("GitHub response had no id".into()))?;
                let profile_email = userinfo
                    .get("email")
                    .and_then(serde_json::Value::as_str)
                    .map(|email| email.trim().to_lowercase())
                    .filter(|email| !email.is_empty());

                let mut verified_email = None;
                if let Ok(emails) = self
                    .provider_client
                    .fetch_github_emails(endpoints, &token_response.access_token)
                    .await
                {
                    let selected = emails
                        .iter()
                        .find(|entry| {
                            entry.get("primary").and_then(serde_json::Value::as_bool) == Some(true)
                                && entry.get("verified").and_then(serde_json::Value::as_bool)
                                    == Some(true)
                        })
                        .or_else(|| {
                            emails.iter().find(|entry| {
                                entry.get("verified").and_then(serde_json::Value::as_bool)
                                    == Some(true)
                            })
                        });
                    verified_email = selected
                        .and_then(|entry| entry.get("email"))
                        .and_then(serde_json::Value::as_str)
                        .map(|email| email.trim().to_lowercase())
                        .filter(|email| !email.is_empty());
                }

                let (email, email_verified) = match verified_email {
                    Some(email) => (Some(email), true),
                    None => (profile_email, false),
                };

                return Ok(ExternalIdentity {
                    subject,
                    email,
                    email_verified,
                    display_name: sanitize_display_name(
                        claim_string(
                            userinfo.get(provider.name_claim.as_deref().unwrap_or("name")),
                        )
                        .or_else(|| claim_string(userinfo.get("login"))),
                    ),
                });
            }
            OAuthProviderType::Facebook => {
                let subject = claim_string(userinfo.get("id")).ok_or_else(|| {
                    AppError::ExternalService("Facebook response had no id".into())
                })?;
                let email =
                    claim_string(userinfo.get(provider.email_claim.as_deref().unwrap_or("email")))
                        .map(|email| email.trim().to_lowercase())
                        .filter(|email| !email.is_empty());
                // Facebook only returns confirmed primary email addresses.
                return Ok(ExternalIdentity {
                    subject,
                    email_verified: email.is_some(),
                    email,
                    display_name: sanitize_display_name(claim_string(
                        userinfo.get(provider.name_claim.as_deref().unwrap_or("name")),
                    )),
                });
            }
            _ => {
                let subject_claim = provider.subject_claim.as_deref().unwrap_or("id");
                let subject = claim_string(userinfo.get(subject_claim)).ok_or_else(|| {
                    AppError::ExternalService("OAuth2 userinfo had no subject claim".into())
                })?;
                let email =
                    claim_string(userinfo.get(provider.email_claim.as_deref().unwrap_or("email")))
                        .map(|email| email.trim().to_lowercase())
                        .filter(|email| !email.is_empty());
                let email_verified = provider
                    .email_verified_claim
                    .as_deref()
                    .map(|claim| claim_bool(userinfo.get(claim), false))
                    .unwrap_or(false);
                return Ok(ExternalIdentity {
                    subject,
                    email,
                    email_verified,
                    display_name: sanitize_display_name(claim_string(
                        userinfo.get(provider.name_claim.as_deref().unwrap_or("name")),
                    )),
                });
            }
        }
    }

    async fn finish_link(
        &self,
        user_id: &UserId,
        provider_key: &str,
        external: ExternalIdentity,
    ) -> Result<OAuthIdentityDto> {
        let user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::Unauthorized("User not found".into()))?;
        if user.status != UserStatus::Active {
            return Err(AppError::Unauthorized("Account is suspended".into()));
        }

        if let Some(existing) = self
            .identity_repo
            .find_by_provider_subject(provider_key, &external.subject)
            .await?
        {
            if existing.user_id != *user_id {
                return Err(AppError::Conflict(
                    "This provider account is linked to another user".into(),
                ));
            }
            self.identity_repo.touch_last_login(&existing.id).await?;
            let mut existing = existing;
            existing.last_login_at = Utc::now();
            return Ok(existing.into());
        }

        let identity = UserOAuthIdentity::new(
            user_id.clone(),
            provider_key.to_string(),
            external.subject,
            external.email,
            external.email_verified,
            external.display_name,
        );
        self.identity_repo.save(&identity).await?;
        Ok(identity.into())
    }

    async fn finish_login(
        &self,
        provider_key: &str,
        external: ExternalIdentity,
        user_agent: Option<String>,
        ip_address: Option<String>,
    ) -> Result<(OAuthExchangeResponseDto, UserId)> {
        if external.subject.trim().is_empty() {
            return Err(AppError::Unauthorized(
                "Provider did not return a stable subject".into(),
            ));
        }

        let user = if let Some(identity) = self
            .identity_repo
            .find_by_provider_subject(provider_key, &external.subject)
            .await?
        {
            let user = self
                .user_repo
                .find_by_id(&identity.user_id)
                .await?
                .ok_or_else(|| AppError::Unauthorized("Linked user not found".into()))?;
            self.identity_repo.touch_last_login(&identity.id).await?;
            user
        } else {
            // Never auto-link by email. A verified email is required to create
            // a fresh account; an existing email requires explicit linking.
            let email_raw = external.email.clone().ok_or_else(|| {
                AppError::Forbidden("Provider did not return an email address".into())
            })?;
            if !external.email_verified {
                return Err(AppError::Forbidden("Provider email is not verified".into()));
            }
            let email = Email::new(email_raw)?;
            if self.user_repo.find_by_email(&email).await?.is_some() {
                return Err(AppError::Conflict(
                    "An account with this email already exists; sign in and link this provider"
                        .into(),
                ));
            }

            let name = external.display_name.clone().unwrap_or_else(|| {
                email
                    .as_str()
                    .split('@')
                    .next()
                    .unwrap_or("Member")
                    .to_string()
            });
            let user = User::new(
                email,
                name,
                OAUTH_ONLY_PASSWORD_MARKER.to_string(),
                crate::contexts::identity::domain::UserRole::Member,
            )?;
            self.user_repo.save(&user).await?;

            let identity = UserOAuthIdentity::new(
                user.id.clone(),
                provider_key.to_string(),
                external.subject.clone(),
                external.email.clone(),
                external.email_verified,
                external.display_name.clone(),
            );
            self.identity_repo.save(&identity).await?;
            user
        };

        if user.status != UserStatus::Active {
            return Err(AppError::Unauthorized("Account is suspended".into()));
        }

        if user.totp_enabled {
            let temp_token = self.token_service.generate_temp_2fa_token(&user.id)?;
            return Ok((
                OAuthExchangeResponseDto {
                    user: None,
                    access_token: None,
                    refresh_token: None,
                    requires_2fa: true,
                    temp_token: Some(temp_token),
                    linked_identity: None,
                },
                user.id,
            ));
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

        let user_id = user.id.clone();
        let user_dto: UserResponseDto = user.into();
        Ok((
            OAuthExchangeResponseDto {
                user: Some(user_dto),
                access_token: Some(access_token),
                refresh_token: Some(refresh_token),
                requires_2fa: false,
                temp_token: None,
                linked_identity: None,
            },
            user_id,
        ))
    }

    pub async fn exchange(&self, code: &str) -> Result<OAuthExchangeResponseDto> {
        if code.trim().is_empty() {
            return Err(AppError::Unauthorized("Exchange code is missing".into()));
        }
        let handoff = self
            .handoff_repo
            .consume_by_hash(&hash_handoff_code(code.trim()))
            .await?
            .ok_or_else(|| {
                AppError::Unauthorized("Exchange code is invalid, expired, or already used".into())
            })?;
        let payload = self.crypto.decrypt(&handoff.encrypted_payload)?;
        serde_json::from_str(&payload)
            .map_err(|_| AppError::Internal("Stored auth handoff payload is invalid".into()))
    }

    pub async fn list_identities(&self, user_id: &UserId) -> Result<Vec<OAuthIdentityDto>> {
        Ok(self
            .identity_repo
            .list_for_user(user_id)
            .await?
            .into_iter()
            .map(OAuthIdentityDto::from)
            .collect())
    }

    pub async fn unlink_identity(&self, user_id: &UserId, identity_id: &str) -> Result<()> {
        let identity = self
            .identity_repo
            .find_by_id(identity_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Linked identity not found".into()))?;
        if identity.user_id != *user_id {
            return Err(AppError::NotFound("Linked identity not found".into()));
        }

        let user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".into()))?;
        let has_password = !user.password_hash.trim().is_empty()
            && user.password_hash != OAUTH_ONLY_PASSWORD_MARKER;

        let linked_count = self.identity_repo.count_for_user(user_id).await?;
        if !has_password && linked_count <= 1 {
            return Err(AppError::Conflict(
                "Cannot remove the last usable sign-in method".into(),
            ));
        }

        if !self.identity_repo.delete(&identity.id).await? {
            return Err(AppError::NotFound("Linked identity not found".into()));
        }
        Ok(())
    }

    pub async fn list_admin_providers(&self) -> Result<Vec<OAuthProviderAdminDto>> {
        Ok(self
            .provider_repo
            .list_all()
            .await?
            .into_iter()
            .map(admin_dto)
            .collect())
    }

    pub async fn upsert_admin_provider(
        &self,
        provider_key: &str,
        request: OAuthProviderUpsertRequestDto,
    ) -> Result<OAuthProviderAdminDto> {
        let provider_key = normalize_provider_key(provider_key)?;
        let existing = self.provider_repo.find_by_key(&provider_key).await?;

        let provider_type = request
            .provider_type
            .or_else(|| existing.as_ref().map(|provider| provider.provider_type))
            .unwrap_or_else(|| infer_provider_type(&provider_key));

        let client_id = request
            .client_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .or_else(|| existing.as_ref().map(|provider| provider.client_id.clone()))
            .ok_or_else(|| AppError::Validation("client_id is required".into()))?;

        let encrypted_client_secret = match request.client_secret.as_deref() {
            Some(secret) if !secret.trim().is_empty() => self.crypto.encrypt(secret.trim())?,
            _ => existing
                .as_ref()
                .map(|provider| provider.encrypted_client_secret.clone())
                .ok_or_else(|| AppError::Validation("client_secret is required".into()))?,
        };

        let redirect_uri = request
            .redirect_uri
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .or_else(|| {
                existing
                    .as_ref()
                    .map(|provider| provider.redirect_uri.clone())
            })
            .ok_or_else(|| AppError::Validation("redirect_uri is required".into()))?;
        validate_redirect_uri(&redirect_uri)?;

        let issuer = merge_optional(
            existing
                .as_ref()
                .and_then(|provider| provider.issuer.clone()),
            request.issuer,
        )
        .or_else(|| provider_type.default_issuer().map(str::to_string));

        let scopes = match request.scopes {
            Some(scopes) if !scopes.is_empty() => scopes,
            Some(_) => provider_type.default_scopes(),
            None => existing
                .as_ref()
                .map(|provider| provider.scopes.clone())
                .filter(|scopes| !scopes.is_empty())
                .unwrap_or_else(|| provider_type.default_scopes()),
        };

        let now = Utc::now();
        let provider = OAuthProviderConfig {
            provider_key: provider_key.clone(),
            name: optional_trimmed(request.name)
                .or_else(|| existing.as_ref().map(|provider| provider.name.clone()))
                .unwrap_or_else(|| provider_key.clone()),
            provider_type,
            issuer,
            client_id,
            encrypted_client_secret,
            redirect_uri,
            scopes,
            authorization_endpoint: merge_optional(
                existing
                    .as_ref()
                    .and_then(|provider| provider.authorization_endpoint.clone()),
                request.authorization_endpoint,
            ),
            token_endpoint: merge_optional(
                existing
                    .as_ref()
                    .and_then(|provider| provider.token_endpoint.clone()),
                request.token_endpoint,
            ),
            userinfo_endpoint: merge_optional(
                existing
                    .as_ref()
                    .and_then(|provider| provider.userinfo_endpoint.clone()),
                request.userinfo_endpoint,
            ),
            email_endpoint: merge_optional(
                existing
                    .as_ref()
                    .and_then(|provider| provider.email_endpoint.clone()),
                request.email_endpoint,
            ),
            jwks_uri: merge_optional(
                existing
                    .as_ref()
                    .and_then(|provider| provider.jwks_uri.clone()),
                request.jwks_uri,
            ),
            subject_claim: merge_optional(
                existing
                    .as_ref()
                    .and_then(|provider| provider.subject_claim.clone()),
                request.subject_claim,
            ),
            email_claim: merge_optional(
                existing
                    .as_ref()
                    .and_then(|provider| provider.email_claim.clone()),
                request.email_claim,
            ),
            name_claim: merge_optional(
                existing
                    .as_ref()
                    .and_then(|provider| provider.name_claim.clone()),
                request.name_claim,
            ),
            email_verified_claim: merge_optional(
                existing
                    .as_ref()
                    .and_then(|provider| provider.email_verified_claim.clone()),
                request.email_verified_claim,
            ),
            is_enabled: request
                .is_enabled
                .or_else(|| existing.as_ref().map(|provider| provider.is_enabled))
                .unwrap_or(true),
            created_at: existing
                .as_ref()
                .map(|provider| provider.created_at)
                .unwrap_or(now),
            updated_at: now,
        };

        self.validate_provider_config(&provider).await?;
        self.provider_repo.save(&provider).await?;
        Ok(admin_dto(provider))
    }

    async fn validate_provider_config(&self, provider: &OAuthProviderConfig) -> Result<()> {
        if provider.provider_type.is_oidc() && provider.issuer.as_deref().is_none() {
            return Err(AppError::Validation(
                "OIDC providers require an issuer".into(),
            ));
        }
        if !provider.provider_type.is_oidc() {
            if provider.authorization_endpoint.is_none()
                && provider
                    .provider_type
                    .default_authorization_endpoint()
                    .is_none()
            {
                return Err(AppError::Validation(
                    "authorization_endpoint is required".into(),
                ));
            }
            if provider.token_endpoint.is_none()
                && provider.provider_type.default_token_endpoint().is_none()
            {
                return Err(AppError::Validation("token_endpoint is required".into()));
            }
            if provider.userinfo_endpoint.is_none()
                && provider.provider_type.default_userinfo_endpoint().is_none()
            {
                return Err(AppError::Validation("userinfo_endpoint is required".into()));
            }
        }

        for endpoint in [
            provider.issuer.as_deref(),
            provider.authorization_endpoint.as_deref(),
            provider.token_endpoint.as_deref(),
            provider.userinfo_endpoint.as_deref(),
            provider.email_endpoint.as_deref(),
            provider.jwks_uri.as_deref(),
        ]
        .into_iter()
        .flatten()
        {
            self.provider_client.validate_endpoint(endpoint).await?;
        }

        // Resolve (and for OIDC, discover) once so unsafe discovered endpoints
        // are rejected at configuration time as well as at use time.
        self.provider_client.resolve_endpoints(provider).await?;
        Ok(())
    }

    pub async fn delete_admin_provider(&self, provider_key: &str) -> Result<()> {
        let provider_key = normalize_provider_key(provider_key)?;
        if !self.provider_repo.delete(&provider_key).await? {
            return Err(AppError::NotFound("OAuth provider not found".into()));
        }
        Ok(())
    }

    async fn require_enabled_provider(&self, provider_key: &str) -> Result<OAuthProviderConfig> {
        let provider_key = normalize_provider_key(provider_key)?;
        match self.provider_repo.find_by_key(&provider_key).await? {
            Some(provider) if provider.is_enabled => Ok(provider),
            _ => Err(AppError::NotFound("OAuth provider not found".into())),
        }
    }

    fn web_callback_url(&self) -> Result<reqwest::Url> {
        let raw = self
            .web_callback_url
            .as_deref()
            .filter(|url| !url.trim().is_empty())
            .ok_or_else(|| {
                AppError::ServiceUnavailable("OAUTH_WEB_CALLBACK_URL is not configured".into())
            })?;
        let url = reqwest::Url::parse(raw).map_err(|_| {
            AppError::ServiceUnavailable("OAUTH_WEB_CALLBACK_URL is invalid".into())
        })?;
        if !matches!(url.scheme(), "http" | "https")
            || url.host_str().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
        {
            return Err(AppError::ServiceUnavailable(
                "OAUTH_WEB_CALLBACK_URL is invalid".into(),
            ));
        }
        Ok(url)
    }

    fn error_redirect(&self, code: CallbackErrorCode) -> Result<CallbackRedirect> {
        let mut url = self.web_callback_url()?;
        url.query_pairs_mut().append_pair("error", code.as_str());
        Ok(CallbackRedirect {
            location: url.to_string(),
        })
    }
}

fn admin_dto(provider: OAuthProviderConfig) -> OAuthProviderAdminDto {
    OAuthProviderAdminDto {
        provider_key: provider.provider_key,
        name: provider.name,
        provider_type: provider.provider_type,
        issuer: provider.issuer,
        client_id: provider.client_id,
        has_client_secret: !provider.encrypted_client_secret.is_empty(),
        redirect_uri: provider.redirect_uri,
        scopes: provider.scopes,
        authorization_endpoint: provider.authorization_endpoint,
        token_endpoint: provider.token_endpoint,
        userinfo_endpoint: provider.userinfo_endpoint,
        email_endpoint: provider.email_endpoint,
        jwks_uri: provider.jwks_uri,
        subject_claim: provider.subject_claim,
        email_claim: provider.email_claim,
        name_claim: provider.name_claim,
        email_verified_claim: provider.email_verified_claim,
        is_enabled: provider.is_enabled,
        created_at: provider.created_at,
        updated_at: provider.updated_at,
    }
}

fn merge_optional(existing: Option<String>, update: Option<String>) -> Option<String> {
    match update {
        None => existing,
        Some(value) if value.trim().is_empty() => None,
        Some(value) => Some(value.trim().to_string()),
    }
}

fn normalize_provider_key(raw: &str) -> Result<String> {
    let key = raw.trim().to_ascii_lowercase();
    if key.len() < 2
        || key.len() > 64
        || !key
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
    {
        return Err(AppError::Validation(
            "provider_key must be 2-64 lowercase letters, digits, '_' or '-'".into(),
        ));
    }
    Ok(key)
}

fn infer_provider_type(provider_key: &str) -> OAuthProviderType {
    match provider_key {
        "google" => OAuthProviderType::Google,
        "github" => OAuthProviderType::Github,
        "facebook" => OAuthProviderType::Facebook,
        _ => OAuthProviderType::OAuth2,
    }
}

fn constant_time_eq(left: &str, right: &str) -> bool {
    left.len() == right.len() && left.as_bytes().ct_eq(right.as_bytes()).into()
}

fn callback_error_code(error: &AppError) -> Option<CallbackErrorCode> {
    match error {
        AppError::Unauthorized(_) => Some(CallbackErrorCode::OAuthFailed),
        AppError::Forbidden(_) => Some(CallbackErrorCode::EmailUnverified),
        AppError::Conflict(_) => Some(CallbackErrorCode::AccountExists),
        AppError::Validation(_) | AppError::ExternalService(_) => {
            Some(CallbackErrorCode::OAuthFailed)
        }
        AppError::NotFound(_) => Some(CallbackErrorCode::OAuthFailed),
        AppError::Database(_) | AppError::Internal(_) | AppError::ServiceUnavailable(_) => None,
    }
}
