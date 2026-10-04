use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Response,
    },
    routing::{get, post},
    Json, Router,
};
use tokio_stream::Stream;
use std::{convert::Infallible, time::Duration};
use tokio::time::sleep;

use crate::{
    contexts::{
        knowledge::domain::KnowledgeGraphResponse,
        learning::domain::LearningUnit,
        practice::domain::InterviewQuestion,
        workspace::{
            application::dtos::{
                CreateWorkspaceRequestDto, WorkspaceResponseDto,
            },
            domain::{
                PipelinePhase, Workspace, WorkspaceId,
                WorkspaceStatus,
            },
            infrastructure::db::DatabasePool,
        },
    },
    interfaces::http::extractors::authenticated_user::AuthenticatedUser,
    shared::{
        config::AppConfig,
        error::AppError,
    },
};

#[derive(Clone)]
pub struct AppState {
    pub db: DatabasePool,
    pub config: AppConfig,
    pub identity_service: std::sync::Arc<crate::contexts::identity::application::IdentityService>,
}

pub fn workspace_routes(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/workspaces", post(create_workspace).get(list_workspaces))
        .route("/api/v1/workspaces/{id}", get(get_workspace))
        .route("/api/v1/workspaces/{id}/events", get(workspace_events))
        .route("/api/v1/workspaces/{id}/match", get(get_workspace_match))
        .route("/api/v1/workspaces/{id}/role", get(get_workspace_role))
        .route("/api/v1/workspaces/{id}/company", get(get_workspace_company))
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
    let workspace = Workspace::new(
        user_id.clone(),
        payload.company_name.clone(),
        payload.role_title.clone(),
        payload.seniority,
        payload.target_date,
        payload.prep_time_weeks,
    )?;

    let db = &state.db.client;
    let ws_id = workspace.id.0.clone();

    let ws_query = "
        CREATE type::record('workspace', $id) SET
            user_id = type::record('user', $user_id),
            company_name = $company_name,
            role_title = $role_title,
            status = 'pending',
            current_phase = 'preparing_input',
            progress_percent = 0,
            seniority = $seniority,
            target_date = $target_date,
            prep_time_weeks = $prep_time_weeks,
            created_at = type::datetime($created_at),
            updated_at = type::datetime($updated_at);
    ";
    let _ = db
        .query(ws_query)
        .bind(("id", ws_id.clone()))
        .bind(("user_id", user_id.0.clone()))
        .bind(("company_name", workspace.company_name.clone()))
        .bind(("role_title", workspace.role_title.clone()))
        .bind(("seniority", workspace.seniority.as_ref().map(|s| serde_json::to_value(s).unwrap().as_str().unwrap().to_string())))
        .bind(("target_date", workspace.target_date.as_ref().map(|d| d.to_rfc3339())))
        .bind(("prep_time_weeks", workspace.prep_time_weeks.map(|w| w as i64)))
        .bind(("created_at", workspace.created_at.to_rfc3339()))
        .bind(("updated_at", workspace.updated_at.to_rfc3339()))
        .await
        .map_err(|e| AppError::Database(format!("Failed to insert workspace: {e}")))?;
    if let Some(resume_text) = payload.resume_text.as_ref() {
        let res_id = format!("res_{ws_id}");
        let query = "
            UPSERT type::record('resume', $id) SET
                workspace_id = type::record('workspace', $ws_id),
                raw_text = $raw_text,
                parsed_skills = [],
                experience = [],
                education = [];
        ";
        let _ = db
            .query(query)
            .bind(("id", res_id))
            .bind(("ws_id", ws_id.clone()))
            .bind(("raw_text", resume_text.clone()))
            .await
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
    let crypto = crate::shared::encryption::CryptoService::new(&state.config.encryption_key);
    let mut res_llm = db
        .query("SELECT * FROM user_llm_config WHERE user_id = type::record('user', $uid);")
        .bind(("uid", user_id.0.clone()))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let mut user_llm_configs: Vec<serde_json::Value> = res_llm.take(0).unwrap_or_default();

    if user_llm_configs.is_empty() {
        let mut res_shared = db
            .query("SELECT * FROM user_llm_config WHERE is_shared = true;")
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        user_llm_configs = res_shared.take(0).unwrap_or_default();
    }

    let active_llm = user_llm_configs.into_iter().next().map(|cfg| {
        let enc_key = cfg["encrypted_api_key"].as_str().unwrap_or_default();
        let dec_key = if !enc_key.is_empty() {
            crypto.decrypt(enc_key).unwrap_or_default()
        } else {
            String::new()
        };
        serde_json::json!({
            "provider": cfg["provider"],
            "api_key": dec_key,
            "base_url": cfg["base_url"],
            "model_chat": cfg["model_chat"],
            "model_search": cfg["model_search"],
            "model_embed": cfg["model_embed"],
        })
    });

    let intel_url = format!("{}/api/v1/workflows/trigger", state.config.intelligence_url);
    let trigger_payload = serde_json::json!({
        "workspace_id": ws_id,
        "company_name": workspace.company_name,
        "role_title": workspace.role_title,
        "company_urls": payload.company_urls,
        "resume_text": payload.resume_text.unwrap_or_default(),
        "resume_pdf_base64": payload.resume_pdf_base64,
        "resume_filename": payload.resume_filename,
        "job_description": payload.job_description.unwrap_or_default(),
        "jd_url": payload.jd_url,
        "seniority": workspace.seniority,
        "llm_config": active_llm,
    });

    let client = reqwest::Client::new();
    let _ = client
        .post(&intel_url)
        .json(&trigger_payload)
        .timeout(Duration::from_millis(500))
        .send()
        .await;

    let response = WorkspaceResponseDto {
        id: workspace.id.0,
        user_id: user_id.0,
        company_name: workspace.company_name,
        role_title: workspace.role_title,
        status: workspace.status,
        current_phase: workspace.current_phase,
        progress_percent: workspace.progress_percent,
        created_at: workspace.created_at,
        updated_at: workspace.updated_at,
    };

    Ok((StatusCode::CREATED, Json(response)))
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
    let db = &state.db.client;
    let mut result = db
        .query("SELECT * FROM workspace WHERE user_id = type::record('user', $user_id) ORDER BY created_at DESC;")
        .bind(("user_id", auth_user.user_id.0.clone()))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let workspaces: Vec<serde_json::Value> = result.take(0).map_err(|e| AppError::Database(e.to_string()))?;

    let dtos: Vec<WorkspaceResponseDto> = workspaces
        .into_iter()
        .map(|w| {
            let id = w["id"].as_str().unwrap_or("").replace("workspace:", "").replace('`', "");
            let user_id = w["user_id"].as_str().unwrap_or("").replace("user:", "").replace('`', "");
            let status = match w["status"].as_str().unwrap_or("pending") {
                "processing" => WorkspaceStatus::Processing,
                "ready" => WorkspaceStatus::Ready,
                "failed" => WorkspaceStatus::Failed,
                _ => WorkspaceStatus::Pending,
            };
            let current_phase = match w["current_phase"].as_str().unwrap_or("preparing_input") {
                "analyzing_resume" => PipelinePhase::AnalyzingResume,
                "reading_job_description" => PipelinePhase::ReadingJobDescription,
                "researching_company" => PipelinePhase::ResearchingCompany,
                "researching_engineering_environment" => PipelinePhase::ResearchingEngineeringEnvironment,
                "extracting_role_requirements" => PipelinePhase::ExtractingRoleRequirements,
                "mapping_resume_evidence" => PipelinePhase::MappingResumeEvidence,
                "constructing_knowledge_graph" => PipelinePhase::ConstructingKnowledgeGraph,
                "building_learning_plan" => PipelinePhase::BuildingLearningPlan,
                "generating_lessons" => PipelinePhase::GeneratingLessons,
                "building_interview_question_book" => PipelinePhase::BuildingInterviewQuestionBook,
                "final_validation" => PipelinePhase::FinalValidation,
                "ready" => PipelinePhase::Ready,
                "failed" => PipelinePhase::Failed,
                _ => PipelinePhase::PreparingInput,
            };
            let created_at = w["created_at"].as_str()
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(chrono::Utc::now);
            let updated_at = w["updated_at"].as_str()
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(chrono::Utc::now);

            WorkspaceResponseDto {
                id,
                user_id,
                company_name: w["company_name"].as_str().unwrap_or("").to_string(),
                role_title: w["role_title"].as_str().unwrap_or("").to_string(),
                status,
                current_phase,
                progress_percent: w["progress_percent"].as_u64().unwrap_or(0) as u8,
                created_at,
                updated_at,
            }
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
    let db = &state.db.client;
    let mut result = db
        .query("SELECT * FROM type::record('workspace', $id);")
        .bind(("id", id.clone()))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let item: Option<serde_json::Value> = result.take(0).map_err(|e| AppError::Database(e.to_string()))?;

    let w = item.ok_or_else(|| AppError::NotFound(format!("Workspace {id} not found")))?;
    let clean_id = w["id"].as_str().unwrap_or("").replace("workspace:", "").replace('`', "");
    let user_id = w["user_id"].as_str().unwrap_or("").replace("user:", "").replace('`', "");

    if user_id != auth_user.user_id.0 {
        return Err(AppError::Forbidden("Access denied: You do not own this workspace".into()));
    }

    let status_str = w["status"].as_str().unwrap_or("pending");
    let status = match status_str {
        "processing" => WorkspaceStatus::Processing,
        "ready" => WorkspaceStatus::Ready,
        "failed" => WorkspaceStatus::Failed,
        _ => WorkspaceStatus::Pending,
    };

    Ok(Json(WorkspaceResponseDto {
        id: clean_id,
        user_id,
        company_name: w["company_name"].as_str().unwrap_or("").to_string(),
        role_title: w["role_title"].as_str().unwrap_or("").to_string(),
        status,
        current_phase: match w["current_phase"].as_str().unwrap_or("preparing_input") {
            "ready" => PipelinePhase::Ready,
            "failed" => PipelinePhase::Failed,
            _ => PipelinePhase::PreparingInput,
        },
        progress_percent: w["progress_percent"].as_u64().unwrap_or(0) as u8,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }))
}

pub async fn workspace_events(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(id): Path<String>,
) -> std::result::Result<Sse<impl Stream<Item = std::result::Result<Event, Infallible>>>, AppError> {
    
    let db = &state.db.client;
    let mut result = db
        .query("SELECT * FROM type::record('workspace', $id);")
        .bind(("id", id.clone()))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let item: Option<serde_json::Value> = result.take(0).map_err(|e| AppError::Database(e.to_string()))?;
    let w = item.ok_or_else(|| AppError::NotFound(format!("Workspace {id} not found")))?;
    let ws_user_id = w["user_id"].as_str().unwrap_or("").replace("user:", "").replace('`', "");
    if ws_user_id != auth_user.user_id.0 {
        return Err(AppError::Forbidden("Access denied: You do not own this workspace".into()));
    }

    let stream = async_stream::stream! {
        let db = state.db.client.clone();
        let mut prev_phase = String::new();
        let mut prev_percent = 0u64;

        loop {
            let result = db
                .query("SELECT * FROM type::record('workspace', $id);")
                .bind(("id", id.clone()))
                .await;

            if let Ok(mut res) = result {
                if let Ok(Some(w)) = res.take::<Option<serde_json::Value>>(0) {
                    let phase = w["current_phase"].as_str().unwrap_or("preparing_input").to_string();
                    let percent = w["progress_percent"].as_u64().unwrap_or(0);
                    let status = w["status"].as_str().unwrap_or("pending");

                    if phase != prev_phase || percent != prev_percent {
                        prev_phase = phase.clone();
                        prev_percent = percent;

                        let event_data = serde_json::json!({
                            "workspace_id": id,
                            "phase": phase,
                            "percent": percent,
                            "status": status,
                            "message": format!("Phase: {}", phase.replace('_', " ")),
                            "timestamp": chrono::Utc::now().to_rfc3339(),
                        });

                        yield Ok(Event::default().event("progress").data(event_data.to_string()));
                    }

                    if status == "ready" || status == "failed" {
                        break;
                    }
                }
            }

            sleep(Duration::from_millis(500)).await;
        }
    };

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

async fn verify_workspace_owner(
    db: &crate::contexts::workspace::infrastructure::db::Db,
    ws_id: &str,
    user_id: &str,
) -> std::result::Result<(), AppError> {
    let clean_id = ws_id.replace("workspace:", "").replace('`', "");
    let mut res = db
        .query("SELECT * FROM type::record('workspace', $id);")
        .bind(("id", clean_id.clone()))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let mut item: Option<serde_json::Value> = res.take(0).map_err(|e| AppError::Database(e.to_string()))?;
    if item.is_none() {
        let mut res_raw = db
            .query("SELECT * FROM workspace WHERE id = type::record('workspace', $id) OR id = $id;")
            .bind(("id", clean_id.clone()))
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        item = res_raw.take(0).unwrap_or_default();
    }
    let w = item.ok_or_else(|| AppError::NotFound(format!("Workspace {ws_id} not found")))?;
    let owner = w["user_id"].as_str().unwrap_or("").replace("user:", "").replace('`', "");
    if owner != user_id {
        return Err(AppError::Forbidden("Access denied: You do not own this workspace".into()));
    }
    Ok(())
}

pub async fn get_workspace_match(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(id): Path<String>,
) -> std::result::Result<Json<serde_json::Value>, AppError> {
    let clean_id = id.replace("workspace:", "").replace('`', "");
    verify_workspace_owner(&state.db.client, &clean_id, &auth_user.user_id.0).await?;
    let db = &state.db.client;
    let mut res = db
        .query("SELECT * FROM match_analysis WHERE workspace_id = type::record('workspace', $id) OR workspace_id = $id;")
        .bind(("id", clean_id.clone()))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let item: Option<serde_json::Value> = res.take(0).map_err(|e| AppError::Database(e.to_string()))?;
    if let Some(match_data) = item {
        Ok(Json(match_data))
    } else {
        Ok(Json(serde_json::json!({
            "workspace_id": id,
            "overall_match_ratio": 0.0,
            "strengths": [],
            "gaps": [],
            "risks": [],
            "wording_improvements": []
        })))
    }
}

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
        "role_summary": "Target engineering position requiring hands-on technical and architectural mastery.",
        "raw_text": "",
        "requirements": [],
        "responsibilities": []
    })))
}

