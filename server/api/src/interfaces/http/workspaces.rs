pub mod auth_check;
pub mod event_handlers;
pub mod learning_handlers;
pub mod research_handlers;
pub mod role_handlers;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{de::DeserializeOwned, Deserialize};
use uuid::Uuid;

pub use auth_check::verify_workspace_owner;
pub use event_handlers::workspace_events;
pub use learning_handlers::{
    __path_toggle_learning_unit_completed, get_workspace_knowledge, get_workspace_learning,
    get_workspace_match, get_workspace_questions, toggle_learning_unit_completed,
};
pub use research_handlers::{
    ask_workspace_knowledge_qa, get_workspace_claims, get_workspace_company, get_workspace_sources,
    KnowledgeQaRequestDto,
};
pub use role_handlers::{get_workspace_role, import_job_description_url, ImportJdUrlRequestDto};

use crate::{
    contexts::{
        identity::domain::UserId,
        workspace::{
            application::dtos::{CreateWorkspaceRequestDto, PatchWorkspaceObjectiveDto, WorkspaceResponseDto},
            domain::WorkspaceId,
            infrastructure::db::DatabasePool,
        },
    },
    interfaces::http::{
        extractors::authenticated_user::AuthenticatedUser,
        workflow_dtos::WorkflowRun,
    },
    interfaces::grpc::{CreateRunPayload, GrpcClient, ResumePlanPayload},
    shared::{config::AppConfig, error::AppError},
};

#[derive(Clone)]
pub struct AppState {
    pub db: DatabasePool,
    pub config: AppConfig,
    pub identity_service: std::sync::Arc<crate::contexts::identity::application::IdentityService>,
    pub workspace_service: std::sync::Arc<crate::contexts::workspace::application::WorkspaceService>,
    pub workflow_service: std::sync::Arc<crate::contexts::workflow::application::WorkflowService>,
    pub grpc_client: std::sync::Arc<GrpcClient>,
    pub storage: std::sync::Arc<crate::contexts::workspace::infrastructure::storage::ResumeObjectStore>,
}

pub fn workspace_routes(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/workspaces", post(create_workspace).get(list_workspaces))
        .route("/api/v1/workspaces/{id}", get(get_workspace))
        .route("/api/v1/workspaces/{id}/objective", axum::routing::patch(update_workspace_objective))
        .route("/api/v1/workspaces/{id}/events", get(workspace_events))
        .route("/api/v1/workspaces/{id}/match", get(get_workspace_match))
        .route("/api/v1/workspaces/{id}/role", get(get_workspace_role))
        .route("/api/v1/workspaces/{id}/company", get(get_workspace_company))
        .route("/api/v1/workspaces/{id}/sources", get(get_workspace_sources))
        .route("/api/v1/workspaces/{id}/claims", get(get_workspace_claims))
        .route("/api/v1/workspaces/{id}/qa", post(ask_workspace_knowledge_qa))
        .route("/api/v1/workspaces/{id}/import-jd", post(import_job_description_url))
        .route("/api/v1/workspaces/{id}/knowledge", get(get_workspace_knowledge))
        .route("/api/v1/workspaces/{id}/learning", get(get_workspace_learning))
        .route("/api/v1/workspaces/{id}/learning/{lesson_id}/toggle", axum::routing::put(toggle_learning_unit_completed))
        .route("/api/v1/workspaces/{id}/questions", get(get_workspace_questions))
        .with_state(state)
}

