pub mod admin;
pub mod auth;
pub mod extractors;
pub mod health;
pub mod intelligence_client;
pub mod middleware;
pub mod settings;
pub mod sse_parser;
pub mod workflow_dtos;
pub mod workflows;
pub mod workspaces;
use axum::{
    http::{header, HeaderValue, Method},
    routing::get,
    Router,
};
use tower_http::cors::CorsLayer;
use tower_http::trace::{DefaultMakeSpan, DefaultOnFailure, DefaultOnResponse, TraceLayer};
use tower_http::LatencyUnit;
use tracing::Level;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    contexts::{
        identity::{
            application::dtos::{
                AuthResponseDto, Login2faRequestDto, LoginRequestDto, LoginResponseDto,
                MessageResponseDto, RefreshTokenRequestDto, RegisterRequestDto, TokenResponseDto,
                TotpSetupResponseDto, UserResponseDto, VerifyTotpRequestDto,
            },
            domain::{SessionId, User, UserId, UserStatus},
        },
        knowledge::domain::{
            CompanyClaim, CompanyResearch, KnowledgeEdge, KnowledgeGraphResponse,
            KnowledgeSkillNode, SourceCitation,
        },
        learning::domain::{LearningUnit, VisualDiagram},
        practice::domain::{InterviewQuestion, QuestionCategory, QuestionDifficulty},
        workspace::{
            application::dtos::{
                CreateWorkspaceRequestDto, WorkspaceProgressEventDto, WorkspaceResponseDto,
            },
            domain::{
                match_analysis::{
                    GapEvidence, MatchAnalysis, StrengthEvidence, WordingImprovement,
                },
                PipelinePhase, SeniorityLevel, Workspace, WorkspaceId, WorkspaceStatus,
            },
        },
    },
    interfaces::http::workspaces::AppState,
};