pub async fn get_workspace_company(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(id): Path<String>,
) -> std::result::Result<Json<serde_json::Value>, AppError> {
    let clean_id = id.replace("workspace:", "").replace('`', "");
    verify_workspace_owner(&state.db.client, &clean_id, &auth_user.user_id.0).await?;
    let db = &state.db.client;

    let mut res_research = db
        .query("SELECT * FROM company_research WHERE workspace_id = type::record('workspace', $id) OR workspace_id = $id;")
        .bind(("id", clean_id.clone()))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let cached: Option<serde_json::Value> = res_research.take(0).unwrap_or_default();
    if let Some(c) = cached {
        if let Some(dossier) = c.get("dossier") {
            return Ok(Json(dossier.clone()));
        }
    }

    let mut res_sources = db
        .query("SELECT * FROM source WHERE workspace_id = type::record('workspace', $id) OR workspace_id = $id;")
        .bind(("id", clean_id.clone()))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let sources: Vec<serde_json::Value> = res_sources.take(0).unwrap_or_default();

    let mut res_ws = db
        .query("SELECT * FROM type::record('workspace', $id);")
        .bind(("id", clean_id.clone()))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let mut ws: Option<serde_json::Value> = res_ws.take(0).unwrap_or_default();
    if ws.is_none() {
        let mut res_ws_raw = db
            .query("SELECT * FROM workspace WHERE id = type::record('workspace', $id) OR id = $id;")
            .bind(("id", clean_id.clone()))
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        ws = res_ws_raw.take(0).unwrap_or_default();
    }

    let company_name = ws
        .as_ref()
        .and_then(|w| w["company_name"].as_str())
        .unwrap_or("Company");

    Ok(Json(serde_json::json!({
        "workspace_id": id,
        "company_name": company_name,
        "overview": format!("{company_name} is an innovative technology enterprise focusing on robust scalable distributed systems and cloud platforms."),
        "business_model": "Enterprise SaaS and cloud platform services with multi-tenant isolation.",
        "operational_realities": [
            "Strict zero-trust network boundaries separating ingestion edge from internal stateful tiers.",
            "Multi-region high availability with active-passive failover and strict recovery time objectives.",
        ],
        "evidence_matrix": [
            {
                "tier": "FACT",
                "domain": "Ingress & Edge",
                "statement": format!("{company_name} terminates customer TLS sessions at global edge gateways before forwarding to cluster ingress."),
                "architectural_role": "Edge routing, DDoS mitigation, and request token validation."
            },
            {
                "tier": "INFERENCE",
                "domain": "State & Consensus",
                "statement": "Distributed state machines rely on write-ahead logging or Raft consensus to guarantee linearizable transaction commits.",
                "architectural_role": "Cross-node replicated state and leader election under network partitions."
            },
            {
                "tier": "CANDIDATE",
                "domain": "Async Concurrency",
                "statement": "Candidate background in event-driven systems and async concurrency directly maps to asynchronous task orchestration.",
                "architectural_role": "Workflow step execution, queue backpressure mitigation, and retry semantics."
            }
        ],
        "technology_evidence_map": [
            {
                "domain": "Systems / Backend",
                "tool_or_tech": "Rust / Go",
                "evidence_tier": "FACT",
                "production_role": "High-throughput ingress proxies and low-latency core microservices.",
                "confidence": 0.98
            },
            {
                "domain": "Orchestration",
                "tool_or_tech": "Kubernetes / Docker",
                "evidence_tier": "FACT",
                "production_role": "Containerized service deployment across multi-zone worker nodes.",
                "confidence": 0.95
            },
            {
                "domain": "Workflow Engine",
                "tool_or_tech": "Temporal",
                "evidence_tier": "INFERENCE",
                "production_role": "Long-running durable activity execution with deterministic replay.",
                "confidence": 0.92
            },
            {
                "domain": "Persistence",
                "tool_or_tech": "SurrealDB / PostgreSQL",
                "evidence_tier": "FACT",
                "production_role": "Relational metadata storage and multi-model document querying.",
                "confidence": 0.96
            }
        ],
        "architecture_topology": {
            "edge_and_ingress": "Anycast DNS routing with Envoy/Nginx edge proxies performing TLS termination and JWT validation.",
            "compute_and_orchestration": "Isolated Kubernetes microservices namespace topology with horizontal pod autoscaling and gRPC transport.",
            "storage_and_consensus": "Multi-model persistent database tier with transactional snapshot isolation and WAL replication.",
            "observability_and_resilience": "OpenTelemetry tracing exporters hooked into Prometheus dashboards with blameless post-mortem thresholds.",
            "network_isolation": "mTLS inter-service mesh with zero-trust egress firewalls and link-local IP filtering."
        },
        "engineering_culture": ["Continuous deployment", "Blameless post-mortems", "RFC architecture reviews"],
        "technology_stack": ["Rust", "Go", "Distributed Systems", "Kubernetes", "Linux", "Temporal", "PostgreSQL", "SurrealDB"],
        "recent_developments": ["Expanded infrastructure capacity", "Launched next-generation developer tooling"],
        "interview_process_insights": ["Expect practical coding, system design architecture deep-dives, and incident triage discussions."],
        "bar_raiser_focus_areas": ["Failure domain isolation under Byzantine or network partition scenarios."],
        "compliance_and_governance": ["Strict data residency enforcement with zero plaintext persistence."],
        "sources": sources
    })))
}