#[utoipa::path(
    post,
    path = "/api/v1/workspaces",
    request_body = CreateWorkspaceRequestDto,
    responses(
        (status = 201, description = "Workspace created", body = WorkspaceResponseDto),
        (status = 400, description = "Invalid payload")
    )
)]
pub async fn create_workspace(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Json(payload): Json<CreateWorkspaceRequestDto>,
) -> std::result::Result<(StatusCode, Json<WorkspaceResponseDto>), AppError> {
    let user_id = auth_user.user_id;
    let clean_uid = user_id.0.replace("user:", "").replace('`', "");




    let mut validated_upload: Option<(String, String)> = None;

    if let Some(upload_id_raw) = &payload.resume_upload_id {
        let clean_upload_id = upload_id_raw.replace("resume_upload:", "").replace('`', "");
        if Uuid::parse_str(&clean_upload_id).is_err() {
            return Err(AppError::Validation("Invalid resume_upload_id format".to_string()));
        }

        if let Some(text) = &payload.resume_text {
            if !text.trim().is_empty() {
                return Err(AppError::Validation(
                    "Cannot provide both resume_upload_id and resume_text simultaneously".to_string(),
                ));
            }
        }

        let db = &state.db.client;
        let mut upload_res = db
            .query("SELECT user_id, filename, object_key, status, workspace_id FROM type::record('resume_upload', $upload_id);")
            .bind(("upload_id", clean_upload_id.clone()))
            .await
            .map_err(|e| AppError::Database(format!("Failed to lookup upload: {e}")))?;

        let rows: Vec<serde_json::Value> = upload_res.take(0).unwrap_or_default();
        let row = rows
            .into_iter()
            .next()
            .ok_or_else(|| AppError::NotFound(format!("Resume upload {clean_upload_id} not found")))?;

        let record_uid = match row.get("user_id") {
            Some(serde_json::Value::String(s)) => s.replace("user:", "").replace('`', ""),
            Some(serde_json::Value::Object(map)) => map
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .replace("user:", "")
                .replace('`', ""),
            _ => String::new(),
        };

        if record_uid != clean_uid {
            return Err(AppError::Forbidden("You do not own this resume upload".to_string()));
        }

        let status = row.get("status").and_then(|v| v.as_str()).unwrap_or("");
        if status != "completed" {
            return Err(AppError::Conflict(format!(
                "Resume upload status is '{status}', expected 'completed'"
            )));
        }

        if let Some(ws_val) = row.get("workspace_id") {
            if !ws_val.is_null() {
                return Err(AppError::Conflict(
                    "Resume upload is already attached to another workspace".to_string(),
                ));
            }
        }

        let filename = row
            .get("filename")
            .and_then(|v| v.as_str())
            .unwrap_or("resume.pdf")
            .to_string();
        let object_key = row
            .get("object_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::Validation("Resume upload missing object_key".to_string()))?
            .to_string();

        validated_upload = Some((filename, object_key));
    }


    let workspace = state
        .workspace_service
        .build_workspace(user_id.clone(), &payload)?;

    let db = &state.db.client;
    let ws_id = workspace.id.0.clone();

    let seniority_str = workspace
        .seniority
        .as_ref()
        .map(|s| serde_json::to_value(s).unwrap_or_default().as_str().unwrap_or("").to_string());
    let target_date_str = workspace.target_date.as_ref().map(|d| d.to_rfc3339());


    let mut statements = vec!["BEGIN TRANSACTION;".to_string()];

    if payload.resume_upload_id.is_some() {
        statements.push(
            "LET $updated_uploads = (UPDATE type::record('resume_upload', $upload_id) SET
                status = 'attached',
                workspace_id = type::record('workspace', $ws_id),
                updated_at = time::now()
             WHERE user_id = type::record('user', $user_id)
               AND status = 'completed'
               AND (workspace_id = NONE OR workspace_id = NULL)
               AND object_key = $source_object_key);
             IF array::len($updated_uploads) != 1 THEN
                 THROW 'Resume upload CAS attachment conflict: upload was not completed or already attached';
             END;".to_string(),
        );
    }

    statements.push(
        "UPSERT type::record('workspace', $ws_id) SET
            user_id = type::record('user', $user_id),
            company_name = $company_name,
            role_title = $role_title,
            status = $status,
            current_phase = $current_phase,
            progress_percent = $progress_percent,
            seniority = $seniority,
            target_date = $target_date,
            prep_time_weeks = $prep_time_weeks,
            user_prompt = $user_prompt,
            current_objective = $current_objective,
            workflow_mode = $workflow_mode,
            company_urls = $company_urls,
            created_at = type::datetime($created_at),
            updated_at = type::datetime($updated_at);".to_string(),
    );

    if payload.resume_upload_id.is_some() {
        statements.push(
            "UPSERT type::record('resume', $res_id) SET
                workspace_id = type::record('workspace', $ws_id),
                raw_text = '',
                source_upload_id = type::record('resume_upload', $upload_id),
                source_object_key = $source_object_key,
                source_filename = $source_filename,
                parsed_skills = [],
                experience = [],
                education = [],
                created_at = time::now();".to_string(),
        );
    } else if payload.resume_text.is_some() {
        statements.push(
            "UPSERT type::record('resume', $res_id) SET
                workspace_id = type::record('workspace', $ws_id),
                raw_text = $raw_text,
                parsed_skills = [],
                experience = [],
                education = [],
                created_at = time::now();".to_string(),
        );
    }

    if payload.job_description.is_some() || payload.jd_url.is_some() {
        statements.push(
            "UPSERT type::record('job_description', $jd_id) SET
                workspace_id = type::record('workspace', $ws_id),
                raw_text = $jd_raw_text,
                jd_url = $jd_url,
                requirements = [];".to_string(),
        );
    }

    statements.push("COMMIT TRANSACTION;".to_string());

    let tx_sql = statements.join("\n");

    let mut query_builder = db.query(&tx_sql)
        .bind(("ws_id", ws_id.clone()))
        .bind(("user_id", clean_uid.clone()))
        .bind(("company_name", workspace.company_name.clone()))
        .bind(("role_title", workspace.role_title.clone()))
        .bind(("status", "pending"))
        .bind(("current_phase", "preparing_input"))
        .bind(("progress_percent", 0i64))
        .bind(("seniority", seniority_str))
        .bind(("target_date", target_date_str))
        .bind(("prep_time_weeks", workspace.prep_time_weeks.map(|w| w as i64)))
        .bind(("user_prompt", workspace.user_prompt.clone()))
        .bind(("current_objective", workspace.current_objective.clone()))
        .bind(("workflow_mode", workspace.workflow_mode.clone()))
        .bind(("company_urls", payload.company_urls.clone()))
        .bind(("created_at", workspace.created_at.to_rfc3339()))
        .bind(("updated_at", workspace.updated_at.to_rfc3339()))
        .bind(("res_id", format!("res_{ws_id}")))
        .bind(("jd_id", format!("jd_{ws_id}")))
        .bind(("raw_text", payload.resume_text.clone().unwrap_or_default()))
        .bind(("jd_raw_text", payload.job_description.clone().unwrap_or_default()))
        .bind(("jd_url", payload.jd_url.clone()));

    if let Some(upload_id_raw) = &payload.resume_upload_id {
        let clean_upload_id = upload_id_raw.replace("resume_upload:", "").replace('`', "");
        let (upload_filename, upload_object_key) = validated_upload.as_ref().unwrap();
        query_builder = query_builder
            .bind(("upload_id", clean_upload_id))
            .bind(("source_filename", upload_filename.clone()))
            .bind(("source_object_key", upload_object_key.clone()));
    }

    let mut tx_res = query_builder
        .await
        .map_err(|e| AppError::Database(format!("Transaction execution failed: {e}")))?;

    let errors = tx_res.take_errors();
    if !errors.is_empty() {
        let error_msg = format!("{:?}", errors);
        if error_msg.contains("attachment conflict") {
            return Err(AppError::Conflict("Failed to attach resume upload: upload is no longer in completed state".to_string()));
        }
        return Err(AppError::Database(format!(
            "Workspace creation transaction failed: {error_msg}"
        )));
    }

    let mut status = workspace.status;
    let mut current_phase = workspace.current_phase;

    let workflow_id = if workspace.workflow_mode == "static" {
        "interview-preparation-static".to_string()
    } else if workspace.workflow_mode == "dynamic" {
        "interview-preparation-dynamic".to_string()
    } else {
        "interview-preparation-hybrid".to_string()
    };

    match start_workspace_run(
        &state,
        &user_id,
        &ws_id,
        workflow_id,
        Some(&ws_id),
    )
    .await
    {
        Ok(run) => {
            tracing::info!(workspace_id = %ws_id, run_id = %run.id, "triggered intelligence workflow run");
        }
        Err(e) => {
            tracing::error!(
                workspace_id = %ws_id,
                error = %e,
                "Failed to trigger intelligence workflow run; marking workspace failed"
            );
            if let Err(e) = state
                .workspace_service
                .update_progress(&workspace.id, crate::contexts::workspace::domain::PipelinePhase::Failed, 0)
                .await
            {
                tracing::error!(workspace_id = %ws_id, error = %e, "failed to mark workspace failed");
            }
            status = crate::contexts::workspace::domain::WorkspaceStatus::Failed;
            current_phase = crate::contexts::workspace::domain::PipelinePhase::Failed;
        }
    }

    let response = WorkspaceResponseDto {
        id: workspace.id.0,
        user_id: user_id.0,
        company_name: workspace.company_name,
        role_title: workspace.role_title,
        status,
        current_phase,
        progress_percent: if status == crate::contexts::workspace::domain::WorkspaceStatus::Failed { 0 } else { workspace.progress_percent },
        seniority: workspace.seniority,
        user_prompt: workspace.user_prompt,
        current_objective: workspace.current_objective,
        workflow_mode: Some(workspace.workflow_mode),
        created_at: workspace.created_at,
        updated_at: chrono::Utc::now(),
    };
    Ok((StatusCode::CREATED, Json(response)))
}

