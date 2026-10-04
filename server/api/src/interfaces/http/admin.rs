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

pub fn admin_routes(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/admin/pipelines", get(list_pipelines))
        .route("/api/v1/admin/pipelines/{workspace_id}", get(get_pipeline_detail))
        .route("/api/v1/admin/pipelines/{workspace_id}/logs", get(get_pipeline_logs))
        .route("/api/v1/admin/pipelines/{workspace_id}/events", get(stream_pipeline_events))
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
