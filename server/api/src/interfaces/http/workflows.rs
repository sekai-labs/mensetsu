use std::collections::HashMap;
use std::convert::Infallible;
use std::time::Duration;

use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::sse::{Event, KeepAlive, Sse},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use tokio_stream::Stream;
use utoipa::{IntoParams, ToSchema};

use crate::{
    contexts::{
        workflow::domain::{RunOwnership, WorkflowRunId},
        workspace::domain::WorkspaceId,
    },
    interfaces::http::{
        extractors::authenticated_user::AuthenticatedUser,
        workflow_dtos::{
            bare_run_id, AgentExecution, Artifact, Delegation, DelegationGraphResponse,
            LogsPageResponse, RunsPageResponse, TaskGraphResponse, WorkflowDefinition,
            WorkflowRun, WorkflowTask, WorkflowType,
        },
        workspaces::{auth_check::verify_workspace_owner, start_workspace_run, AppState},
    },
    shared::error::AppError,
};

const DEFAULT_PAGE_SIZE: usize = 50;
const MAX_PAGE_SIZE: usize = 100;

#[derive(Debug, Clone, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct TriggerWorkflowRunRequestDto {
    pub workflow_id: String,
    pub workspace_id: String,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ListRunsQuery {
    pub workspace_id: Option<String>,
    pub status: Option<String>,
    #[serde(rename = "type")]
    #[param(rename = "type")]
    pub run_type: Option<WorkflowType>,
    pub limit: Option<usize>,
    pub cursor: Option<String>,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct EventStreamQuery {
    pub after_seq: Option<u64>,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct LogsQuery {
    pub cursor: Option<String>,
    pub limit: Option<u32>,
    pub severity: Option<String>,
    pub task_id: Option<String>,
    pub execution_id: Option<String>,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct EventsHistoryQuery {
    pub after_seq: Option<i64>,
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct AgentToolsQuery {
    pub cursor: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ToolOutputQuery {
    pub offset: Option<i64>,
    pub limit: Option<i64>,
}
pub fn workflow_routes(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/workflows", get(list_workflows))
        .route("/api/v1/workflows/runs", get(list_workflow_runs).post(trigger_workflow_run))
        .route("/api/v1/workflows/runs/{run_id}", get(get_workflow_run))
        .route("/api/v1/workflows/runs/{run_id}/graph", get(get_workflow_run_graph))
        .route("/api/v1/workflows/runs/{run_id}/delegations/graph", get(get_workflow_run_delegation_graph))
        .route("/api/v1/workflows/runs/{run_id}/events", get(workflow_run_events))
        .route("/api/v1/workflows/runs/{run_id}/events/history", get(get_workflow_run_events_history))
        .route("/api/v1/workflows/runs/{run_id}/logs", get(get_workflow_run_logs))
        .route("/api/v1/workflows/runs/{run_id}/tasks/{task_id}", get(get_workflow_run_task))
        .route("/api/v1/workflows/runs/{run_id}/agents", get(list_workflow_run_agents))
        .route("/api/v1/workflows/runs/{run_id}/agents/{execution_id}", get(get_workflow_run_agent))
        .route("/api/v1/workflows/runs/{run_id}/agents/{execution_id}/tools", get(list_workflow_run_agent_tools))
        .route("/api/v1/workflows/runs/{run_id}/tools/{tool_invocation_id}/output", get(get_workflow_run_tool_output))
        .route("/api/v1/workflows/runs/{run_id}/delegations", get(get_workflow_run_delegations))
        .route("/api/v1/workflows/runs/{run_id}/artifacts", get(get_workflow_run_artifacts))
        .route("/api/v1/workflows/runs/{run_id}/cancel", post(cancel_workflow_run))
        .with_state(state)
}

async fn authorize(state: &AppState, user: &AuthenticatedUser, run_id: &str) -> Result<RunOwnership, AppError> {
    let run_id = WorkflowRunId::new(run_id).map_err(|_| AppError::NotFound("Workflow run not found".into()))?;
    state.workflow_service.authorize_run(&run_id, &user.user_id).await
}

fn ensure_same_run(expected: &RunOwnership, actual_run_id: &str) -> Result<(), AppError> {
    if bare_run_id(actual_run_id) == expected.run_id.0 {
        Ok(())
    } else {
        tracing::error!(expected = %expected.run_id, actual = %actual_run_id, "intelligence returned an object for a different run");
        Err(AppError::ExternalService("Intelligence response referenced a different run".into()))
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/workflows",
    tag = "workflows",
    responses(
        (status = 200, description = "Workflow definitions", body = Vec<WorkflowDefinition>),
        (status = 401, description = "Unauthenticated"),
        (status = 502, description = "Intelligence response did not match contract")
    )
)]
pub async fn list_workflows(
    State(state): State<AppState>,
    _auth_user: AuthenticatedUser,
) -> Result<Json<Vec<WorkflowDefinition>>, AppError> {
    Ok(Json(state.grpc_client.list_workflows().await?))
}

#[utoipa::path(
    get,
    path = "/api/v1/workflows/runs",
    tag = "workflows",
    params(ListRunsQuery),
    responses(
        (status = 200, description = "Page of the caller's workflow runs", body = RunsPageResponse),
        (status = 400, description = "Invalid cursor or limit"),
        (status = 401, description = "Unauthenticated"),
        (status = 403, description = "workspace_id not owned by caller"),
        (status = 404, description = "workspace_id not found"),
        (status = 502, description = "Intelligence response did not match contract")
    )
)]
pub async fn list_workflow_runs(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Query(query): Query<ListRunsQuery>,
) -> Result<Json<RunsPageResponse>, AppError> {
    let limit = query.limit.unwrap_or(DEFAULT_PAGE_SIZE);
    if !(1..=MAX_PAGE_SIZE).contains(&limit) {
        return Err(AppError::Validation(format!("limit must be between 1 and {MAX_PAGE_SIZE}")));
    }
    let workspace_id = query.workspace_id.as_deref().map(|id| WorkspaceId(id.to_string()));
    let run_type = query.run_type.map(|t| match t {
        WorkflowType::Static => "static",
        WorkflowType::Dynamic => "dynamic",
        WorkflowType::Hybrid => "hybrid",
    });

    let (run_ids, next_cursor) = state
        .workflow_service
        .list_user_run_ids(
            &auth_user.user_id,
            workspace_id.as_ref(),
            query.status.as_deref(),
            run_type,
            limit,
            query.cursor.as_deref(),
        )
        .await?;

    if run_ids.is_empty() {
        return Ok(Json(RunsPageResponse { items: Vec::new(), next_cursor: None }));
    }

    let ids_vec: Vec<String> = run_ids.iter().map(|r| r.0.clone()).collect();
    let page: RunsPageResponse = state
        .grpc_client
        .list_runs(ids_vec, None, None, None, run_ids.len() as u32, None)
        .await?;

    let mut by_id: HashMap<String, WorkflowRun> = page
        .items
        .into_iter()
        .map(|mut r| {
            r.normalize_record_ids();
            (r.bare_id().to_string(), r)
        })
        .collect();
    let items = run_ids.iter().filter_map(|id| by_id.remove(&id.0)).collect();
    if !by_id.is_empty() {
        tracing::warn!(extra = ?by_id.keys().collect::<Vec<_>>(), "intelligence returned runs outside the requested run_ids; dropped");
    }

    Ok(Json(RunsPageResponse { items, next_cursor }))
}

#[utoipa::path(
    post,
    path = "/api/v1/workflows/runs",
    tag = "workflows",
    request_body = TriggerWorkflowRunRequestDto,
    params(("Idempotency-Key" = Option<String>, Header, description = "Forwarded to intelligence to dedupe retries")),
    responses(
        (status = 201, description = "Workflow run created", body = WorkflowRun),
        (status = 401, description = "Unauthenticated"),
        (status = 403, description = "Workspace not owned by caller"),
        (status = 404, description = "Workspace not found"),
        (status = 400, description = "Unknown workflow_id"),
        (status = 422, description = "Invalid body (including unknown fields such as user_id)"),
        (status = 502, description = "Intelligence error or contract mismatch"),
        (status = 503, description = "Intelligence unavailable")
    )
)]
pub async fn trigger_workflow_run(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    headers: HeaderMap,
    Json(payload): Json<TriggerWorkflowRunRequestDto>,
) -> Result<(StatusCode, Json<WorkflowRun>), AppError> {
    let workspace_id = payload.workspace_id.replace("workspace:", "").replace('`', "");
    verify_workspace_owner(&state.db.client, &workspace_id, &auth_user.user_id.0).await?;

    let idempotency_key = headers.get("idempotency-key").and_then(|v| v.to_str().ok());
    let mut run =
        start_workspace_run(&state, &auth_user.user_id, &workspace_id, payload.workflow_id, idempotency_key).await?;
    run.normalize_record_ids();
    Ok((StatusCode::CREATED, Json(run)))
}

#[utoipa::path(
    get,
    path = "/api/v1/workflows/runs/{run_id}",
    tag = "workflows",
    params(("run_id" = String, Path)),
    responses(
        (status = 200, description = "Workflow run snapshot", body = WorkflowRun),
        (status = 404, description = "Unknown run or not owned by caller"),
        (status = 502, description = "Intelligence response did not match contract")
    )
)]
pub async fn get_workflow_run(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(run_id): Path<String>,
) -> Result<Json<WorkflowRun>, AppError> {
    let owned = authorize(&state, &auth_user, &run_id).await?;
    let mut run: WorkflowRun = state.grpc_client.get_run(&owned.run_id.0).await?;
    ensure_same_run(&owned, &run.id)?;
    run.normalize_record_ids();
    Ok(Json(run))
}

