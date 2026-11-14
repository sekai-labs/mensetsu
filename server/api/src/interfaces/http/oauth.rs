use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
    Json, Router,
};

use crate::contexts::identity::application::oauth_dtos::{
    OAuthCallbackQuery, OAuthExchangeRequestDto, OAuthExchangeResponseDto, OAuthIdentityDto,
    OAuthLinkStartResponseDto, OAuthProviderAdminDto, OAuthProviderSummaryDto,
    OAuthProviderUpsertRequestDto,
};
use crate::interfaces::http::{
    admin::verify_admin, extractors::authenticated_user::AuthenticatedUser, workspaces::AppState,
};
use crate::shared::error::{AppError, Result};

const OAUTH_STATE_COOKIE: &str = "mensetsu_oauth_state";
const OAUTH_STATE_COOKIE_PATH: &str = "/api/v1/auth/oauth";

pub fn oauth_routes(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/auth/oauth/providers", get(list_providers))
        .route(
            "/api/v1/auth/oauth/authorize/{provider_key}",
            get(authorize_login),
        )
        .route(
            "/api/v1/auth/oauth/link/{provider_key}",
            post(authorize_link),
        )
        .route("/api/v1/auth/oauth/callback", get(oauth_callback))
        .route("/api/v1/auth/oauth/exchange", post(exchange_handoff))
        .route("/api/v1/auth/oauth/accounts", get(list_accounts))
        .route(
            "/api/v1/auth/oauth/accounts/{identity_id}",
            delete(unlink_account),
        )
        .route("/api/v1/admin/oauth/providers", get(admin_list_providers))
        .route(
            "/api/v1/admin/oauth/providers/{provider_key}",
            put(admin_upsert_provider).delete(admin_delete_provider),
        )
        .with_state(state)
}

fn oauth_service(
    state: &AppState,
) -> Result<&std::sync::Arc<crate::contexts::identity::application::oauth_service::OAuthService>> {
    state
        .oauth_service
        .as_ref()
        .ok_or_else(|| AppError::ServiceUnavailable("OAuth is not configured".into()))
}

fn state_cookie(headers: &HeaderMap) -> Option<String> {
    let cookie_header = headers.get(header::COOKIE)?.to_str().ok()?;
    for cookie in cookie_header.split(';') {
        if let Some((name, value)) = cookie.trim().split_once('=') {
            if name == OAUTH_STATE_COOKIE {
                return Some(value.trim().to_string());
            }
        }
    }
    None
}

fn build_state_cookie(state: &str, ttl_seconds: i64) -> String {
    format!(
        "{OAUTH_STATE_COOKIE}={state}; HttpOnly; Secure; SameSite=Lax; Path={OAUTH_STATE_COOKIE_PATH}; Max-Age={ttl_seconds}"
    )
}

fn clear_state_cookie() -> String {
    format!(
        "{OAUTH_STATE_COOKIE}=; HttpOnly; Secure; SameSite=Lax; Path={OAUTH_STATE_COOKIE_PATH}; Max-Age=0; Expires=Thu, 01 Jan 1970 00:00:00 GMT"
    )
}

fn redirect_response(location: &str, cookie: Option<String>) -> Result<Response> {
    let mut response = StatusCode::SEE_OTHER.into_response();
    response.headers_mut().insert(
        header::LOCATION,
        HeaderValue::from_str(location)
            .map_err(|_| AppError::Internal("Invalid OAuth redirect URL".into()))?,
    );
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    if let Some(cookie) = cookie {
        response.headers_mut().append(
            header::SET_COOKIE,
            HeaderValue::from_str(&cookie)
                .map_err(|_| AppError::Internal("Invalid OAuth cookie".into()))?,
        );
    }
    Ok(response)
}

fn request_origin_headers(headers: &HeaderMap) -> (Option<String>, Option<String>) {
    let user_agent = headers
        .get(header::USER_AGENT)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    let ip_address = headers
        .get("x-forwarded-for")
        .or_else(|| headers.get("x-real-ip"))
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    (user_agent, ip_address)
}

#[utoipa::path(
    get,
    path = "/api/v1/auth/oauth/providers",
    security(()),
    responses(
        (status = 200, description = "Enabled OAuth providers", body = [OAuthProviderSummaryDto])
    )
)]
pub async fn list_providers(
    State(state): State<AppState>,
) -> Result<Json<Vec<OAuthProviderSummaryDto>>> {
    Ok(Json(oauth_service(&state)?.list_public_providers().await?))
}