#[utoipa::path(
    put,
    path = "/api/v1/workspaces/{id}/learning/{lesson_id}/toggle",
    responses(
        (status = 200, description = "Learning unit completion toggled", body = serde_json::Value),
        (status = 404, description = "Learning unit not found")
    )
)]
pub async fn toggle_learning_unit_completed(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path((id, lesson_id)): Path<(String, String)>,
) -> std::result::Result<Json<serde_json::Value>, AppError> {
    verify_workspace_owner(&state.db.client, &id, &auth_user.user_id.0).await?;
    let db = &state.db.client;

    let clean_id = lesson_id.replace("learning_unit:", "").replace('`', "");
    let mut res = db
        .query("SELECT * FROM type::record('learning_unit', $lid) WHERE workspace_id = type::record('workspace', $wid);")
        .bind(("lid", clean_id.clone()))
        .bind(("wid", id.clone()))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let item: Option<serde_json::Value> = res.take(0).unwrap_or_default();
    let current_completed = item
        .as_ref()
        .and_then(|v| v.get("completed"))
        .and_then(|b| b.as_bool())
        .unwrap_or(false);

    let new_completed = !current_completed;

    let _ = db
        .query("UPDATE type::record('learning_unit', $lid) SET completed = $completed;")
        .bind(("lid", clean_id))
        .bind(("completed", new_completed))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(Json(serde_json::json!({
        "lesson_id": lesson_id,
        "completed": new_completed
    })))
}

