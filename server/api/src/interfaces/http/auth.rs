use axum::{
    extract::State,
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};

use crate::{
    contexts::identity::application::dtos::{
        AuthResponseDto, Login2faRequestDto, LoginRequestDto, LoginResponseDto,
        MessageResponseDto, RefreshTokenRequestDto, RegisterRequestDto, TokenResponseDto,
        TotpSetupResponseDto, UserResponseDto, VerifyTotpRequestDto,
    },
    interfaces::http::{
        extractors::authenticated_user::AuthenticatedUser,
        workspaces::AppState,
    },
    shared::error::{AppError, Result},
};

pub fn auth_routes(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/auth/register", post(register))
        .route("/api/v1/auth/login", post(login))
        .route("/api/v1/auth/login-2fa", post(login_2fa))
        .route("/api/v1/auth/2fa/setup", post(setup_2fa))
        .route("/api/v1/auth/2fa/verify", post(verify_2fa))
        .route("/api/v1/auth/2fa/disable", post(disable_2fa))
        .route("/api/v1/auth/refresh", post(refresh))
        .route("/api/v1/auth/logout", post(logout))
        .route("/api/v1/auth/logout-all", post(logout_all))
        .route("/api/v1/auth/me", get(me))
        .with_state(state)
}

fn extract_refresh_token_from_cookies(headers: &HeaderMap) -> Option<String> {
    if let Some(cookie_header) = headers.get(header::COOKIE) {
        if let Ok(cookie_str) = cookie_header.to_str() {
            for cookie in cookie_str.split(';') {
                let parts: Vec<&str> = cookie.trim().splitn(2, '=').collect();
                if parts.len() == 2 && parts[0] == "mensetsu_refresh" {
                    return Some(parts[1].trim().to_string());
                }
            }
        }
    }
    None
}

fn build_auth_cookies(
    access_token: &str,
    refresh_token: &str,
    access_ttl: i64,
    refresh_ttl: i64,
) -> Vec<String> {
    vec![
        format!(
            "mensetsu_access={access_token}; HttpOnly; SameSite=Lax; Path=/; Max-Age={access_ttl}"
        ),
        format!(
            "mensetsu_refresh={refresh_token}; HttpOnly; SameSite=Lax; Path=/; Max-Age={refresh_ttl}"
        ),
    ]
}