#[derive(OpenApi)]
#[openapi(
    paths(
        auth::register,
        auth::login,
        auth::login_2fa,
        auth::setup_2fa,
        auth::verify_2fa,
        auth::disable_2fa,
        auth::refresh,
        auth::logout,
        auth::logout_all,
        auth::me,
        workspaces::create_workspace,
        workspaces::list_workspaces,
        workspaces::get_workspace,
        workspaces::toggle_learning_unit_completed,
        settings::get_profile,
        settings::get_llm_settings,
        settings::upsert_llm_setting,
        settings::delete_llm_setting,
        workflows::list_workflows,
        workflows::list_workflow_runs,
        workflows::trigger_workflow_run,
        workflows::get_workflow_run,
        workflows::get_workflow_run_graph,
        workflows::get_workflow_run_delegation_graph,
        workflows::workflow_run_events,
        workflows::get_workflow_run_logs,
        workflows::get_workflow_run_task,
        workflows::list_workflow_run_agents,
        workflows::get_workflow_run_agent,
        workflows::list_workflow_run_agent_tools,
        workflows::get_workflow_run_tool_output,
        workflows::get_workflow_run_events_history,
        workflows::get_workflow_run_delegations,
        workflows::get_workflow_run_artifacts,
        workflows::cancel_workflow_run
    ),
    components(
        schemas(
            UserId,
            SessionId,
            UserStatus,
            crate::contexts::identity::domain::UserRole,
            User,
            RegisterRequestDto,
            LoginRequestDto,
            LoginResponseDto,
            Login2faRequestDto,
            TotpSetupResponseDto,
            VerifyTotpRequestDto,
            RefreshTokenRequestDto,
            UserResponseDto,
            AuthResponseDto,
            TokenResponseDto,
            MessageResponseDto,
            WorkspaceId,
            WorkspaceStatus,
            PipelinePhase,
            SeniorityLevel,
            Workspace,
            CreateWorkspaceRequestDto,
            WorkspaceResponseDto,
            WorkspaceProgressEventDto,
            StrengthEvidence,
            GapEvidence,
            WordingImprovement,
            MatchAnalysis,
            SourceCitation,
            CompanyClaim,
            KnowledgeSkillNode,
            KnowledgeEdge,
            KnowledgeGraphResponse,
            CompanyResearch,
            VisualDiagram,
            LearningUnit,
            QuestionCategory,
            QuestionDifficulty,
            InterviewQuestion,
            crate::contexts::learning::domain::LessonChapter,
            settings::UserProfileResponseDto,
            settings::LlmConfigItemDto,
            settings::UpsertLlmConfigRequestDto,
            workflows::TriggerWorkflowRunRequestDto,
            workflow_dtos::WorkflowType,
            workflow_dtos::RunStatus,
            workflow_dtos::TaskStatus,
            workflow_dtos::ExecutionStatus,
            workflow_dtos::ToolStatus,
            workflow_dtos::DelegationStatus,
            workflow_dtos::LogSeverity,
            workflow_dtos::ErrorCategory,
            workflow_dtos::ArtifactType,
            workflow_dtos::WorkflowProgress,
            workflow_dtos::WorkflowDefinition,
            workflow_dtos::WorkflowRun,
            workflow_dtos::WorkflowTask,
            workflow_dtos::ExecutionNote,
            workflow_dtos::ExecutionBudgets,
            workflow_dtos::ExecutionBudgetsUsed,
            workflow_dtos::AgentExecution,
            workflow_dtos::ToolInvocation,
            workflow_dtos::Delegation,
            workflow_dtos::SourceLink,
            workflow_dtos::EvidenceProvenance,
            workflow_dtos::ArtifactContent,
            workflow_dtos::Artifact,
            workflow_dtos::LogEntry,
            workflow_dtos::ExecutionEventType,
            workflow_dtos::WorkflowRetryingPayload,
            workflow_dtos::ExecutionEvent,
            workflow_dtos::RunsPageResponse,
            workflow_dtos::LogsPageResponse,
            workflow_dtos::EventsHistoryPageResponse,
            workflow_dtos::ToolInvocationsPageResponse,
            workflow_dtos::ToolOutputResponse,
            workflow_dtos::TaskGraphNodeData,
            workflow_dtos::TaskGraphNode,
            workflow_dtos::DependencyEdgeKind,
            workflow_dtos::TaskGraphEdge,
            workflow_dtos::TaskGraphResponse,
            workflow_dtos::DelegationGraphNodeData,
            workflow_dtos::DelegationGraphNode,
            workflow_dtos::DelegationGraphEdgeData,
            workflow_dtos::DelegationEdgeKind,
            workflow_dtos::DelegationGraphEdge,
            workflow_dtos::DelegationGraphResponse
        )
    ),
    tags(
        (name = "workspaces", description = "Interview workspace endpoints"),
        (name = "workflows", description = "Multi-agent workflow runs (authorized per run, proxied to intelligence)")
    )
)]
pub struct ApiDoc;

pub fn create_router(state: AppState) -> Router {
    let origins: Vec<HeaderValue> = state
        .config
        .cors_origins
        .iter()
        .filter_map(|s| s.parse::<HeaderValue>().ok())
        .collect();

    let cors = CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::ACCEPT,
            header::COOKIE,
            header::HeaderName::from_static("x-correlation-id"),
        ])
        .allow_credentials(true);
    Router::new()
        .route("/health", get(health::health_check))
        .route("/health/live", get(health::health_check))
        .route("/health/ready", get(health::ready_check))
        .route("/ready", get(health::ready_check))
        .merge(auth::auth_routes(state.clone()))
        .merge(settings::settings_routes(state.clone()))
        .merge(admin::admin_routes(state.clone()))
        .merge(workspaces::workspace_routes(state.clone()))
        .merge(workflows::workflow_routes(state))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(axum::middleware::from_fn(middleware::correlation_id_middleware))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO).latency_unit(LatencyUnit::Millis))
                .on_failure(DefaultOnFailure::new().level(Level::WARN).latency_unit(LatencyUnit::Millis)),
        )
        .layer(cors)
}
