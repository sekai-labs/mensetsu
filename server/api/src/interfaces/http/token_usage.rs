use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use chrono::{DateTime, SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::{
    interfaces::http::{
        extractors::authenticated_user::AuthenticatedUser, workspaces::AppState,
    },
    shared::error::AppError,
};

const TOKEN_USAGE_EVENT: &str = "agent.model_usage";
const PAGE_SIZE: usize = 100;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TokenUsageRowDto {
    pub id: String,
    pub run_id: String,
    pub execution_id: Option<String>,
    pub agent_id: String,
    pub model_provider: String,
    pub model_name: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_tokens: u64,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TokenUsageTotalsDto {
    pub request_count: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_tokens: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TokenUsageCursorDto {
    pub before_timestamp: String,
    pub before_event_id: String,
}

#[derive(Debug, Default, Deserialize, IntoParams)]
pub struct TokenUsageCursorQuery {
    pub before_timestamp: Option<String>,
    pub before_event_id: Option<String>,
}

impl TokenUsageCursorQuery {
    fn into_cursor(self) -> Result<Option<TokenUsageCursorDto>, AppError> {
        match (self.before_timestamp, self.before_event_id) {
            (None, None) => Ok(None),
            (Some(timestamp), Some(event_id)) if !event_id.is_empty() => {
                let parsed = DateTime::parse_from_rfc3339(&timestamp)
                    .map_err(|_| AppError::Validation("Invalid token-usage cursor timestamp".into()))?
                    .with_timezone(&Utc);
                Ok(Some(TokenUsageCursorDto {
                    before_timestamp: parsed.to_rfc3339_opts(SecondsFormat::Nanos, true),
                    before_event_id: event_id,
                }))
            }
            _ => Err(AppError::Validation(
                "Token-usage cursor requires a timestamp and event id".into(),
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TokenUsageResponseDto {
    pub items: Vec<TokenUsageRowDto>,
    pub totals: TokenUsageTotalsDto,
    pub next_cursor: Option<TokenUsageCursorDto>,
}

pub fn token_usage_routes(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/usage/tokens", get(user_token_usage))
        .route("/api/v1/admin/usage/tokens", get(admin_token_usage))
        .with_state(state)
}

#[utoipa::path(
    get,
    path = "/api/v1/usage/tokens",
    params(TokenUsageCursorQuery),
    responses((status = 200, description = "Token usage for the authenticated user", body = TokenUsageResponseDto), (status = 401, description = "Unauthenticated")),
    security(("bearerAuth" = []))
)]
pub async fn user_token_usage(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Query(query): Query<TokenUsageCursorQuery>,
) -> Result<Json<TokenUsageResponseDto>, AppError> {
    let cursor = query.into_cursor()?;
    load_token_usage(&state, Some(&auth_user.user_id.0), false, cursor).await
}

#[utoipa::path(
    get,
    path = "/api/v1/admin/usage/tokens",
    params(TokenUsageCursorQuery),
    responses((status = 200, description = "Token usage across all users", body = TokenUsageResponseDto), (status = 403, description = "Administrator privileges required")),
    security(("bearerAuth" = []))
)]
pub async fn admin_token_usage(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Query(query): Query<TokenUsageCursorQuery>,
) -> Result<Json<TokenUsageResponseDto>, AppError> {
    let cursor = query.into_cursor()?;
    let mut role_result = state
        .db
        .client
        .query("SELECT role FROM type::record('user', $user_id);")
        .bind(("user_id", auth_user.user_id.0.clone()))
        .await
        .map_err(|error| AppError::Database(error.to_string()))?;
    let role: Option<serde_json::Value> = role_result.take(0).unwrap_or_default();
    if role
        .as_ref()
        .and_then(|row| row.get("role"))
        .and_then(|role| role.as_str())
        != Some("admin")
    {
        return Err(AppError::Forbidden(
            "Administrator privileges required".into(),
        ));
    }
    load_token_usage(&state, None, true, cursor).await
}

async fn load_token_usage(
    state: &AppState,
    user_id: Option<&str>,
    include_user: bool,
    cursor: Option<TokenUsageCursorDto>,
) -> Result<Json<TokenUsageResponseDto>, AppError> {
    let db = &state.db.client;
    let (rows_query, totals_query) = match (user_id.is_some(), cursor.is_some()) {
        (true, true) => (
            "SELECT event_id AS id, event_id, run_id.id AS run_id, execution_id, payload.agent_id AS agent_id, payload.model_provider AS model_provider, payload.model_name AS model_name, payload.input_tokens AS input_tokens, payload.output_tokens AS output_tokens, payload.total_tokens AS total_tokens, ts, <string>ts AS timestamp FROM execution_event WHERE type = $event_type AND run_id.user_id = $user_id AND (ts < <datetime>$before_timestamp OR (ts = <datetime>$before_timestamp AND event_id < $before_event_id)) ORDER BY ts DESC, event_id DESC LIMIT $limit;",
            "SELECT count() AS request_count, math::sum(payload.input_tokens) AS input_tokens, math::sum(payload.output_tokens) AS output_tokens, math::sum(payload.total_tokens) AS total_tokens FROM execution_event WHERE type = $event_type AND run_id.user_id = $user_id GROUP ALL;",
        ),
        (true, false) => (
            "SELECT event_id AS id, event_id, run_id.id AS run_id, execution_id, payload.agent_id AS agent_id, payload.model_provider AS model_provider, payload.model_name AS model_name, payload.input_tokens AS input_tokens, payload.output_tokens AS output_tokens, payload.total_tokens AS total_tokens, ts, <string>ts AS timestamp FROM execution_event WHERE type = $event_type AND run_id.user_id = $user_id ORDER BY ts DESC, event_id DESC LIMIT $limit;",
            "SELECT count() AS request_count, math::sum(payload.input_tokens) AS input_tokens, math::sum(payload.output_tokens) AS output_tokens, math::sum(payload.total_tokens) AS total_tokens FROM execution_event WHERE type = $event_type AND run_id.user_id = $user_id GROUP ALL;",
        ),
        (false, true) => (
            "SELECT event_id AS id, event_id, run_id.id AS run_id, execution_id, payload.agent_id AS agent_id, payload.model_provider AS model_provider, payload.model_name AS model_name, payload.input_tokens AS input_tokens, payload.output_tokens AS output_tokens, payload.total_tokens AS total_tokens, ts, <string>ts AS timestamp, run_id.user_id AS user_id FROM execution_event WHERE type = $event_type AND (ts < <datetime>$before_timestamp OR (ts = <datetime>$before_timestamp AND event_id < $before_event_id)) ORDER BY ts DESC, event_id DESC LIMIT $limit;",
            "SELECT count() AS request_count, math::sum(payload.input_tokens) AS input_tokens, math::sum(payload.output_tokens) AS output_tokens, math::sum(payload.total_tokens) AS total_tokens FROM execution_event WHERE type = $event_type GROUP ALL;",
        ),
        (false, false) => (
            "SELECT event_id AS id, event_id, run_id.id AS run_id, execution_id, payload.agent_id AS agent_id, payload.model_provider AS model_provider, payload.model_name AS model_name, payload.input_tokens AS input_tokens, payload.output_tokens AS output_tokens, payload.total_tokens AS total_tokens, ts, <string>ts AS timestamp, run_id.user_id AS user_id FROM execution_event WHERE type = $event_type ORDER BY ts DESC, event_id DESC LIMIT $limit;",
            "SELECT count() AS request_count, math::sum(payload.input_tokens) AS input_tokens, math::sum(payload.output_tokens) AS output_tokens, math::sum(payload.total_tokens) AS total_tokens FROM execution_event WHERE type = $event_type GROUP ALL;",
        ),
    };

    let mut rows_result = db
        .query(rows_query)
        .bind(("event_type", TOKEN_USAGE_EVENT))
        .bind(("limit", PAGE_SIZE + 1))
        .bind(("user_id", user_id.unwrap_or_default().to_owned()))
        .bind((
            "before_timestamp",
            cursor
                .as_ref()
                .map(|value| value.before_timestamp.clone())
                .unwrap_or_default(),
        ))
        .bind((
            "before_event_id",
            cursor
                .as_ref()
                .map(|value| value.before_event_id.clone())
                .unwrap_or_default(),
        ))
        .await
        .map_err(|error| AppError::Database(error.to_string()))?;
    let rows: Vec<serde_json::Value> = rows_result.take(0).unwrap_or_default();
    let mut items: Vec<TokenUsageRowDto> = rows
        .into_iter()
        .filter_map(|row| {
            let id = row.get("id")?.as_str()?.to_owned();
            let value = |key: &str| {
                row.get(key)
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default()
                    .to_owned()
            };
            Some(TokenUsageRowDto {
                id,
                run_id: value("run_id"),
                execution_id: row
                    .get("execution_id")
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_owned),
                agent_id: value("agent_id"),
                model_provider: value("model_provider"),
                model_name: value("model_name"),
                input_tokens: row
                    .get("input_tokens")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or_default(),
                output_tokens: row
                    .get("output_tokens")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or_default(),
                total_tokens: row
                    .get("total_tokens")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or_default(),
                timestamp: value("timestamp"),
                user_id: include_user.then(|| value("user_id")),
            })
        })
        .collect();
    let has_more = items.len() > PAGE_SIZE;
    items.truncate(PAGE_SIZE);
    let next_cursor = if has_more {
        items.last().map(|item| TokenUsageCursorDto {
            before_timestamp: item.timestamp.clone(),
            before_event_id: item.id.clone(),
        })
    } else {
        None
    };

    let mut totals_result = db
        .query(totals_query)
        .bind(("event_type", TOKEN_USAGE_EVENT))
        .bind(("user_id", user_id.unwrap_or_default().to_owned()))
        .await
        .map_err(|error| AppError::Database(error.to_string()))?;
    let totals_row: Option<serde_json::Value> = totals_result.take(0).unwrap_or_default();
    let number = |key: &str| {
        totals_row
            .as_ref()
            .and_then(|row| row.get(key))
            .and_then(serde_json::Value::as_u64)
            .unwrap_or_default()
    };

    Ok(Json(TokenUsageResponseDto {
        items,
        totals: TokenUsageTotalsDto {
            request_count: number("request_count"),
            input_tokens: number("input_tokens"),
            output_tokens: number("output_tokens"),
            total_tokens: number("total_tokens"),
        },
        next_cursor,
    }))
}