#[derive(Deserialize)]
struct StoredWorkspace {
    company_name: String,
    role_title: String,
    seniority: Option<String>,
    company_urls: Option<Vec<String>>,
    user_prompt: Option<String>,
    workflow_mode: Option<String>,
}
#[derive(Default, Deserialize)]
struct StoredResume {
    raw_text: Option<String>,
    source_upload_id: Option<serde_json::Value>,
    source_filename: Option<String>,
}

#[derive(Default, Deserialize)]
struct StoredJobDescription {
    raw_text: Option<String>,
    jd_url: Option<String>,
}

fn parse_stored<T: DeserializeOwned>(row: Option<serde_json::Value>) -> std::result::Result<Option<T>, AppError> {
    row.map(|v| serde_json::from_value(v).map_err(|e| AppError::Database(format!("Malformed stored workspace input: {e}"))))
        .transpose()
}

pub(crate) async fn start_workspace_run(
    state: &AppState,
    user_id: &UserId,
    workspace_id: &str,
    workflow_id: String,
    idempotency_key: Option<&str>,
) -> std::result::Result<WorkflowRun, AppError> {
    let mut res = state
        .db
        .client
        .query(
            "SELECT company_name, role_title, seniority, company_urls, user_prompt, workflow_mode FROM type::record('workspace', $ws_id);
             SELECT raw_text, source_upload_id, source_filename FROM type::record('resume', $res_id);
             SELECT raw_text, jd_url FROM type::record('job_description', $jd_id);",
        )
        .bind(("ws_id", workspace_id.to_string()))
        .bind(("res_id", format!("res_{workspace_id}")))
        .bind(("jd_id", format!("jd_{workspace_id}")))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let workspace: StoredWorkspace = parse_stored(res.take(0).map_err(|e| AppError::Database(e.to_string()))?)?
        .ok_or_else(|| AppError::NotFound(format!("Workspace {workspace_id} not found")))?;
    let resume: StoredResume =
        parse_stored(res.take(1).map_err(|e| AppError::Database(e.to_string()))?)?.unwrap_or_default();
    let jd: StoredJobDescription =
        parse_stored(res.take(2).map_err(|e| AppError::Database(e.to_string()))?)?.unwrap_or_default();

    let resume_upload_id = resume.source_upload_id.and_then(|val| match val {
        serde_json::Value::String(s) => Some(s.replace("resume_upload:", "").replace('`', "")),
        serde_json::Value::Object(map) => map
            .get("id")
            .and_then(|v| v.as_str())
            .map(|s| s.replace("resume_upload:", "").replace('`', "")),
        _ => None,
    });

    let payload = CreateRunPayload {
        workspace_id: workspace_id.to_string(),
        user_id: user_id.0.clone(),
        workflow_id,
        company_name: workspace.company_name,
        role_title: workspace.role_title,
        company_urls: workspace.company_urls.unwrap_or_default(),
        resume_text: resume.raw_text.unwrap_or_default(),
        resume_upload_id,
        resume_filename: resume.source_filename,
        job_description: jd.raw_text.unwrap_or_default(),
        jd_url: jd.jd_url,
        seniority: workspace.seniority,
        user_prompt: workspace.user_prompt,
        workflow_mode: workspace.workflow_mode,
        llm_config: resolve_llm_config(state, user_id).await?,
    };
    state.grpc_client.create_run(&payload, idempotency_key).await
}
pub(crate) async fn resolve_llm_config(
    state: &AppState,
    user_id: &crate::contexts::identity::domain::UserId,
) -> std::result::Result<Option<serde_json::Value>, AppError> {
    let db = &state.db.client;
    let crypto = crate::shared::encryption::CryptoService::new(&state.config.encryption_key);

    let mut user_prov_res = db
        .query("SELECT * FROM user_llm_provider WHERE user_id = type::record('user', $uid);")
        .bind(("uid", user_id.0.clone()))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let user_providers: Vec<serde_json::Value> = user_prov_res.take(0).unwrap_or_default();

    let mut sys_prov_res = db
        .query("SELECT * FROM system_llm_provider WHERE is_enabled = true;")
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let system_providers: Vec<serde_json::Value> = sys_prov_res.take(0).unwrap_or_default();

    let mut routing_res = db
        .query("SELECT slots FROM type::record('user_model_routing', $uid);")
        .bind(("uid", user_id.0.clone()))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let routing_rows: Vec<serde_json::Value> = routing_res.take(0).unwrap_or_default();
    let user_slots = routing_rows.into_iter().next().and_then(|r| r.get("slots").cloned()).unwrap_or(serde_json::json!({}));

    let mut legacy_res = db
        .query("SELECT * FROM user_llm_config WHERE user_id = type::record('user', $uid);")
        .bind(("uid", user_id.0.clone()))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let mut configs: Vec<serde_json::Value> = legacy_res.take(0).unwrap_or_default();
    if configs.is_empty() {
        let mut shared = db
            .query("SELECT * FROM user_llm_config WHERE is_shared = true;")
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        configs = shared.take(0).unwrap_or_default();
    }

    let legacy_active = configs.into_iter().next().map(|cfg| {
        let api_key = cfg["encrypted_api_key"]
            .as_str()
            .filter(|k| !k.is_empty())
            .map(|k| crypto.decrypt(k).unwrap_or_default())
            .unwrap_or_default();
        serde_json::json!({
            "provider": cfg["provider"],
            "api_key": api_key,
            "base_url": cfg["base_url"],
            "model_chat": cfg["model_chat"],
            "model_search": cfg["model_search"],
            "model_embed": cfg["model_embed"],
        })
    });

    let mut decrypted_user_providers = serde_json::Map::new();
    for up in user_providers {
        if let Some(pkey) = up.get("provider_key").and_then(|v| v.as_str()) {
            let key = up.get("encrypted_api_key")
                .and_then(|v| v.as_str())
                .map(|k| crypto.decrypt(k).unwrap_or_default())
                .unwrap_or_default();
            decrypted_user_providers.insert(pkey.to_string(), serde_json::json!({
                "provider_key": pkey,
                "api_key": key,
                "base_url": up.get("base_url").cloned()
            }));
        }
    }

    let mut decrypted_system_providers = serde_json::Map::new();
    for sp in system_providers {
        if let Some(pkey) = sp.get("provider_key").and_then(|v| v.as_str()) {
            let key = sp.get("encrypted_api_key")
                .and_then(|v| v.as_str())
                .map(|k| crypto.decrypt(k).unwrap_or_default())
                .unwrap_or_default();
            decrypted_system_providers.insert(pkey.to_string(), serde_json::json!({
                "provider_key": pkey,
                "name": sp.get("name").cloned(),
                "api_key": key,
                "base_url": sp.get("base_url").cloned(),
                "allowed_models": sp.get("allowed_models").cloned()
            }));
        }
    }

    Ok(Some(serde_json::json!({
        "user_providers": decrypted_user_providers,
        "system_providers": decrypted_system_providers,
        "model_routing": user_slots,
        "legacy": legacy_active,
        "provider": legacy_active.as_ref().and_then(|l| l.get("provider")).cloned().unwrap_or(serde_json::Value::Null),
        "api_key": legacy_active.as_ref().and_then(|l| l.get("api_key")).cloned().unwrap_or(serde_json::Value::Null),
        "base_url": legacy_active.as_ref().and_then(|l| l.get("base_url")).cloned().unwrap_or(serde_json::Value::Null),
        "model_chat": legacy_active.as_ref().and_then(|l| l.get("model_chat")).cloned().unwrap_or(serde_json::Value::Null),
        "model_search": legacy_active.as_ref().and_then(|l| l.get("model_search")).cloned().unwrap_or(serde_json::Value::Null),
        "model_embed": legacy_active.as_ref().and_then(|l| l.get("model_embed")).cloned().unwrap_or(serde_json::Value::Null),
    })))
}

