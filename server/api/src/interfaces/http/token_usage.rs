use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use chrono::{DateTime, SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::{
    contexts::workspace::infrastructure::db::DatabasePool,
    interfaces::http::{extractors::authenticated_user::AuthenticatedUser, workspaces::AppState},
    shared::error::AppError,
};

pub const TOKEN_USAGE_EVENT: &str = "agent.model_usage";
pub const DEFAULT_LIMIT: usize = 25;
pub const MAX_LIMIT: usize = 100;

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
    /// Scope-wide total count of model usage requests
    pub request_count: u64,
    /// Scope-wide total input tokens
    pub input_tokens: u64,
    /// Scope-wide total output tokens
    pub output_tokens: u64,
    /// Scope-wide total tokens (input + output)
    pub total_tokens: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TokenUsageCursorDto {
    pub before_timestamp: String,
    pub before_event_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DailyTokenUsageDto {
    /// Date of recorded usage in YYYY-MM-DD format
    pub date: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_tokens: u64,
    pub request_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DailyTokenUsageResponseDto {
    /// Daily token usage items sorted in ascending chronological order
    pub items: Vec<DailyTokenUsageDto>,
    /// Aggregate totals over the 30-day analytics period
    pub totals: TokenUsageTotalsDto,
}

#[derive(Debug, Default, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct TokenUsageQuery {
    pub before_timestamp: Option<String>,
    pub before_event_id: Option<String>,
    pub limit: Option<usize>,
    pub q: Option<String>,
    pub search: Option<String>,
    pub provider: Option<String>,
    pub model_provider: Option<String>,
    pub agent: Option<String>,
    pub agent_id: Option<String>,
    pub from: Option<String>,
    pub from_date: Option<String>,
    pub start_date: Option<String>,
    pub to: Option<String>,
    pub to_date: Option<String>,
    pub end_date: Option<String>,
    pub user_id: Option<String>,
}

pub type TokenUsageCursorQuery = TokenUsageQuery;

impl TokenUsageQuery {
    fn into_cursor(&self) -> Result<Option<TokenUsageCursorDto>, AppError> {
        match (&self.before_timestamp, &self.before_event_id) {
            (None, None) => Ok(None),
            (Some(timestamp), Some(event_id)) if !event_id.is_empty() => {
                let parsed = DateTime::parse_from_rfc3339(timestamp)
                    .map_err(|_| {
                        AppError::Validation("Invalid token-usage cursor timestamp".into())
                    })?
                    .with_timezone(&Utc);
                Ok(Some(TokenUsageCursorDto {
                    before_timestamp: parsed.to_rfc3339_opts(SecondsFormat::Nanos, true),
                    before_event_id: event_id.clone(),
                }))
            }
            _ => Err(AppError::Validation(
                "Token-usage cursor requires a timestamp and event id".into(),
            )),
        }
    }
}

fn parse_date_filter(val: Option<&str>, is_end_of_day: bool) -> Result<Option<String>, AppError> {
    let val = match val {
        Some(v) if !v.trim().is_empty() => v.trim(),
        _ => return Ok(None),
    };
    if let Ok(dt) = DateTime::parse_from_rfc3339(val) {
        return Ok(Some(
            dt.with_timezone(&Utc)
                .to_rfc3339_opts(SecondsFormat::Nanos, true),
        ));
    }
    if let Ok(naive_date) = chrono::NaiveDate::parse_from_str(val, "%Y-%m-%d") {
        let naive_dt = if is_end_of_day {
            naive_date
                .and_hms_nano_opt(23, 59, 59, 999_999_999)
                .unwrap()
        } else {
            naive_date.and_hms_nano_opt(0, 0, 0, 0).unwrap()
        };
        let dt = DateTime::<Utc>::from_naive_utc_and_offset(naive_dt, Utc);
        return Ok(Some(dt.to_rfc3339_opts(SecondsFormat::Nanos, true)));
    }
    Err(AppError::Validation(format!(
        "Invalid date format: '{}'. Expected RFC3339 timestamp or YYYY-MM-DD",
        val
    )))
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TokenUsageResponseDto {
    pub items: Vec<TokenUsageRowDto>,
    /// Scope-wide token totals across all requests in the user or platform scope (independent of page cursor or filters)
    pub totals: TokenUsageTotalsDto,
    pub next_cursor: Option<TokenUsageCursorDto>,
}

pub fn token_usage_routes(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/usage/tokens", get(user_token_usage))
        .route("/api/v1/admin/usage/tokens", get(admin_token_usage))
        .route("/api/v1/usage/tokens/daily", get(user_daily_token_usage))
        .route(
            "/api/v1/admin/usage/tokens/daily",
            get(admin_daily_token_usage),
        )
        .route(
            "/api/v1/analytics/tokens/daily",
            get(user_daily_token_usage),
        )
        .route(
            "/api/v1/admin/analytics/tokens/daily",
            get(admin_daily_token_usage),
        )
        .with_state(state)
}

#[utoipa::path(
    get,
    path = "/api/v1/usage/tokens",
    params(TokenUsageQuery),
    responses(
        (status = 200, description = "Token usage for the authenticated user", body = TokenUsageResponseDto),
        (status = 401, description = "Unauthenticated")
    ),
    security(("bearerAuth" = []))
)]
pub async fn user_token_usage(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Query(query): Query<TokenUsageQuery>,
) -> Result<Json<TokenUsageResponseDto>, AppError> {
    let cursor = query.into_cursor()?;
    let limit = query.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let search = query
        .q
        .as_deref()
        .or(query.search.as_deref())
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let provider = query
        .provider
        .as_deref()
        .or(query.model_provider.as_deref())
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let agent = query
        .agent
        .as_deref()
        .or(query.agent_id.as_deref())
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let from_date = parse_date_filter(
        query
            .from
            .as_deref()
            .or(query.from_date.as_deref())
            .or(query.start_date.as_deref()),
        false,
    )?;
    let to_date = parse_date_filter(
        query
            .to
            .as_deref()
            .or(query.to_date.as_deref())
            .or(query.end_date.as_deref()),
        true,
    )?;

    load_token_usage(
        &state,
        Some(&auth_user.user_id.0),
        false,
        cursor,
        limit,
        search,
        provider,
        agent,
        from_date.as_deref(),
        to_date.as_deref(),
        None,
    )
    .await
}

#[utoipa::path(
    get,
    path = "/api/v1/admin/usage/tokens",
    params(TokenUsageQuery),
    responses(
        (status = 200, description = "Token usage across all users", body = TokenUsageResponseDto),
        (status = 403, description = "Administrator privileges required")
    ),
    security(("bearerAuth" = []))
)]
pub async fn admin_token_usage(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Query(query): Query<TokenUsageQuery>,
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

    let limit = query.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let search = query
        .q
        .as_deref()
        .or(query.search.as_deref())
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let provider = query
        .provider
        .as_deref()
        .or(query.model_provider.as_deref())
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let agent = query
        .agent
        .as_deref()
        .or(query.agent_id.as_deref())
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let from_date = parse_date_filter(
        query
            .from
            .as_deref()
            .or(query.from_date.as_deref())
            .or(query.start_date.as_deref()),
        false,
    )?;
    let to_date = parse_date_filter(
        query
            .to
            .as_deref()
            .or(query.to_date.as_deref())
            .or(query.end_date.as_deref()),
        true,
    )?;
    let filter_user_id = query
        .user_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());

    load_token_usage(
        &state,
        None,
        true,
        cursor,
        limit,
        search,
        provider,
        agent,
        from_date.as_deref(),
        to_date.as_deref(),
        filter_user_id,
    )
    .await
}