#[utoipa::path(
    get,
    path = "/api/v1/workflows/runs/{run_id}/graph",
    tag = "workflows",
    params(("run_id" = String, Path)),
    responses(
        (status = 200, description = "Task dependency graph", body = TaskGraphResponse),
        (status = 404, description = "Unknown run or not owned by caller"),
        (status = 502, description = "Intelligence response did not match contract")
    )
)]
pub async fn get_workflow_run_graph(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(run_id): Path<String>,
) -> Result<Json<TaskGraphResponse>, AppError> {
    let owned = authorize(&state, &auth_user, &run_id).await?;
    Ok(Json(state.grpc_client.get_task_graph(&owned.run_id.0).await?))
}

#[utoipa::path(
    get,
    path = "/api/v1/workflows/runs/{run_id}/delegations/graph",
    tag = "workflows",
    params(("run_id" = String, Path)),
    responses(
        (status = 200, description = "Agent delegation graph", body = DelegationGraphResponse),
        (status = 404, description = "Unknown run or not owned by caller"),
        (status = 502, description = "Intelligence response did not match contract")
    )
)]
pub async fn get_workflow_run_delegation_graph(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(run_id): Path<String>,
) -> Result<Json<DelegationGraphResponse>, AppError> {
    let owned = authorize(&state, &auth_user, &run_id).await?;
    Ok(Json(
        state
            .grpc_client
            .get_delegation_graph(&owned.run_id.0)
            .await?,
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/workflows/runs/{run_id}/events",
    tag = "workflows",
    params(
        ("run_id" = String, Path),
        EventStreamQuery,
        ("Last-Event-ID" = Option<String>, Header, description = "Resume cursor, used only when after_seq is absent")
    ),
    responses(
        (status = 200, description = "text/event-stream of ExecutionEvent frames (id: seq, event: type, data: ExecutionEvent JSON)", content_type = "text/event-stream"),
        (status = 404, description = "Unknown run or not owned by caller")
    )
)]
pub async fn workflow_run_events(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    headers: HeaderMap,
    Path(run_id): Path<String>,
    Query(query): Query<EventStreamQuery>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, AppError> {
    let owned = authorize(&state, &auth_user, &run_id).await?;

    let after_seq = query.after_seq.unwrap_or_else(|| {
        headers
            .get("last-event-id")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.trim().parse().ok())
            .unwrap_or(0)
    });

    let stream = state.grpc_client.stream_run_events(owned.run_id.0.clone(), after_seq).await?;
    Ok(Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(15))))
}