#[utoipa::path(
    get,
    path = "/api/v1/workspaces",
    responses(
        (status = 200, description = "List workspaces", body = [WorkspaceResponseDto])
    )
)]
pub async fn list_workspaces(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
) -> std::result::Result<Json<Vec<WorkspaceResponseDto>>, AppError> {
    let workspaces = state
        .workspace_service
        .list_workspaces(&auth_user.user_id)
        .await?;

    let dtos: Vec<WorkspaceResponseDto> = workspaces
        .into_iter()
        .map(|w| WorkspaceResponseDto {
            id: w.id.0,
            user_id: w.user_id.0,
            company_name: w.company_name,
            role_title: w.role_title,
            status: w.status,
            current_phase: w.current_phase,
            progress_percent: w.progress_percent,
            seniority: w.seniority,
            user_prompt: w.user_prompt,
            current_objective: w.current_objective,
            workflow_mode: Some(w.workflow_mode),
            created_at: w.created_at,
            updated_at: w.updated_at,
        })
        .collect();
    Ok(Json(dtos))
}

#[utoipa::path(
    get,
    path = "/api/v1/workspaces/{id}",
    responses(
        (status = 200, description = "Workspace details", body = WorkspaceResponseDto),
        (status = 404, description = "Workspace not found")
    )
)]
pub async fn get_workspace(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(id): Path<String>,
) -> std::result::Result<Json<WorkspaceResponseDto>, AppError> {
    let clean_id = id.replace("workspace:", "").replace('`', "");
    let w = state
        .workspace_service
        .get_workspace(&WorkspaceId(clean_id.clone()), &auth_user.user_id)
        .await?;

    Ok(Json(WorkspaceResponseDto {
        id: w.id.0,
        user_id: w.user_id.0,
        company_name: w.company_name,
        role_title: w.role_title,
        status: w.status,
        current_phase: w.current_phase,
        progress_percent: w.progress_percent,
        seniority: w.seniority,
        user_prompt: w.user_prompt,
        current_objective: w.current_objective,
        workflow_mode: Some(w.workflow_mode),
        created_at: w.created_at,
        updated_at: w.updated_at,
    }))
}
#[utoipa::path(
    patch,
    path = "/api/v1/workspaces/{id}/objective",
    request_body = PatchWorkspaceObjectiveDto,
    responses(
        (status = 200, description = "Workspace objective updated and resume-plan triggered", body = serde_json::Value),
        (status = 404, description = "Workspace not found")
    )
)]
pub async fn update_workspace_objective(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(id): Path<String>,
    Json(payload): Json<crate::contexts::workspace::application::dtos::PatchWorkspaceObjectiveDto>,
) -> std::result::Result<Json<serde_json::Value>, AppError> {
    let clean_id = id.replace("workspace:", "").replace('`', "");
    let _ = state
        .workspace_service
        .get_workspace(&WorkspaceId(clean_id.clone()), &auth_user.user_id)
        .await?;

    let db = &state.db.client;
    db.query("UPDATE type::record('workspace', $ws_id) SET current_objective = $prompt, updated_at = time::now();")
        .bind(("ws_id", clean_id.clone()))
        .bind(("prompt", payload.prompt.clone()))
        .await
        .and_then(|res| res.check())
        .map_err(|e| AppError::Database(format!("Failed to update objective: {e}")))?;

    let mut run_res = db
        .query("SELECT id FROM workflow_run WHERE workspace_id = type::record('workspace', $ws_id) ORDER BY created_at DESC LIMIT 1;")
        .bind(("ws_id", clean_id.clone()))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let latest_run_rows: Vec<serde_json::Value> = run_res.take(0).unwrap_or_default();
    let latest_run = latest_run_rows.into_iter().next();

    if let Some(run_val) = latest_run {
        let run_id = run_val["id"]
            .as_str()
            .unwrap_or_default()
            .replace("workflow_run:", "")
            .replace('`', "");

        let resume_payload = ResumePlanPayload {
            prompt: payload.prompt.clone(),
        };

        let _ = state
            .grpc_client
            .resume_plan(&run_id, &resume_payload)
            .await
            .map_err(|e| {
                tracing::warn!(run_id = %run_id, error = %e, "resume_plan call error");
                e
            });
    }

    Ok(Json(serde_json::json!({
        "workspace_id": clean_id,
        "current_objective": payload.prompt,
        "status": "updated"
    })))
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg),
            AppError::Database(detail) => {
                tracing::error!(error = %detail, "Database error while handling request");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "An internal server error occurred".to_owned(),
                )
            },
            AppError::ExternalService(msg) => (StatusCode::BAD_GATEWAY, msg),
            AppError::ServiceUnavailable(msg) => (StatusCode::SERVICE_UNAVAILABLE, msg),
            AppError::Internal(detail) => {
                tracing::error!(error = %detail, "Internal error while handling request");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "An internal server error occurred".to_owned(),
                )
            },
        };

        let body = Json(serde_json::json!({
            "error": {
                "message": error_message,
                "status_code": status.as_u16(),
            }
        }));

        (status, body).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn database_and_internal_errors_return_generic_server_error() {
        let sentinel = "sensitive-error-detail-7d91";
        let cases = [
            AppError::Database(sentinel.to_owned()),
            AppError::Internal(sentinel.to_owned()),
        ];

        for error in cases {
            let response = error.into_response();
            assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

            let body = axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .expect("response body should be readable");
            let body_text = String::from_utf8(body.to_vec()).expect("response body should be UTF-8");
            assert!(!body_text.contains(sentinel));
            let body_json: serde_json::Value =
                serde_json::from_str(&body_text).expect("response body should be JSON");
            assert_eq!(
                body_json["error"]["message"],
                "An internal server error occurred"
            );
        }
    }
}
