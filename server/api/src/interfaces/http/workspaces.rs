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
    let workspace = state
        .workspace_service
        .create_workspace(user_id.clone(), payload.clone())
        .await?;

    let db = &state.db.client;
    let ws_id = workspace.id.0.clone();
    db.query("UPDATE type::record('workspace', $ws_id) SET company_urls = $company_urls;")
        .bind(("ws_id", ws_id.clone()))
        .bind(("company_urls", payload.company_urls.clone()))
        .await
        .and_then(|res| res.check())
        .map_err(|e| AppError::Database(format!("Failed to save company URLs: {e}")))?;

    if payload.resume_text.is_some() || payload.resume_pdf_base64.is_some() {
        let res_id = format!("res_{ws_id}");
        let query = "
            UPSERT type::record('resume', $id) SET
                workspace_id = type::record('workspace', $ws_id),
                raw_text = $raw_text,
                source_pdf_base64 = $source_pdf_base64,
                source_filename = $source_filename,
                parsed_skills = [],
                experience = [],
                education = [];
        ";
        db.query(query)
            .bind(("id", res_id))
            .bind(("ws_id", ws_id.clone()))
            .bind(("raw_text", payload.resume_text.clone().unwrap_or_default()))
            .bind(("source_pdf_base64", payload.resume_pdf_base64.clone()))
            .bind(("source_filename", payload.resume_filename.clone()))
            .await
            .and_then(|res| res.check())
            .map_err(|e| AppError::Database(format!("Failed to save resume: {e}")))?;
    }

    if payload.job_description.is_some() || payload.jd_url.is_some() {
        let jd_id = format!("jd_{ws_id}");
        let query = "
            UPSERT type::record('job_description', $id) SET
                workspace_id = type::record('workspace', $ws_id),
                raw_text = $raw_text,
                jd_url = $jd_url,
                requirements = [];
        ";
        let _ = db
            .query(query)
            .bind(("id", jd_id))
            .bind(("ws_id", ws_id.clone()))
            .bind(("raw_text", payload.job_description.clone().unwrap_or_default()))
            .bind(("jd_url", payload.jd_url.clone()))
            .await
            .map_err(|e| AppError::Database(format!("Failed to save JD: {e}")))?;
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
    source_pdf_base64: Option<String>,
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
             SELECT raw_text, source_pdf_base64, source_filename FROM type::record('resume', $res_id);
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

    let payload = CreateRunPayload {
        workspace_id: workspace_id.to_string(),
        user_id: user_id.0.clone(),
        workflow_id,
        company_name: workspace.company_name,
        role_title: workspace.role_title,
        company_urls: workspace.company_urls.unwrap_or_default(),
        resume_text: resume.raw_text.unwrap_or_default(),
        resume_pdf_base64: resume.source_pdf_base64,
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
            AppError::Database(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::ExternalService(msg) => (StatusCode::BAD_GATEWAY, msg),
            AppError::ServiceUnavailable(msg) => (StatusCode::SERVICE_UNAVAILABLE, msg),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
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