#[utoipa::path(
    get,
    path = "/api/v1/workflows/runs/{run_id}/logs",
    tag = "workflows",
    params(("run_id" = String, Path), LogsQuery),
    responses(
        (status = 200, description = "Page of execution logs", body = LogsPageResponse),
        (status = 404, description = "Unknown run or not owned by caller"),
        (status = 502, description = "Intelligence response did not match contract")
    )
)]
pub async fn get_workflow_run_logs(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(run_id): Path<String>,
    Query(query): Query<LogsQuery>,
) -> Result<Json<LogsPageResponse>, AppError> {
    let owned = authorize(&state, &auth_user, &run_id).await?;
    let page: LogsPageResponse = state
        .grpc_client
        .get_logs(
            &owned.run_id.0,
            query.cursor,
            query.limit.unwrap_or(100),
            query.severity,
            query.task_id,
            query.execution_id,
        )
        .await?;
    for entry in &page.items {
        ensure_same_run(&owned, &entry.run_id)?;
    }
    Ok(Json(page))
}

#[utoipa::path(
    get,
    path = "/api/v1/workflows/runs/{run_id}/tasks/{task_id}",
    tag = "workflows",
    params(("run_id" = String, Path), ("task_id" = String, Path)),
    responses(
        (status = 200, description = "Task details", body = WorkflowTask),
        (status = 404, description = "Unknown run/task or run not owned by caller"),
        (status = 502, description = "Intelligence response did not match contract")
    )
)]
pub async fn get_workflow_run_task(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path((run_id, task_id)): Path<(String, String)>,
) -> Result<Json<WorkflowTask>, AppError> {
    let owned = authorize(&state, &auth_user, &run_id).await?;
    let task: WorkflowTask = state.grpc_client.get_task(&owned.run_id.0, &task_id).await?;
    ensure_same_run(&owned, &task.run_id)?;
    Ok(Json(task))
}