#[utoipa::path(
    get,
    path = "/api/v1/auth/oauth/authorize/{provider_key}",
    security(()),
    params(("provider_key" = String, Path, description = "OAuth provider key")),
    responses(
        (status = 303, description = "Redirect to the identity provider"),
        (status = 404, description = "Unknown or disabled provider")
    )
)]
pub async fn authorize_login(
    State(state): State<AppState>,
    Path(provider_key): Path<String>,
) -> Result<Response> {
    let start = oauth_service(&state)?.begin_login(&provider_key).await?;
    redirect_response(
        &start.authorization_url,
        Some(build_state_cookie(&start.state, start.expires_in_seconds)),
    )
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/oauth/link/{provider_key}",
    params(("provider_key" = String, Path, description = "OAuth provider key")),
    responses(
        (status = 200, description = "Provider authorization URL for explicit linking", body = OAuthLinkStartResponseDto),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn authorize_link(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(provider_key): Path<String>,
) -> Result<Json<OAuthLinkStartResponseDto>> {
    let start = oauth_service(&state)?
        .begin_link(&provider_key, &auth_user.user_id)
        .await?;
    Ok(Json(OAuthLinkStartResponseDto {
        authorization_url: start.authorization_url,
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/auth/oauth/callback",
    security(()),
    params(OAuthCallbackQuery),
    responses(
        (status = 303, description = "Redirect to the web BFF with a one-use exchange code or stable error code"),
        (status = 400, description = "Invalid callback state")
    )
)]
pub async fn oauth_callback(
    State(state): State<AppState>,
    Query(query): Query<OAuthCallbackQuery>,
    headers: HeaderMap,
) -> Result<Response> {
    let (user_agent, ip_address) = request_origin_headers(&headers);
    let redirect = oauth_service(&state)?
        .handle_callback(
            query.code.as_deref(),
            query.state.as_deref(),
            query.error.as_deref(),
            state_cookie(&headers).as_deref(),
            user_agent,
            ip_address,
        )
        .await?;
    redirect_response(&redirect.location, Some(clear_state_cookie()))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/oauth/exchange",
    security(()),
    request_body = OAuthExchangeRequestDto,
    responses(
        (status = 200, description = "Exchanged one-use OAuth handoff", body = OAuthExchangeResponseDto),
        (status = 401, description = "Invalid, expired, or reused exchange code")
    )
)]
pub async fn exchange_handoff(
    State(state): State<AppState>,
    Json(payload): Json<OAuthExchangeRequestDto>,
) -> Result<Response> {
    let exchanged = oauth_service(&state)?.exchange(&payload.code).await?;
    let mut response = Json(exchanged).into_response();
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    Ok(response)
}

#[utoipa::path(
    get,
    path = "/api/v1/auth/oauth/accounts",
    responses(
        (status = 200, description = "Linked external identities", body = [OAuthIdentityDto]),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn list_accounts(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<Json<Vec<OAuthIdentityDto>>> {
    Ok(Json(
        oauth_service(&state)?
            .list_identities(&auth_user.user_id)
            .await?,
    ))
}

#[utoipa::path(
    delete,
    path = "/api/v1/auth/oauth/accounts/{identity_id}",
    params(("identity_id" = String, Path, description = "Linked identity id")),
    responses(
        (status = 204, description = "Identity unlinked"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Identity not found"),
        (status = 409, description = "Cannot remove the last usable sign-in method")
    )
)]
pub async fn unlink_account(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(identity_id): Path<String>,
) -> Result<StatusCode> {
    oauth_service(&state)?
        .unlink_identity(&auth_user.user_id, &identity_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/api/v1/admin/oauth/providers",
    responses(
        (status = 200, description = "Configured OAuth providers (secrets redacted)", body = [OAuthProviderAdminDto]),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn admin_list_providers(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<Json<Vec<OAuthProviderAdminDto>>> {
    verify_admin(&state, &auth_user.user_id.0).await?;
    Ok(Json(oauth_service(&state)?.list_admin_providers().await?))
}

#[utoipa::path(
    put,
    path = "/api/v1/admin/oauth/providers/{provider_key}",
    params(("provider_key" = String, Path, description = "OAuth provider key")),
    request_body = OAuthProviderUpsertRequestDto,
    responses(
        (status = 200, description = "OAuth provider saved; omitted fields preserved", body = OAuthProviderAdminDto),
        (status = 400, description = "Validation error"),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn admin_upsert_provider(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(provider_key): Path<String>,
    Json(payload): Json<OAuthProviderUpsertRequestDto>,
) -> Result<Json<OAuthProviderAdminDto>> {
    verify_admin(&state, &auth_user.user_id.0).await?;
    Ok(Json(
        oauth_service(&state)?
            .upsert_admin_provider(&provider_key, payload)
            .await?,
    ))
}

#[utoipa::path(
    delete,
    path = "/api/v1/admin/oauth/providers/{provider_key}",
    params(("provider_key" = String, Path, description = "OAuth provider key")),
    responses(
        (status = 204, description = "OAuth provider deleted"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Provider not found")
    )
)]
pub async fn admin_delete_provider(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(provider_key): Path<String>,
) -> Result<StatusCode> {
    verify_admin(&state, &auth_user.user_id.0).await?;
    oauth_service(&state)?
        .delete_admin_provider(&provider_key)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