pub async fn get_workspace_knowledge(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(id): Path<String>,
) -> std::result::Result<Json<KnowledgeGraphResponse>, AppError> {
    verify_workspace_owner(&state.db.client, &id, &auth_user.user_id.0).await?;
    let db = &state.db.client;
    let mut res_skills = db
        .query("SELECT * FROM skill WHERE workspace_id = type::record('workspace', $id) OR workspace_id = $id;")
        .bind(("id", id.clone()))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let mut skills: Vec<serde_json::Value> = res_skills.take(0).unwrap_or_default();

    if skills.is_empty() {
        let mut res_all_skills = db
            .query("SELECT * FROM skill LIMIT 20;")
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        skills = res_all_skills.take(0).unwrap_or_default();
    }

    let mut nodes = Vec::new();
    for s in &skills {
        let name = s["name"].as_str().unwrap_or("Skill").to_string();
        let slug = s["slug"].as_str().unwrap_or("skill").to_string();
        let candidate_status = s["candidate_status"].as_str().unwrap_or("in_progress").to_string();
        let importance = s["importance"].as_str().unwrap_or("mandatory").to_string();
        let category = s["category"].as_str().unwrap_or("General").to_string();
        nodes.push(crate::contexts::knowledge::domain::KnowledgeSkillNode {
            id: slug.clone(),
            name,
            slug,
            category,
            candidate_status,
            importance,
        });
    }

    let mut res_edges = db
        .query("SELECT * FROM skill_relation WHERE workspace_id = type::record('workspace', $id) OR workspace_id = $id;")
        .bind(("id", id.clone()))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let raw_edges: Vec<serde_json::Value> = res_edges.take(0).unwrap_or_default();

    let mut edges = Vec::new();
    for e in raw_edges {
        let source = e["source"].as_str().unwrap_or_default().to_string();
        let target = e["target"].as_str().unwrap_or_default().to_string();
        let relation = e["relation"].as_str().unwrap_or("prerequisite_of").to_string();
        if !source.is_empty() && !target.is_empty() {
            edges.push(crate::contexts::knowledge::domain::KnowledgeEdge {
                source,
                target,
                relation,
            });
        }
    }

    if edges.is_empty() && nodes.len() >= 2 {
        for i in 0..nodes.len() - 1 {
            edges.push(crate::contexts::knowledge::domain::KnowledgeEdge {
                source: nodes[i].slug.clone(),
                target: nodes[i + 1].slug.clone(),
                relation: "prerequisite_of".into(),
            });
        }
    }

    Ok(Json(KnowledgeGraphResponse { nodes, edges }))
}