#[utoipa::path(
    get,
    path = "/api/v1/workflows/runs/{run_id}/agents/{execution_id}",
    tag = "workflows",
    params(("run_id" = String, Path), ("execution_id" = String, Path)),
    responses(
        (status = 200, description = "Agent execution details", body = AgentExecution),
        (status = 404, description = "Unknown run/execution or run not owned by caller"),
        (status = 502, description = "Intelligence response did not match contract")
    )
)]
pub async fn get_workflow_run_agent(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path((run_id, execution_id)): Path<(String, String)>,
) -> Result<Json<AgentExecution>, AppError> {
    let owned = authorize(&state, &auth_user, &run_id).await?;
    let agent: AgentExecution = state
        .grpc_client
        .get_agent(&owned.run_id.0, &execution_id)
        .await?;
    ensure_same_run(&owned, &agent.run_id)?;
    Ok(Json(agent))
}

#[utoipa::path(
    get,
    path = "/api/v1/workflows/runs/{run_id}/agents",
    tag = "workflows",
    params(("run_id" = String, Path)),
    responses(
        (status = 200, description = "Agent executions in this run", body = Vec<AgentExecution>),
        (status = 404, description = "Unknown run or not owned by caller"),
        (status = 502, description = "Intelligence response did not match contract")
    )
)]
pub async fn list_workflow_run_agents(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(run_id): Path<String>,
) -> Result<Json<Vec<AgentExecution>>, AppError> {
    let owned = authorize(&state, &auth_user, &run_id).await?;
    let items: Vec<AgentExecution> = state.grpc_client.list_agents(&owned.run_id.0).await?;
    for a in &items {
        ensure_same_run(&owned, &a.run_id)?;
    }
    Ok(Json(items))
}

#[utoipa::path(
    get,
    path = "/api/v1/workflows/runs/{run_id}/events/history",
    tag = "workflows",
    params(("run_id" = String, Path), EventsHistoryQuery),
    responses(
        (status = 200, description = "Historical execution events", body = crate::interfaces::http::workflow_dtos::EventsHistoryPageResponse),
        (status = 404, description = "Unknown run or not owned by caller"),
        (status = 502, description = "Intelligence response did not match contract")
    )
)]
pub async fn get_workflow_run_events_history(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(run_id): Path<String>,
    Query(query): Query<EventsHistoryQuery>,
) -> Result<Json<crate::interfaces::http::workflow_dtos::EventsHistoryPageResponse>, AppError> {
    let owned = authorize(&state, &auth_user, &run_id).await?;
    let page: crate::interfaces::http::workflow_dtos::EventsHistoryPageResponse = state
        .grpc_client
        .get_events_history(
            &owned.run_id.0,
            query.after_seq.unwrap_or(0),
            query.limit.unwrap_or(50),
        )
        .await?;
    for e in &page.events {
        ensure_same_run(&owned, &e.run_id)?;
    }
    Ok(Json(page))
}

#[utoipa::path(
    get,
    path = "/api/v1/workflows/runs/{run_id}/agents/{execution_id}/tools",
    tag = "workflows",
    params(("run_id" = String, Path), ("execution_id" = String, Path), AgentToolsQuery),
    responses(
        (status = 200, description = "Tool invocations for this agent execution", body = crate::interfaces::http::workflow_dtos::ToolInvocationsPageResponse),
        (status = 404, description = "Unknown run/execution or not owned by caller"),
        (status = 502, description = "Intelligence response did not match contract")
    )
)]
pub async fn list_workflow_run_agent_tools(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path((run_id, execution_id)): Path<(String, String)>,
    Query(query): Query<AgentToolsQuery>,
) -> Result<Json<crate::interfaces::http::workflow_dtos::ToolInvocationsPageResponse>, AppError> {
    let owned = authorize(&state, &auth_user, &run_id).await?;
    let agent: AgentExecution = state
        .grpc_client
        .get_agent(&owned.run_id.0, &execution_id)
        .await?;
    ensure_same_run(&owned, &agent.run_id)?;

    let page: crate::interfaces::http::workflow_dtos::ToolInvocationsPageResponse = state
        .grpc_client
        .list_agent_tools(
            &owned.run_id.0,
            &execution_id,
            query.cursor,
            query.limit.unwrap_or(50),
        )
        .await?;
    for item in &page.items {
        if item.execution_id != execution_id {
            tracing::error!(expected = %execution_id, actual = %item.execution_id, "tool invocation belongs to a different execution");
            return Err(AppError::ExternalService("Intelligence response referenced a different execution".into()));
        }
    }
    Ok(Json(page))
}

