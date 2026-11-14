pub mod admin;
pub mod application_dispatch;
pub mod applications;
pub mod auth;
pub mod extractors;
pub mod health;
pub mod metrics;
pub mod middleware;
pub mod oauth;
pub mod resume_uploads;
pub mod saved_jobs;
pub mod settings;
pub mod setup;
pub mod sse_parser;
pub mod token_usage;
pub mod workflow_dtos;
pub mod workflows;
pub mod workspaces;
use axum::{
    http::{header, HeaderValue, Method},
    routing::get,
    Router,
};
use token_usage::{TokenUsageResponseDto, TokenUsageRowDto, TokenUsageTotalsDto};
use tower_http::cors::CorsLayer;
use tower_http::trace::{DefaultMakeSpan, DefaultOnFailure, DefaultOnResponse, TraceLayer};
use tower_http::LatencyUnit;
use tracing::Level;
use utoipa::OpenApi;

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
    modifiers(&ApiInfoModifier),
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
        oauth::list_providers,
        oauth::authorize_login,
        oauth::authorize_link,
        oauth::oauth_callback,
        oauth::exchange_handoff,
        oauth::list_accounts,
        oauth::unlink_account,
        oauth::admin_list_providers,
        oauth::admin_upsert_provider,
        oauth::admin_delete_provider,
        applications::list_applications,
        applications::get_application,
        applications::prepare_application,
        applications::reprepare_application,
        applications::approve_application,
        applications::start_application,
        applications::block_application,
        applications::record_result,
        applications::reconcile_application,
        saved_jobs::list_saved_jobs,
        saved_jobs::create_saved_job,
        saved_jobs::approve_saved_job,
        saved_jobs::dismiss_saved_job,
        saved_jobs::prepare_saved_job,
        resume_uploads::create_resume_upload,
        resume_uploads::complete_resume_upload,
        resume_uploads::list_resume_uploads,
        resume_uploads::download_resume_upload,
        resume_uploads::delete_resume_upload,
        workspaces::create_workspace,
        workspaces::list_workspaces,
        workspaces::get_workspace,
        workspaces::toggle_learning_unit_completed,
        workspaces::learning_handlers::get_workspace_match,
        workspaces::learning_handlers::get_workspace_knowledge,
        workspaces::learning_handlers::get_workspace_learning,
        workspaces::learning_handlers::get_workspace_questions,
        workspaces::research_handlers::get_workspace_company,
        workspaces::event_handlers::workspace_events,
        workspaces::role_handlers::get_workspace_role,
        workspaces::role_handlers::import_job_description_url,
        workspaces::research_handlers::get_workspace_sources,
        workspaces::research_handlers::get_workspace_claims,
        workspaces::research_handlers::ask_workspace_knowledge_qa,
        settings::get_profile,
        settings::get_candidate_profile,
        settings::update_candidate_profile,
        settings::get_llm_settings,
        settings::upsert_llm_setting,
        settings::delete_llm_setting,
        settings::get_combined_providers,
        settings::upsert_user_provider,
        settings::delete_user_provider,
        settings::get_user_routing,
        settings::update_user_routing,
        settings::discover_models,
        admin::list_admin_providers,
        admin::upsert_admin_provider,
        admin::get_admin_settings,
        admin::update_admin_settings,
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
        metrics::prometheus_metrics,
        metrics::metrics_summary,
        token_usage::user_token_usage,
        token_usage::admin_token_usage,
        token_usage::user_daily_token_usage,
        token_usage::admin_daily_token_usage,
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
            applications::PrepareApplicationRequestDto,
            applications::ApproveApplicationRequestDto,
            applications::BlockApplicationRequestDto,
            applications::ResultApplicationRequestDto,
            applications::ReconcileApplicationRequestDto,
            applications::PreparedFieldsDto,
            applications::JobApplicationResponseDto,
            applications::ApplicationSite,
            applications::ApplicationStatus,
            applications::ApplicationOutcome,
            applications::BlockReasonCode,
            saved_jobs::CreateSavedJobRequestDto,
            saved_jobs::PrepareSavedJobRequestDto,
            saved_jobs::SavedJobResponseDto,
            saved_jobs::SavedJobsPageResponseDto,
            saved_jobs::SavedJobStatus,
            applications::ApplicationMethod,
            settings::CandidateProfileDto,
            settings::UpdateCandidateProfileRequestDto,
            WorkspaceId,
            WorkspaceStatus,
            PipelinePhase,
            SeniorityLevel,
            Workspace,
            CreateWorkspaceRequestDto,
            WorkspaceResponseDto,
            WorkspaceProgressEventDto,
            workspaces::role_handlers::ImportJdUrlRequestDto,
            workspaces::research_handlers::KnowledgeQaRequestDto,
            resume_uploads::CreateResumeUploadRequestDto,
            resume_uploads::CreateResumeUploadResponseDto,
            resume_uploads::CompleteResumeUploadResponseDto,
            StrengthEvidence,
            GapEvidence,
            WordingImprovement,
            MatchAnalysis,
            SourceCitation,
            CompanyClaim,
            KnowledgeSkillNode,
            KnowledgeEdge,
            KnowledgeGraphResponse,
            metrics::ServiceMetricsDto,
            metrics::MetricsSummaryResponseDto,
            token_usage::DailyTokenUsageDto,
            token_usage::DailyTokenUsageResponseDto,
            TokenUsageResponseDto,
            TokenUsageRowDto,
            TokenUsageTotalsDto,
            token_usage::TokenUsageCursorDto,
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
            settings::DiscoverModelsRequestDto,
            settings::ProviderStatusItemDto,
            settings::UpsertUserProviderRequestDto,
            settings::ModelRoutingSlotDto,
            settings::ModelRoutingConfigDto,
            settings::UserRoutingResponseDto,
            settings::UpdateUserRoutingRequestDto,
            admin::AdminProviderDto,
            admin::UpsertAdminProviderRequestDto,