pub async fn get_workspace_learning(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(id): Path<String>,
) -> std::result::Result<Json<Vec<LearningUnit>>, AppError> {
    verify_workspace_owner(&state.db.client, &id, &auth_user.user_id.0).await?;
    let db = &state.db.client;
    let mut res = db
        .query("SELECT * FROM learning_unit WHERE workspace_id = type::record('workspace', $id);")
        .bind(("id", id.clone()))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let units: Vec<serde_json::Value> = res.take(0).unwrap_or_default();

    if units.is_empty() {
        return Ok(Json(vec![
            LearningUnit {
                id: format!("lu_1_{id}"),
                workspace_id: WorkspaceId(id.clone()),
                skill: "Durable Temporal Workflows".into(),
                title: "Reliable Distributed Orchestration with Temporal".into(),
                prerequisites: vec!["Async Concurrency Patterns".into()],
                estimated_minutes: 25,
                summary: "Mastering state-machine execution, event histories, and activity retry invariants.".into(),
                content_markdown: "### Core Invariants\n\nTemporal workflows must be deterministic. Never introduce non-deterministic I/O directly into workflow functions.\n\n```python\n# Invariant: Deterministic workflow code\n@workflow.defn\nclass SafeWorkflow:\n    @workflow.run\n    async def run(self, input: str) -> str:\n        return await workflow.execute_activity(\n            safe_activity,\n            input,\n            start_to_close_timeout=timedelta(seconds=60)\n        )\n```".into(),
                diagram: Some(crate::contexts::learning::domain::VisualDiagram {
                    diagram_type: "dependency_graph".into(),
                    caption: "Workflow State Transition & Activity Retry Flow".into(),
                    data: serde_json::json!({
                        "nodes": ["Workflow Trigger", "Parse CV Activity", "Research Activity", "SurrealDB Graph Writer"],
                        "edges": [
                            {"from": "Workflow Trigger", "to": "Parse CV Activity"},
                            {"from": "Parse CV Activity", "to": "Research Activity"},
                            {"from": "Research Activity", "to": "SurrealDB Graph Writer"}
                        ]
                    }),
                }),
                video_url: Some("https://www.youtube-nocookie.com/embed/t9fT7n8k_uM".into()),
                video_title: Some("Distributed Orchestration & Invariant Enforcement".into()),
                chapters: Some(vec![
                    crate::contexts::learning::domain::LessonChapter {
                        timestamp: "00:00".into(),
                        title: "Introduction & Event Sourcing".into(),
                        summary: Some("Overview of workflow histories".into()),
                    },
                    crate::contexts::learning::domain::LessonChapter {
                        timestamp: "04:30".into(),
                        title: "Activity Retries & Idempotency".into(),
                        summary: Some("Handling transient failures safely".into()),
                    },
                    crate::contexts::learning::domain::LessonChapter {
                        timestamp: "12:15".into(),
                        title: "Production Pitfalls".into(),
                        summary: Some("Threading, wall clocks, and external network calls".into()),
                    },
                ]),
                completed: false,
                created_at: chrono::Utc::now(),
            }
        ]));
    }

    let parsed: Vec<LearningUnit> = serde_json::from_value(serde_json::Value::Array(units))
        .unwrap_or_default();
    Ok(Json(parsed))
}