#[utoipa::path(
    get,
    path = "/api/v1/usage/tokens/daily",
    responses(
        (status = 200, description = "Daily token usage analytics for the last 30 days for authenticated user", body = DailyTokenUsageResponseDto),
        (status = 401, description = "Unauthenticated")
    ),
    security(("bearerAuth" = []))
)]
pub async fn user_daily_token_usage(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<Json<DailyTokenUsageResponseDto>, AppError> {
    load_daily_token_usage(&state, Some(&auth_user.user_id.0)).await
}

#[utoipa::path(
    get,
    path = "/api/v1/admin/usage/tokens/daily",
    responses(
        (status = 200, description = "Daily token usage analytics for the last 30 days across all users", body = DailyTokenUsageResponseDto),
        (status = 403, description = "Administrator privileges required")
    ),
    security(("bearerAuth" = []))
)]
pub async fn admin_daily_token_usage(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<Json<DailyTokenUsageResponseDto>, AppError> {
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
    load_daily_token_usage(&state, None).await
}

pub async fn load_scope_totals(
    db: &DatabasePool,
    user_id: Option<&str>,
) -> Result<TokenUsageTotalsDto, AppError> {
    let totals_query = if user_id.is_some() {
        "SELECT count() AS request_count, math::sum(payload.input_tokens) AS input_tokens, math::sum(payload.output_tokens) AS output_tokens, math::sum(payload.total_tokens) AS total_tokens FROM execution_event WHERE type = $event_type AND run_id.user_id = $scoped_user_id GROUP ALL;"
    } else {
        "SELECT count() AS request_count, math::sum(payload.input_tokens) AS input_tokens, math::sum(payload.output_tokens) AS output_tokens, math::sum(payload.total_tokens) AS total_tokens FROM execution_event WHERE type = $event_type GROUP ALL;"
    };

    let mut q = db
        .client
        .query(totals_query)
        .bind(("event_type", TOKEN_USAGE_EVENT));
    if let Some(uid) = user_id {
        q = q.bind(("scoped_user_id", uid.to_string()));
    }

    let mut totals_result = q
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

    Ok(TokenUsageTotalsDto {
        request_count: number("request_count"),
        input_tokens: number("input_tokens"),
        output_tokens: number("output_tokens"),
        total_tokens: number("total_tokens"),
    })
}

async fn load_token_usage(
    state: &AppState,
    user_id: Option<&str>,
    include_user: bool,
    cursor: Option<TokenUsageCursorDto>,
    limit: usize,
    search: Option<&str>,
    provider: Option<&str>,
    agent: Option<&str>,
    from_date: Option<&str>,
    to_date: Option<&str>,
    filter_user_id: Option<&str>,
) -> Result<Json<TokenUsageResponseDto>, AppError> {
    let db = &state.db;

    let mut conditions = vec!["type = $event_type".to_string()];
    if user_id.is_some() {
        conditions.push("run_id.user_id = $scoped_user_id".to_string());
    } else if filter_user_id.is_some() {
        conditions.push("run_id.user_id = $filter_user_id".to_string());
    }

    if cursor.is_some() {
        conditions.push("(ts < <datetime>$before_timestamp OR (ts = <datetime>$before_timestamp AND event_id < $before_event_id))".to_string());
    }

    if provider.is_some() {
        conditions.push("payload.model_provider = $filter_provider".to_string());
    }

    if agent.is_some() {
        conditions.push("payload.agent_id = $filter_agent".to_string());
    }

    if from_date.is_some() {
        conditions.push("ts >= <datetime>$from_date".to_string());
    }

    if to_date.is_some() {
        conditions.push("ts <= <datetime>$to_date".to_string());
    }

    if search.is_some() {
        if include_user {
            conditions.push("(string::lowercase(type::string(event_id)) CONTAINS $search_lower OR string::lowercase(type::string(run_id.id)) CONTAINS $search_lower OR (execution_id != NONE AND string::lowercase(type::string(execution_id)) CONTAINS $search_lower) OR string::lowercase(type::string(payload.agent_id)) CONTAINS $search_lower OR string::lowercase(type::string(payload.model_provider)) CONTAINS $search_lower OR string::lowercase(type::string(payload.model_name)) CONTAINS $search_lower OR (run_id.user_id != NONE AND string::lowercase(type::string(run_id.user_id)) CONTAINS $search_lower))".to_string());
        } else {
            conditions.push("(string::lowercase(type::string(event_id)) CONTAINS $search_lower OR string::lowercase(type::string(run_id.id)) CONTAINS $search_lower OR (execution_id != NONE AND string::lowercase(type::string(execution_id)) CONTAINS $search_lower) OR string::lowercase(type::string(payload.agent_id)) CONTAINS $search_lower OR string::lowercase(type::string(payload.model_provider)) CONTAINS $search_lower OR string::lowercase(type::string(payload.model_name)) CONTAINS $search_lower)".to_string());
        }
    }

    let where_clause = conditions.join(" AND ");
    let select_user = if include_user {
        ", run_id.user_id AS user_id"
    } else {
        ""
    };
    let rows_query = format!(
        "SELECT event_id AS id, event_id, run_id.id AS run_id, execution_id, payload.agent_id AS agent_id, payload.model_provider AS model_provider, payload.model_name AS model_name, payload.input_tokens AS input_tokens, payload.output_tokens AS output_tokens, payload.total_tokens AS total_tokens, ts, <string>ts AS timestamp{} FROM execution_event WHERE {} ORDER BY ts DESC, event_id DESC LIMIT $limit_plus_one;",
        select_user, where_clause
    );

    let mut q = db
        .client
        .query(&rows_query)
        .bind(("event_type", TOKEN_USAGE_EVENT))
        .bind(("limit_plus_one", (limit + 1) as i64));

    if let Some(uid) = user_id {
        q = q.bind(("scoped_user_id", uid.to_string()));
    }
    if let Some(f_uid) = filter_user_id {
        q = q.bind(("filter_user_id", f_uid.to_string()));
    }
    if let Some(c) = &cursor {
        q = q
            .bind(("before_timestamp", c.before_timestamp.clone()))
            .bind(("before_event_id", c.before_event_id.clone()));
    }
    if let Some(p) = provider {
        q = q.bind(("filter_provider", p.to_string()));
    }
    if let Some(a) = agent {
        q = q.bind(("filter_agent", a.to_string()));
    }
    if let Some(fd) = from_date {
        q = q.bind(("from_date", fd.to_string()));
    }
    if let Some(td) = to_date {
        q = q.bind(("to_date", td.to_string()));
    }
    if let Some(s) = search {
        q = q.bind(("search_lower", s.to_lowercase()));
    }

    let mut rows_result = q
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

    let has_more = items.len() > limit;
    items.truncate(limit);
    let next_cursor = if has_more {
        items.last().map(|item| TokenUsageCursorDto {
            before_timestamp: item.timestamp.clone(),
            before_event_id: item.id.clone(),
        })
    } else {
        None
    };

    // Scope-wide totals across user or platform scope
    let totals = load_scope_totals(db, user_id).await?;

    Ok(Json(TokenUsageResponseDto {
        items,
        totals,
        next_cursor,
    }))
}

fn daily_usage_date(timestamp: &str) -> Option<String> {
    DateTime::parse_from_rfc3339(timestamp)
        .ok()
        .map(|value| value.format("%Y-%m-%d").to_string())
        .or_else(|| timestamp.get(..10).map(str::to_owned))
}

async fn load_daily_token_usage(
    state: &AppState,
    user_id: Option<&str>,
) -> Result<Json<DailyTokenUsageResponseDto>, AppError> {
    let db = &state.db.client;
    let since = Utc::now() - chrono::Duration::days(30);
    let since_str = since.to_rfc3339_opts(SecondsFormat::Nanos, true);

    let query_str = if user_id.is_some() {
        "SELECT <string>ts AS timestamp, payload.input_tokens AS input_tokens, payload.output_tokens AS output_tokens, payload.total_tokens AS total_tokens FROM execution_event WHERE type = $event_type AND run_id.user_id = $scoped_user_id AND ts >= <datetime>$since;"
    } else {
        "SELECT <string>ts AS timestamp, payload.input_tokens AS input_tokens, payload.output_tokens AS output_tokens, payload.total_tokens AS total_tokens FROM execution_event WHERE type = $event_type AND ts >= <datetime>$since;"
    };

    let mut q = db
        .query(query_str)
        .bind(("event_type", TOKEN_USAGE_EVENT))
        .bind(("since", since_str));
    if let Some(uid) = user_id {
        q = q.bind(("scoped_user_id", uid.to_string()));
    }

    let mut res = q.await.map_err(|e| AppError::Database(e.to_string()))?;
    let rows: Vec<serde_json::Value> = res.take(0).unwrap_or_default();

    let mut daily_map: std::collections::BTreeMap<String, DailyTokenUsageDto> =
        std::collections::BTreeMap::new();
    let mut agg_requests = 0u64;
    let mut agg_input = 0u64;
    let mut agg_output = 0u64;
    let mut agg_total = 0u64;

    for row in rows {
        let timestamp_str = row.get("timestamp").and_then(|v| v.as_str()).unwrap_or("");
        let date = daily_usage_date(timestamp_str)
            .unwrap_or_else(|| Utc::now().format("%Y-%m-%d").to_string());

        let input = row
            .get("input_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let output = row
            .get("output_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let total = row
            .get("total_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(input + output);

        let entry = daily_map
            .entry(date.clone())
            .or_insert_with(|| DailyTokenUsageDto {
                date,
                input_tokens: 0,
                output_tokens: 0,
                total_tokens: 0,
                request_count: 0,
            });
        entry.input_tokens += input;
        entry.output_tokens += output;
        entry.total_tokens += total;
        entry.request_count += 1;

        agg_requests += 1;
        agg_input += input;
        agg_output += output;
        agg_total += total;
    }

    let items: Vec<DailyTokenUsageDto> = daily_map.into_values().collect();
    let totals = TokenUsageTotalsDto {
        request_count: agg_requests,
        input_tokens: agg_input,
        output_tokens: agg_output,
        total_tokens: agg_total,
    };

    Ok(Json(DailyTokenUsageResponseDto { items, totals }))
}

#[cfg(test)]
mod tests {
    use super::daily_usage_date;

    #[test]
    fn malformed_unicode_timestamp_does_not_panic_when_slicing_date() {
        assert_eq!(
            daily_usage_date("2026-01-15T10:00:00Z"),
            Some("2026-01-15".into())
        );
        assert_eq!(daily_usage_date("123456789é"), None);
    }
}
