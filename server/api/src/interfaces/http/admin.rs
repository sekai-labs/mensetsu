use std::{convert::Infallible, time::Duration};

use axum::{
    extract::{Path, State},
    response::sse::{Event, KeepAlive, Sse},
    routing::get,
    Json, Router,
};
use tokio_stream::Stream;

use crate::{
    interfaces::http::{
        extractors::authenticated_user::AuthenticatedUser, workspaces::AppState,
    },
    shared::error::AppError,
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct AdminProviderDto {
    pub provider_key: String,
    pub name: String,
    pub base_url: Option<String>,
    pub encrypted_api_key: Option<String>,
    pub is_enabled: bool,
    pub allowed_models: Vec<String>,
    pub has_key: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct UpsertAdminProviderRequestDto {
    pub provider_key: String,
    pub name: Option<String>,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub is_enabled: Option<bool>,
    pub allowed_models: Option<Vec<String>>,
}

pub fn admin_routes(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/admin/pipelines", get(list_pipelines))
        .route("/api/v1/admin/pipelines/{workspace_id}", get(get_pipeline_detail))
        .route("/api/v1/admin/pipelines/{workspace_id}/logs", get(get_pipeline_logs))
        .route("/api/v1/admin/pipelines/{workspace_id}/events", get(stream_pipeline_events))
        .route("/api/v1/admin/providers", get(list_admin_providers).post(upsert_admin_provider))
        .route("/api/v1/admin/providers/{provider_key}", axum::routing::put(upsert_admin_provider))
        .route("/api/v1/admin/settings", get(get_admin_settings).put(update_admin_settings))
        .with_state(state)
}

async fn verify_admin(state: &AppState, user_id: &str) -> Result<(), AppError> {
    let db = &state.db.client;
    let mut res = db
        .query("SELECT role FROM type::record('user', $uid);")
        .bind(("uid", user_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let row: Option<serde_json::Value> = res.take(0).unwrap_or_default();
    let role = row
        .as_ref()
        .and_then(|r| r.get("role"))
        .and_then(|v| v.as_str())
        .unwrap_or("member");

    if role != "admin" {
        return Err(AppError::Forbidden("Administrator privileges required".into()));
    }
    Ok(())
}

pub async fn list_pipelines(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>, AppError> {
    verify_admin(&state, &auth_user.user_id.0).await?;
    let db = &state.db.client;

    let query = "
        SELECT 
            id,
            company_name,
            role_title,
            status,
            current_phase,
            progress_percent,
            created_at,
            updated_at
        FROM workspace
        ORDER BY created_at DESC
        LIMIT 50;
    ";
    let mut res = db
        .query(query)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let workspaces: Vec<serde_json::Value> = res.take(0).unwrap_or_default();

    let mut pipelines = Vec::new();
    for ws in workspaces {
        let ws_id = ws["id"].as_str().unwrap_or_default().replace("workspace:", "").replace('`', "");
        let mut exec_res = db
            .query("SELECT * FROM type::record('pipeline_execution', $eid);")
            .bind(("eid", format!("exec_{ws_id}")))
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        let exec: Option<serde_json::Value> = exec_res.take(0).unwrap_or_default();

        pipelines.push(serde_json::json!({
            "workspace_id": ws_id,
            "company_name": ws["company_name"],
            "role_title": ws["role_title"],
            "status": ws["status"],
            "current_phase": ws["current_phase"],
            "progress_percent": ws["progress_percent"],
            "created_at": ws["created_at"],
            "execution": exec,
        }));
    }

    Ok(Json(serde_json::json!({ "pipelines": pipelines })))
}

pub async fn get_pipeline_detail(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(workspace_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    verify_admin(&state, &auth_user.user_id.0).await?;
    let db = &state.db.client;
    let clean_id = workspace_id.replace("workspace:", "").replace('`', "");

    let mut ws_res = db
        .query("SELECT * FROM type::record('workspace', $wid);")
        .bind(("wid", clean_id.clone()))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let ws: Option<serde_json::Value> = ws_res.take(0).unwrap_or_default();

    let mut exec_res = db
        .query("SELECT * FROM type::record('pipeline_execution', $eid);")
        .bind(("eid", format!("exec_{clean_id}")))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let exec: Option<serde_json::Value> = exec_res.take(0).unwrap_or_default();

    Ok(Json(serde_json::json!({
        "workspace_id": clean_id,
        "workspace": ws,
        "execution": exec,
    })))
}

pub async fn get_pipeline_logs(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(workspace_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    verify_admin(&state, &auth_user.user_id.0).await?;
    let db = &state.db.client;
    let clean_id = workspace_id.replace("workspace:", "").replace('`', "");

    let query = "
        SELECT * FROM pipeline_log
        WHERE workspace_id = type::record('workspace', $wid) OR workspace_id = $wid
        ORDER BY timestamp ASC
        LIMIT 200;
    ";
    let mut res = db
        .query(query)
        .bind(("wid", clean_id.clone()))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let logs: Vec<serde_json::Value> = res.take(0).unwrap_or_default();

    Ok(Json(serde_json::json!({
        "workspace_id": clean_id,
        "logs": logs
    })))
}

pub async fn stream_pipeline_events(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(workspace_id): Path<String>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, AppError> {
    verify_admin(&state, &auth_user.user_id.0).await?;
    let clean_id = workspace_id.replace("workspace:", "").replace('`', "");
    let db = state.db.client.clone();

    let stream = async_stream::stream! {
        let mut last_seen_count = 0;
        loop {
            let query = "
                SELECT * FROM pipeline_log
                WHERE workspace_id = type::record('workspace', $wid) OR workspace_id = $wid
                ORDER BY timestamp ASC;
            ";
            if let Ok(mut res) = db.query(query).bind(("wid", clean_id.clone())).await {
                let logs: Vec<serde_json::Value> = res.take(0).unwrap_or_default();
                if logs.len() > last_seen_count {
                    for log in &logs[last_seen_count..] {
                        let payload = serde_json::to_string(log).unwrap_or_default();
                        yield Ok(Event::default().event("log").data(payload));
                    }
                    last_seen_count = logs.len();
                }
            }

            let ws_res = db
                .query("SELECT status, current_phase, progress_percent FROM type::record('workspace', $wid);")
                .bind(("wid", clean_id.clone()))
                .await;
            if let Ok(mut r) = ws_res {
                let ws: Option<serde_json::Value> = r.take(0).unwrap_or_default();
                if let Some(w) = ws {
                    let payload = serde_json::to_string(&w).unwrap_or_default();
                    yield Ok(Event::default().event("status").data(payload));
                    if w["status"] == "ready" || w["status"] == "failed" {
                        break;
                    }
                }
            }

            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    };

    Ok(Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(5))))
}

#[utoipa::path(
    get,
    path = "/api/v1/admin/providers",
    responses(
        (status = 200, description = "Admin platform providers list", body = [AdminProviderDto]),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn list_admin_providers(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<Json<Vec<AdminProviderDto>>, AppError> {
    verify_admin(&state, &auth_user.user_id.0).await?;
    let db = &state.db.client;

    let mut res = db
        .query("SELECT * FROM system_llm_provider;")
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let rows: Vec<serde_json::Value> = res.take(0).unwrap_or_default();

    let mut providers: Vec<AdminProviderDto> = rows.into_iter().map(|r| {
        let pkey = r["provider_key"].as_str().unwrap_or_default().to_string();
        let name = r["name"].as_str().unwrap_or(&pkey).to_string();
        let base_url = r["base_url"].as_str().map(|s| s.to_string());
        let enc_key = r["encrypted_api_key"].as_str();
        let has_key = enc_key.map(|k| !k.is_empty()).unwrap_or(false);
        let is_enabled = r["is_enabled"].as_bool().unwrap_or(true);
        let allowed_models = r["allowed_models"]
            .as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();

        AdminProviderDto {
            provider_key: pkey,
            name,
            base_url,
            encrypted_api_key: None,
            is_enabled,
            allowed_models,
            has_key,
        }
    }).collect();

    if providers.is_empty() {
        let defaults = [
            ("openai", "OpenAI"),
            ("anthropic", "Anthropic"),
            ("gemini", "Google Gemini"),
            ("openrouter", "OpenRouter"),
            ("ollama", "Ollama (Local)"),
        ];
        for (key, name) in defaults {
            providers.push(AdminProviderDto {
                provider_key: key.to_string(),
                name: name.to_string(),
                base_url: None,
                encrypted_api_key: None,
                is_enabled: true,
                allowed_models: Vec::new(),
                has_key: false,
            });
        }
    }

    Ok(Json(providers))
}

#[utoipa::path(
    post,
    path = "/api/v1/admin/providers",
    request_body = UpsertAdminProviderRequestDto,
    responses(
        (status = 200, description = "Admin platform provider updated"),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn upsert_admin_provider(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Json(payload): Json<UpsertAdminProviderRequestDto>,
) -> Result<axum::http::StatusCode, AppError> {
    verify_admin(&state, &auth_user.user_id.0).await?;
    let db = &state.db.client;
    let crypto = crate::shared::encryption::CryptoService::new(&state.config.encryption_key);

    let pkey = payload.provider_key.to_lowercase();
    let name = payload.name.unwrap_or_else(|| pkey.clone());
    let is_enabled = payload.is_enabled.unwrap_or(true);
    let allowed_models = payload.allowed_models.unwrap_or_default();

    let enc_key = match payload.api_key.as_deref() {
        Some(raw) if !raw.trim().is_empty() => Some(crypto.encrypt(raw.trim())?),
        _ => None,
    };

    let query = if enc_key.is_some() {
        "
        UPSERT type::record('system_llm_provider', $pkey) SET
            provider_key = $pkey,
            name = $name,
            base_url = $base_url,
            encrypted_api_key = $enc_key,
            is_enabled = $is_enabled,
            allowed_models = $allowed_models,
            updated_at = time::now();
        "
    } else {
        "
        UPSERT type::record('system_llm_provider', $pkey) SET
            provider_key = $pkey,
            name = $name,
            base_url = $base_url,
            is_enabled = $is_enabled,
            allowed_models = $allowed_models,
            updated_at = time::now();
        "
    };

    let mut q = db.query(query)
        .bind(("pkey", pkey.clone()))
        .bind(("name", name))
        .bind(("base_url", payload.base_url))
        .bind(("is_enabled", is_enabled))
        .bind(("allowed_models", allowed_models));

    if let Some(ek) = enc_key {
        q = q.bind(("enc_key", ek));
    }

    q.await
        .and_then(|r| r.check())
        .map_err(|e| AppError::Database(format!("Failed to save admin provider: {e}")))?;

    Ok(axum::http::StatusCode::OK)
}

#[utoipa::path(
    get,
    path = "/api/v1/admin/settings",
    responses(
        (status = 200, description = "Admin dynamic system settings", body = serde_json::Value),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn get_admin_settings(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>, AppError> {
    verify_admin(&state, &auth_user.user_id.0).await?;
    let db = &state.db.client;

    let mut res = db
        .query("SELECT key, value, is_secret, updated_at FROM `system_config`;")
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let rows: Vec<serde_json::Value> = res.take(0).unwrap_or_default();

    let mut map = serde_json::Map::new();
    for r in rows {
        if let Some(key) = r["key"].as_str() {
            map.insert(key.to_string(), r["value"].clone());
        }
    }

    Ok(Json(serde_json::Value::Object(map)))
}

#[utoipa::path(
    put,
    path = "/api/v1/admin/settings",
    request_body = serde_json::Value,
    responses(
        (status = 200, description = "Admin settings updated"),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn update_admin_settings(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Json(payload): Json<serde_json::Value>,
) -> Result<axum::http::StatusCode, AppError> {
    verify_admin(&state, &auth_user.user_id.0).await?;
    let db = &state.db.client;

    if let Some(map) = payload.as_object() {
        for (key, val) in map {
            let query = "
                UPSERT type::record('system_config', $key) SET
                    key = $key,
                    value = $val,
                    is_secret = false,
                    updated_by = type::record('user', $uid),
                    updated_at = time::now();
            ";
            let query = query.replace("system_config", "`system_config`");
            db.query(query)
                .bind(("key", key.clone()))
                .bind(("val", val.clone()))
                .bind(("uid", auth_user.user_id.0.clone()))
                .await
                .and_then(|r| r.check())
                .map_err(|e| AppError::Database(format!("Failed to update config {key}: {e}")))?;
        }
    }

    Ok(axum::http::StatusCode::OK)
}