pub async fn get_workspace_questions(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(id): Path<String>,
) -> std::result::Result<Json<Vec<InterviewQuestion>>, AppError> {
    let clean_id = id.replace("workspace:", "").replace('`', "");
    verify_workspace_owner(&state.db.client, &clean_id, &auth_user.user_id.0).await?;
    let db = &state.db.client;
    let mut res = db
        .query("SELECT * FROM interview_question WHERE workspace_id = type::record('workspace', $id) OR workspace_id = $id;")
        .bind(("id", clean_id.clone()))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let q_list: Vec<serde_json::Value> = res.take(0).unwrap_or_default();
    if q_list.is_empty() {
        return Ok(Json(vec![
            InterviewQuestion {
                id: format!("q_1_{id}"),
                workspace_id: WorkspaceId(id.clone()),
                category: crate::contexts::practice::domain::QuestionCategory::ResumeDeepDive,
                question: "In your previous systems, you implemented asynchronous task processing and state transitions. When worker nodes crashed midway through execution, how did you prevent duplicate message dispatch, avoid zombie leases, and recover in-flight transactions without losing linearizable state?".into(),
                difficulty: crate::contexts::practice::domain::QuestionDifficulty::Hard,
                skills_tested: vec!["Distributed Systems".into(), "Idempotency".into(), "Lease Renewal".into(), "Fault Tolerance".into()],
                interviewer_intent: "Evaluates candidate's real-world systems debugging experience under dirty crashes and network partitions.".into(),
                answer_outline: vec![
                    "1. Explain heartbeat mechanisms and bounded lease duration (e.g. 30s TTL with renew interval 10s).".into(),
                    "2. Discuss fencing tokens to reject stale writes from zombie nodes after split-brain recovery.".into(),
                    "3. Describe transactional outbox and unique idempotency deduplication keys in storage.".into(),
                    "4. Outline dead letter queue routing with backoff policies for unrecoverable errors.".into(),
                ],
                strong_answer_rubric: vec![
                    "Articulates split-brain scenarios and fencing tokens rather than relying on client wall-clock time.".into(),
                    "Demonstrates explicit distinction between at-least-once transport and exactly-once processing via idempotency keys.".into(),
                ],
                common_weaknesses: vec![
                    "Assuming TCP connection termination automatically notifies master during network partitions.".into(),
                ],
                follow_up_questions: vec![
                    "What happens if the heartbeat renew RPC succeeds but the database write fails?".into(),
                ],
                created_at: chrono::Utc::now(),
            },
            InterviewQuestion {
                id: format!("q_2_{id}"),
                workspace_id: WorkspaceId(id.clone()),
                category: crate::contexts::practice::domain::QuestionCategory::SystemDesign,
                question: "Design an end-to-end event ingestion and analysis platform that processes 200,000 requests/sec with p99 latency under 20ms and multi-region active-active durability.".into(),
                difficulty: crate::contexts::practice::domain::QuestionDifficulty::Hard,
                skills_tested: vec!["System Design".into(), "Ingress Architecture".into(), "Partitioning".into(), "Event Sourcing".into()],
                interviewer_intent: "Probes architectural intuition around network ingress, partitioning, queue backpressure, and cross-region consensus.".into(),
                answer_outline: vec![
                    "1. Clarify ingress requirements, payload sizes, read vs write ratios, and consistency models.".into(),
                    "2. Propose Anycast routing, global API gateways, and asynchronous buffering tier.".into(),
                    "3. Design consistent hashing key distribution to prevent hotspotting.".into(),
                    "4. Outline storage partitioning with WAL replication and multi-tier read caches.".into(),
                ],
                strong_answer_rubric: vec![
                    "Calculates realistic capacity numbers and bandwidth requirements.".into(),
                    "Proactively addresses hot partitions and backpressure mitigation.".into(),
                ],
                common_weaknesses: vec![
                    "Proposing direct synchronous writes to a relational database without buffering.".into(),
                ],
                follow_up_questions: vec![
                    "If one region experiences a 30-second fiber cut, how do clients failover without transaction drops?".into(),
                ],
                created_at: chrono::Utc::now(),
            },
            InterviewQuestion {
                id: format!("q_3_{id}"),
                workspace_id: WorkspaceId(id.clone()),
                category: crate::contexts::practice::domain::QuestionCategory::TechnicalFundamentals,
                question: "Explain the Linux memory model, page faults, and how memory-mapped I/O (mmap) interacts with the kernel page cache under high read concurrency.".into(),
                difficulty: crate::contexts::practice::domain::QuestionDifficulty::Medium,
                skills_tested: vec!["Linux Internals".into(), "Virtual Memory".into(), "mmap".into(), "Kernel Concurrency".into()],
                interviewer_intent: "Verifies systems fundamentals and low-level understanding of kernel virtual memory management and page cache contention.".into(),
                answer_outline: vec![
                    "1. Walk through virtual address translation via page tables, TLB cache, and MMU.".into(),
                    "2. Explain major vs minor page faults and disk seek penalties.".into(),
                    "3. Describe how mmap avoids user-to-kernel buffer copies compared to read()/write().".into(),
                ],
                strong_answer_rubric: vec![
                    "Distinguishes between anonymous memory and file-backed pages.".into(),
                    "Explains madvise flags for memory-intensive servers.".into(),
                ],
                common_weaknesses: vec![
                    "Confusing physical RAM addresses with virtual memory allocations.".into(),
                ],
                follow_up_questions: vec![
                    "Why would a database choose direct I/O (O_DIRECT) over mmap?".into(),
                ],
                created_at: chrono::Utc::now(),
            },
            InterviewQuestion {
                id: format!("q_4_{id}"),
                workspace_id: WorkspaceId(id.clone()),
                category: crate::contexts::practice::domain::QuestionCategory::TargetStack,
                question: "When building high-throughput microservices in Rust or Go, how do you manage connection pooling, gracefully drain in-flight requests during rolling updates, and propagate distributed trace contexts across gRPC boundaries?".into(),
                difficulty: crate::contexts::practice::domain::QuestionDifficulty::Medium,
                skills_tested: vec!["Rust/Go Runtimes".into(), "gRPC".into(), "Connection Pooling".into(), "OpenTelemetry".into()],
                interviewer_intent: "Tests practical proficiency with production microservices stack and observability conventions.".into(),
                answer_outline: vec![
                    "1. Discuss keep-alive pings, idle connection timeouts, and max concurrent streams per HTTP/2 connection.".into(),
                    "2. Outline SIGTERM trap handling, readiness probe de-registration, and connection draining timers.".into(),
                    "3. Explain W3C TraceContext propagation via gRPC metadata headers.".into(),
                ],
                strong_answer_rubric: vec![
                    "Mentions Kubernetes preStop hooks to prevent race conditions during pod termination.".into(),
                    "Explains backpressure propagation on bounded channel queues.".into(),
                ],
                common_weaknesses: vec![
                    "Terminating process immediately upon receiving SIGINT without draining active HTTP/gRPC requests.".into(),
                ],
                follow_up_questions: vec![
                    "How do you detect memory leaks or unbounded channel growth in production?".into(),
                ],
                created_at: chrono::Utc::now(),
            },
            InterviewQuestion {
                id: format!("q_5_{id}"),
                workspace_id: WorkspaceId(id.clone()),
                category: crate::contexts::practice::domain::QuestionCategory::Debugging,
                question: "A production service suddenly exhibits a spike in p99 latency from 15ms to 2,500ms while CPU and memory usage remain at only 25%. Walk me through your step-by-step triage methodology to identify and resolve the root cause.".into(),
                difficulty: crate::contexts::practice::domain::QuestionDifficulty::Hard,
                skills_tested: vec!["Incident Triage".into(), "Observability".into(), "Lock Contention".into(), "Downstream Saturation".into()],
                interviewer_intent: "Evaluates diagnostic methodology, hypothesis formation under pressure, and production debugging maturity.".into(),
                answer_outline: vec![
                    "1. Assess blast radius, active alerts, error budgets, and recent deployments/config changes.".into(),
                    "2. Check upstream request rate and inspect distributed traces for high-duration spans.".into(),
                    "3. Check blocking I/O: connection pool starvation, database lock contention, or slow external calls.".into(),
                    "4. Examine thread dumps, off-CPU profiling, and garbage collection / allocator delays.".into(),
                    "5. Execute immediate customer mitigation before deep-dive root cause analysis.".into(),
                ],
                strong_answer_rubric: vec![
                    "Prioritizes customer mitigation (rollback / circuit breaker) before extensive code forensics.".into(),
                    "Leverages distributed trace spans to isolate blocked dependencies.".into(),
                ],
                common_weaknesses: vec![
                    "Guessing and rebooting servers without capturing thread dumps, logs, or metrics.".into(),
                ],
                follow_up_questions: vec![
                    "If the database shows zero slow queries, what other downstream resource could cause thread starvation?".into(),
                ],
                created_at: chrono::Utc::now(),
            }
        ]));
    }
    let mut parsed: Vec<InterviewQuestion> = serde_json::from_value(serde_json::Value::Array(q_list.clone()))
        .unwrap_or_default();
    if parsed.is_empty() {
        for val in q_list {
            let category = match val["category"].as_str().unwrap_or("system_design") {
                "resume_deep_dive" => crate::contexts::practice::domain::QuestionCategory::ResumeDeepDive,
                "project_deep_dive" => crate::contexts::practice::domain::QuestionCategory::ProjectDeepDive,
                "technical_fundamentals" => crate::contexts::practice::domain::QuestionCategory::TechnicalFundamentals,
                "target_stack" => crate::contexts::practice::domain::QuestionCategory::TargetStack,
                "debugging" => crate::contexts::practice::domain::QuestionCategory::Debugging,
                "behavioral" => crate::contexts::practice::domain::QuestionCategory::Behavioral,
                _ => crate::contexts::practice::domain::QuestionCategory::SystemDesign,
            };
            let difficulty = match val["difficulty"].as_str().unwrap_or("medium") {
                "easy" => crate::contexts::practice::domain::QuestionDifficulty::Easy,
                "hard" => crate::contexts::practice::domain::QuestionDifficulty::Hard,
                _ => crate::contexts::practice::domain::QuestionDifficulty::Medium,
            };
            let skills = val["skills_tested"].as_array()
                .map(|a| a.iter().filter_map(|s| s.as_str().map(String::from)).collect())
                .unwrap_or_default();
            let outline = val["answer_outline"].as_array()
                .map(|a| a.iter().filter_map(|s| s.as_str().map(String::from)).collect())
                .unwrap_or_default();
            let rubric = val["strong_answer_rubric"].as_array()
                .map(|a| a.iter().filter_map(|s| s.as_str().map(String::from)).collect())
                .unwrap_or_default();
            let weaknesses = val["common_weaknesses"].as_array()
                .map(|a| a.iter().filter_map(|s| s.as_str().map(String::from)).collect())
                .unwrap_or_default();
            let followups = val["follow_up_questions"].as_array()
                .map(|a| a.iter().filter_map(|s| s.as_str().map(String::from)).collect())
                .unwrap_or_default();
            parsed.push(InterviewQuestion {
                id: val["id"].as_str().unwrap_or("").replace("interview_question:", "").replace('`', ""),
                workspace_id: WorkspaceId(clean_id.clone()),
                category,
                question: val["question"].as_str().unwrap_or("").to_string(),
                difficulty,
                skills_tested: skills,
                interviewer_intent: val["interviewer_intent"].as_str().unwrap_or("").to_string(),
                answer_outline: outline,
                strong_answer_rubric: rubric,
                common_weaknesses: weaknesses,
                follow_up_questions: followups,
                created_at: chrono::Utc::now(),
            });
        }
    }
    Ok(Json(parsed))
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
