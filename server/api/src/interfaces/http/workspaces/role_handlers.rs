use axum::{
    extract::{Path, State},
    Json,
};
use serde::Deserialize;

use super::auth_check::verify_workspace_owner;
use super::AppState;
use crate::interfaces::http::extractors::authenticated_user::AuthenticatedUser;
use crate::shared::error::AppError;

pub async fn get_workspace_role(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(id): Path<String>,
) -> std::result::Result<Json<serde_json::Value>, AppError> {
    let clean_id = id.replace("workspace:", "").replace('`', "");
    verify_workspace_owner(&state.db.client, &clean_id, &auth_user.user_id.0).await?;
    let db = &state.db.client;

    let mut res_jd = db
        .query("SELECT * FROM job_description WHERE workspace_id = type::record('workspace', $id) OR workspace_id = $id;")
        .bind(("id", clean_id.clone()))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let jd_opt: Option<serde_json::Value> = res_jd.take(0).unwrap_or_default();

    let mut res_ws = db
        .query("SELECT * FROM type::record('workspace', $id);")
        .bind(("id", clean_id.clone()))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let mut ws_opt: Option<serde_json::Value> = res_ws.take(0).unwrap_or_default();
    if ws_opt.is_none() {
        let mut res_ws_raw = db
            .query("SELECT * FROM workspace WHERE id = type::record('workspace', $id) OR id = $id;")
            .bind(("id", clean_id.clone()))
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        ws_opt = res_ws_raw.take(0).unwrap_or_default();
    }

    let role_title = ws_opt
        .as_ref()
        .and_then(|w| w["role_title"].as_str())
        .unwrap_or("Software Engineer");
    let company_name = ws_opt
        .as_ref()
        .and_then(|w| w["company_name"].as_str())
        .unwrap_or("Target Company");

    if let Some(jd) = jd_opt {
        return Ok(Json(serde_json::json!({
            "workspace_id": id,
            "title": jd.get("title").and_then(|t| t.as_str()).unwrap_or(role_title),
            "company_name": company_name,
            "role_summary": jd.get("role_summary").and_then(|s| s.as_str()).unwrap_or(""),
            "raw_text": jd.get("raw_text").and_then(|t| t.as_str()).unwrap_or(""),
            "requirements": jd.get("requirements").cloned().unwrap_or(serde_json::json!([])),
            "responsibilities": jd.get("responsibilities").cloned().unwrap_or(serde_json::json!([])),
        })));
    }

    Ok(Json(serde_json::json!({
        "workspace_id": id,
        "title": role_title,
        "company_name": company_name,
        "role_summary": "",
        "raw_text": "",
        "requirements": [],
        "responsibilities": []
    })))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImportJdUrlRequestDto {
    pub url: String,
    pub custom_title: Option<String>,
}

pub async fn import_job_description_url(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(id): Path<String>,
    Json(payload): Json<ImportJdUrlRequestDto>,
) -> std::result::Result<Json<serde_json::Value>, AppError> {
    let clean_id = id.replace("workspace:", "").replace('`', "");
    verify_workspace_owner(&state.db.client, &clean_id, &auth_user.user_id.0).await?;
    let db = &state.db.client;

    let jd_id = format!("jd_{clean_id}");
    let query = "
        UPSERT type::record('job_description', $id) SET
            workspace_id = type::record('workspace', $ws_id),
            jd_url = $jd_url,
            source_url = $jd_url,
            title = $title,
            raw_text = 'Importing from URL...',
            extraction_method = 'url_import',
            requirements = [];
    ";
    let title = payload.custom_title.unwrap_or_else(|| "Imported Role".into());
    let _ = db
        .query(query)
        .bind(("id", jd_id))
        .bind(("ws_id", clean_id.clone()))
        .bind(("jd_url", payload.url.clone()))
        .bind(("title", title))
        .await
        .map_err(|e| AppError::Database(format!("Failed to record imported JD: {e}")))?;

    Ok(Json(serde_json::json!({
        "workspace_id": clean_id,
        "status": "imported",
        "source_url": payload.url
    })))
}