crate::contexts::identity::application::oauth_dtos::OAuthProviderSummaryDto,
crate::contexts::identity::application::oauth_dtos::OAuthProviderAdminDto,
crate::contexts::identity::application::oauth_dtos::OAuthProviderUpsertRequestDto,
crate::contexts::identity::application::oauth_dtos::OAuthIdentityDto,
crate::contexts::identity::application::oauth_dtos::OAuthLinkStartResponseDto,
crate::contexts::identity::application::oauth_dtos::OAuthExchangeRequestDto,
crate::contexts::identity::application::oauth_dtos::OAuthExchangeResponseDto,
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
        (name = "workflows", description = "Multi-agent workflow runs (authorized per run, proxied to intelligence)"),
        (name = "applications", description = "Owner-bound job application preparation and tracking"),
        (name = "saved-jobs", description = "Owner-bound saved jobs and email-first application flow")
    )
)]
pub struct ApiDoc;

struct ApiInfoModifier;

impl utoipa::Modify for ApiInfoModifier {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        openapi.info.license = Some(
            utoipa::openapi::info::LicenseBuilder::new()
                .name("Apache-2.0")
                .identifier(Some("Apache-2.0"))
                .build(),
        );
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearerAuth",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::Http::new(
                        utoipa::openapi::security::HttpAuthScheme::Bearer,
                    ),
                ),
            );
        }
        openapi.security = Some(vec![utoipa::openapi::security::SecurityRequirement::new(
            "bearerAuth",
            Vec::<String>::new(),
        )]);
    }
}

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
        .merge(metrics::metrics_routes(state.clone()))
        .merge(auth::auth_routes(state.clone()))
        .merge(oauth::oauth_routes(state.clone()))
        .merge(setup::setup_routes(state.clone()))
        .merge(settings::settings_routes(state.clone()))
        .merge(admin::admin_routes(state.clone()))
        .merge(workspaces::workspace_routes(state.clone()))
        .merge(applications::application_routes(state.clone()))
        .merge(saved_jobs::saved_job_routes(state.clone()))
        .merge(token_usage::token_usage_routes(state.clone()))
        .merge(
            Router::new()
                .route(
                    "/api/v1/resume-uploads",
                    get(resume_uploads::list_resume_uploads)
                        .post(resume_uploads::create_resume_upload),
                )
                .route(
                    "/api/v1/resume-uploads/{upload_id}",
                    axum::routing::delete(resume_uploads::delete_resume_upload),
                )
                .route(
                    "/api/v1/resume-uploads/{upload_id}/download",
                    get(resume_uploads::download_resume_upload),
                )
                .route(
                    "/api/v1/resume-uploads/{upload_id}/complete",
                    axum::routing::post(resume_uploads::complete_resume_upload),
                )
                .with_state(state.clone()),
        )
        .merge(workflows::workflow_routes(state.clone()))
        .layer(axum::middleware::from_fn_with_state(
            state.metrics.clone(),
            metrics::metrics_middleware,
        ))
        .layer(axum::middleware::from_fn(
            middleware::correlation_id_middleware,
        ))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(
                    DefaultOnResponse::new()
                        .level(Level::INFO)
                        .latency_unit(LatencyUnit::Millis),
                )
                .on_failure(
                    DefaultOnFailure::new()
                        .level(Level::WARN)
                        .latency_unit(LatencyUnit::Millis),
                ),
        )
        .layer(cors)
}
