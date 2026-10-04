pub mod admin;
pub mod auth;
pub mod extractors;
pub mod health;
pub mod middleware;
pub mod settings;
pub mod workspaces;
use axum::{
    http::{header, HeaderValue, Method},
    routing::get,
    Router,
};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
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
        settings::delete_llm_setting
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
            settings::UpsertLlmConfigRequestDto
        )
    ),
    tags(
        (name = "workspaces", description = "Interview workspace endpoints")
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
        .merge(workspaces::workspace_routes(state))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(axum::middleware::from_fn(middleware::correlation_id_middleware))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
}
