use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts},
};

use crate::{
    contexts::identity::domain::{SessionId, UserId},
    interfaces::http::workspaces::AppState,
    shared::error::AppError,
};

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: UserId,
    pub session_id: SessionId,
}

impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let mut token: Option<String> = None;

        
        if let Some(auth_header) = parts.headers.get(header::AUTHORIZATION) {
            if let Ok(auth_str) = auth_header.to_str() {
                if let Some(bearer_token) = auth_str.strip_prefix("Bearer ") {
                    token = Some(bearer_token.trim().to_string());
                }
            }
        }

        
        if token.is_none() {
            if let Some(cookie_header) = parts.headers.get(header::COOKIE) {
                if let Ok(cookie_str) = cookie_header.to_str() {
                    for cookie in cookie_str.split(';') {
                        let parts: Vec<&str> = cookie.trim().splitn(2, '=').collect();
                        if parts.len() == 2 && parts[0] == "mensetsu_access" {
                            token = Some(parts[1].trim().to_string());
                            break;
                        }
                    }
                }
            }
        }

        let token = token.ok_or_else(|| {
            AppError::Unauthorized("Missing authentication token (header or cookie)".into())
        })?;

        let claims = state.identity_service.token_service().verify_access_token(&token)?;

        let session_id = SessionId(claims.sid);
        let user_id = UserId(claims.sub);

        
        let session = state
            .identity_service
            .session_repo()
            .find_by_id(&session_id)
            .await?
            .ok_or_else(|| AppError::Unauthorized("Session not found or invalidated".into()))?;

        if !session.is_active() {
            return Err(AppError::Unauthorized("Session expired or revoked".into()));
        }

        if session.user_id != user_id {
            return Err(AppError::Unauthorized("Session user mismatch".into()));
        }

        Ok(AuthenticatedUser {
            user_id,
            session_id,
        })
    }
}