fn build_clear_cookies() -> Vec<String> {
    vec![
        "mensetsu_access=; HttpOnly; SameSite=Lax; Path=/; Max-Age=0; Expires=Thu, 01 Jan 1970 00:00:00 GMT".to_string(),
        "mensetsu_refresh=; HttpOnly; SameSite=Lax; Path=/; Max-Age=0; Expires=Thu, 01 Jan 1970 00:00:00 GMT".to_string(),
    ]
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/register",
    request_body = RegisterRequestDto,
    responses(
        (status = 201, description = "Registration successful", body = AuthResponseDto),
        (status = 400, description = "Validation error"),
        (status = 409, description = "Email already registered")
    )
)]
pub async fn register(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<RegisterRequestDto>,
) -> Result<Response> {
    let user_agent = headers
        .get(header::USER_AGENT)
        .and_then(|h| h.to_str().ok())
        .map(String::from);
    let ip_address = headers
        .get("x-forwarded-for")
        .or_else(|| headers.get("x-real-ip"))
        .and_then(|h| h.to_str().ok())
        .map(String::from);

    let auth_data = state
        .identity_service
        .register(payload, user_agent, ip_address)
        .await?;

    let cookies = build_auth_cookies(
        &auth_data.access_token,
        &auth_data.refresh_token,
        state.config.access_token_ttl_seconds,
        state.config.refresh_token_ttl_seconds,
    );

    let mut response = (StatusCode::CREATED, Json(auth_data)).into_response();
    for cookie in cookies {
        if let Ok(hv) = header::HeaderValue::from_str(&cookie) {
            response.headers_mut().append(header::SET_COOKIE, hv);
        }
    }

    Ok(response)
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    request_body = LoginRequestDto,
    responses(
        (status = 200, description = "Login response or 2FA challenge", body = LoginResponseDto),
        (status = 401, description = "Invalid credentials or account suspended")
    )
)]
pub async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<LoginRequestDto>,
) -> Result<Response> {
    let user_agent = headers
        .get(header::USER_AGENT)
        .and_then(|h| h.to_str().ok())
        .map(String::from);
    let ip_address = headers
        .get("x-forwarded-for")
        .or_else(|| headers.get("x-real-ip"))
        .and_then(|h| h.to_str().ok())
        .map(String::from);

    let login_data = state
        .identity_service
        .login(payload, user_agent, ip_address)
        .await?;

    if login_data.requires_2fa {
        return Ok((StatusCode::OK, Json(login_data)).into_response());
    }

    let mut response = (StatusCode::OK, Json(&login_data)).into_response();
    if let (Some(access), Some(refresh)) = (&login_data.access_token, &login_data.refresh_token) {
        let cookies = build_auth_cookies(
            access,
            refresh,
            state.config.access_token_ttl_seconds,
            state.config.refresh_token_ttl_seconds,
        );
        for cookie in cookies {
            if let Ok(hv) = header::HeaderValue::from_str(&cookie) {
                response.headers_mut().append(header::SET_COOKIE, hv);
            }
        }
    }

    Ok(response)
}
#[utoipa::path(
    post,
    path = "/api/v1/auth/login-2fa",
    request_body = Login2faRequestDto,
    responses(
        (status = 200, description = "Authenticated with 2FA", body = AuthResponseDto),
        (status = 401, description = "Invalid or expired 2FA code")
    )
)]
pub async fn login_2fa(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<crate::contexts::identity::application::dtos::Login2faRequestDto>,
) -> Result<Response> {
    let user_agent = headers
        .get(header::USER_AGENT)
        .and_then(|h| h.to_str().ok())
        .map(String::from);
    let ip_address = headers
        .get("x-forwarded-for")
        .or_else(|| headers.get("x-real-ip"))
        .and_then(|h| h.to_str().ok())
        .map(String::from);

    let auth_data = state
        .identity_service
        .login_2fa(payload, user_agent, ip_address)
        .await?;

    let cookies = build_auth_cookies(
        &auth_data.access_token,
        &auth_data.refresh_token,
        state.config.access_token_ttl_seconds,
        state.config.refresh_token_ttl_seconds,
    );

    let mut response = (StatusCode::OK, Json(auth_data)).into_response();
    for cookie in cookies {
        if let Ok(hv) = header::HeaderValue::from_str(&cookie) {
            response.headers_mut().append(header::SET_COOKIE, hv);
        }
    }

    Ok(response)
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/2fa/setup",
    responses(
        (status = 200, description = "Initiate 2FA setup and get QR code", body = TotpSetupResponseDto),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn setup_2fa(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<Json<crate::contexts::identity::application::dtos::TotpSetupResponseDto>> {
    let setup = state.identity_service.setup_totp(&auth_user.user_id).await?;
    Ok(Json(setup))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/2fa/verify",
    request_body = VerifyTotpRequestDto,
    responses(
        (status = 200, description = "Verify code and activate 2FA", body = UserResponseDto),
        (status = 401, description = "Invalid code")
    )
)]
pub async fn verify_2fa(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Json(payload): Json<crate::contexts::identity::application::dtos::VerifyTotpRequestDto>,
) -> Result<Json<crate::contexts::identity::application::dtos::UserResponseDto>> {
    let user = state
        .identity_service
        .verify_and_enable_totp(&auth_user.user_id, &payload.code)
        .await?;
    Ok(Json(user))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/2fa/disable",
    request_body = VerifyTotpRequestDto,
    responses(
        (status = 200, description = "Disable 2FA", body = UserResponseDto),
        (status = 401, description = "Invalid code")
    )
)]
pub async fn disable_2fa(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Json(payload): Json<crate::contexts::identity::application::dtos::VerifyTotpRequestDto>,
) -> Result<Json<crate::contexts::identity::application::dtos::UserResponseDto>> {
    let user = state
        .identity_service
        .disable_totp(&auth_user.user_id, &payload.code)
        .await?;
    Ok(Json(user))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/refresh",
    request_body(content = Option<RefreshTokenRequestDto>),
    responses(
        (status = 200, description = "Token refreshed", body = TokenResponseDto),
        (status = 401, description = "Invalid or expired refresh token")
    )
)]
pub async fn refresh(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Option<Json<RefreshTokenRequestDto>>,
) -> Result<Response> {
    let raw_refresh_token = body
        .and_then(|Json(b)| b.refresh_token)
        .or_else(|| extract_refresh_token_from_cookies(&headers))
        .ok_or_else(|| AppError::Unauthorized("Refresh token missing".into()))?;

    let token_data = state
        .identity_service
        .refresh(&raw_refresh_token)
        .await?;

    let cookies = build_auth_cookies(
        &token_data.access_token,
        &token_data.refresh_token,
        state.config.access_token_ttl_seconds,
        state.config.refresh_token_ttl_seconds,
    );

    let mut response = (StatusCode::OK, Json(token_data)).into_response();
    for cookie in cookies {
        if let Ok(hv) = header::HeaderValue::from_str(&cookie) {
            response.headers_mut().append(header::SET_COOKIE, hv);
        }
    }

    Ok(response)
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/logout",
    responses(
        (status = 200, description = "Logged out successfully", body = MessageResponseDto)
    )
)]
pub async fn logout(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<Response> {
    state.identity_service.logout(&auth_user.session_id).await?;

    let mut response = (
        StatusCode::OK,
        Json(MessageResponseDto {
            message: "Logged out successfully".into(),
        }),
    )
        .into_response();

    for cookie in build_clear_cookies() {
        if let Ok(hv) = header::HeaderValue::from_str(&cookie) {
            response.headers_mut().append(header::SET_COOKIE, hv);
        }
    }

    Ok(response)
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/logout-all",
    responses(
        (status = 200, description = "Logged out from all sessions", body = MessageResponseDto)
    )
)]
pub async fn logout_all(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<Response> {
    state
        .identity_service
        .logout_all(&auth_user.user_id)
        .await?;

    let mut response = (
        StatusCode::OK,
        Json(MessageResponseDto {
            message: "Logged out from all sessions successfully".into(),
        }),
    )
        .into_response();

    for cookie in build_clear_cookies() {
        if let Ok(hv) = header::HeaderValue::from_str(&cookie) {
            response.headers_mut().append(header::SET_COOKIE, hv);
        }
    }

    Ok(response)
}

#[utoipa::path(
    get,
    path = "/api/v1/auth/me",
    responses(
        (status = 200, description = "Current authenticated user profile", body = UserResponseDto),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn me(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>> {
    let user = state
        .identity_service
        .get_user_by_id(&auth_user.user_id)
        .await?;

    Ok(Json(serde_json::json!({
        "user": user
    })))
}