#[utoipa::path(
    get,
    path = "/api/v1/workflows/runs/{run_id}/tools/{tool_invocation_id}/output",
    tag = "workflows",
    params(("run_id" = String, Path), ("tool_invocation_id" = String, Path), ToolOutputQuery),
    responses(
        (status = 200, description = "Chunked tool invocation output", body = crate::interfaces::http::workflow_dtos::ToolOutputResponse),
        (status = 404, description = "Unknown run/tool or not owned by caller"),
        (status = 502, description = "Intelligence response did not match contract")
    )
)]
pub async fn get_workflow_run_tool_output(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path((run_id, tool_invocation_id)): Path<(String, String)>,
    Query(query): Query<ToolOutputQuery>,
) -> Result<Json<crate::interfaces::http::workflow_dtos::ToolOutputResponse>, AppError> {
    let owned = authorize(&state, &auth_user, &run_id).await?;
    let output: crate::interfaces::http::workflow_dtos::ToolOutputResponse = state
        .grpc_client
        .get_tool_output(
            &owned.run_id.0,
            &tool_invocation_id,
            query.offset.unwrap_or(0) as u32,
            query.limit.unwrap_or(1000) as u32,
        )
        .await?;
    if output.tool_invocation_id != tool_invocation_id {
        tracing::error!(expected = %tool_invocation_id, actual = %output.tool_invocation_id, "tool output response referenced a different tool");
        return Err(AppError::ExternalService("Intelligence response referenced a different tool".into()));
    }
    Ok(Json(output))
}

#[utoipa::path(
    get,
    path = "/api/v1/workflows/runs/{run_id}/delegations",
    tag = "workflows",
    params(("run_id" = String, Path)),
    responses(
        (status = 200, description = "Delegations in this run", body = Vec<Delegation>),
        (status = 404, description = "Unknown run or not owned by caller"),
        (status = 502, description = "Intelligence response did not match contract")
    )
)]
pub async fn get_workflow_run_delegations(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(run_id): Path<String>,
) -> Result<Json<Vec<Delegation>>, AppError> {
    let owned = authorize(&state, &auth_user, &run_id).await?;
    let items: Vec<Delegation> = state.grpc_client.list_delegations(&owned.run_id.0).await?;
    for d in &items {
        ensure_same_run(&owned, &d.run_id)?;
    }
    Ok(Json(items))
}
#[utoipa::path(
    get,
    path = "/api/v1/workflows/runs/{run_id}/artifacts",
    tag = "workflows",
    params(("run_id" = String, Path)),
    responses(
        (status = 200, description = "Artifacts produced by this run", body = Vec<Artifact>),
        (status = 404, description = "Unknown run or not owned by caller"),
        (status = 502, description = "Intelligence response did not match contract")
    )
)]
pub async fn get_workflow_run_artifacts(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(run_id): Path<String>,
) -> Result<Json<Vec<Artifact>>, AppError> {
    let owned = authorize(&state, &auth_user, &run_id).await?;
    let items: Vec<Artifact> = state.grpc_client.list_artifacts(&owned.run_id.0).await?;
    for a in &items {
        ensure_same_run(&owned, &a.run_id)?;
    }
    Ok(Json(items))
}

#[utoipa::path(
    post,
    path = "/api/v1/workflows/runs/{run_id}/cancel",
    tag = "workflows",
    params(("run_id" = String, Path)),
    responses(
        (status = 200, description = "Run after cancellation (idempotent)", body = WorkflowRun),
        (status = 404, description = "Unknown run or not owned by caller; nothing is forwarded upstream"),
        (status = 502, description = "Intelligence response did not match contract")
    )
)]
pub async fn cancel_workflow_run(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(run_id): Path<String>,
) -> Result<Json<WorkflowRun>, AppError> {
    let owned = authorize(&state, &auth_user, &run_id).await?;
    let run: WorkflowRun = state.grpc_client.cancel_run(&owned.run_id.0).await?;
    ensure_same_run(&owned, &run.id)?;
    Ok(Json(run))
}
