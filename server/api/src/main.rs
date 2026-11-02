use mensetsu_api::{
    contexts,
    interfaces::{
        grpc::{proto::resume::resume_storage_service_server::ResumeStorageServiceServer, resume_storage::ResumeStorageGrpcService},
        http::{create_router, workspaces::AppState},
    },
    shared::config::AppConfig,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer().with_ansi(false))
        .init();

    let config = AppConfig::from_env();
    tracing::info!("Connecting to SurrealDB at {}", config.surreal_url);

    let db = contexts::workspace::infrastructure::db::DatabasePool::connect(&config).await?;
    let storage = std::sync::Arc::new(
        contexts::workspace::infrastructure::storage::ResumeObjectStore::from_config(&config).await?,
    );
    let user_repo = std::sync::Arc::new(contexts::identity::infrastructure::SurrealUserRepository::new(db.client.clone()));
    let session_repo = std::sync::Arc::new(contexts::identity::infrastructure::SurrealAuthSessionRepository::new(db.client.clone()));
    let hasher = std::sync::Arc::new(contexts::identity::domain::hasher::Argon2PasswordHasher::new());
    let token_service = contexts::identity::domain::token::TokenService::new(
        &config.jwt_secret,
        config.access_token_ttl_seconds,
        config.refresh_token_ttl_seconds,
    );
    let identity_service = std::sync::Arc::new(contexts::identity::application::IdentityService::new(
        user_repo,
        session_repo,
        hasher,
        token_service,
    ));
    let ws_repo = std::sync::Arc::new(contexts::workspace::infrastructure::SurrealWorkspaceRepository::new((*db.client).clone()));
    let workspace_service = std::sync::Arc::new(contexts::workspace::application::WorkspaceService::new(ws_repo.clone()));

    let wf_repo = std::sync::Arc::new(contexts::workflow::infrastructure::SurrealWorkflowRunRepository::new((*db.client).clone()));
    let workflow_service = std::sync::Arc::new(contexts::workflow::application::WorkflowService::new(wf_repo, ws_repo));
    tracing::info!("Connecting to Intelligence gRPC at {}", config.intelligence_grpc_url);
    let grpc_client = std::sync::Arc::new(mensetsu_api::interfaces::grpc::GrpcClient::connect(&config).await?);
    tracing::info!("Successfully connected to Intelligence gRPC service");
    let grpc_service = ResumeStorageGrpcService::new(
        db.clone(),
        storage.clone(),
        &config.internal_service_token,
    );
    let state = AppState {
        db,
        config: config.clone(),
        identity_service,
        workspace_service,
        workflow_service,
        grpc_client,
        storage,
    };
    let app = create_router(state);
    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Mensetsu Core API listening on {}", addr);
    tracing::info!("OpenAPI Swagger UI at http://{}/swagger-ui", addr);

    let grpc_addr: std::net::SocketAddr = config.api_grpc_bind.parse()?;
    let grpc_server = tonic::transport::Server::builder()
        .add_service(ResumeStorageServiceServer::new(grpc_service))
        .serve(grpc_addr);
    tracing::info!(%grpc_addr, "Internal resume gRPC service listening");
    tokio::select! {
        result = axum::serve(listener, app) => result?,
        result = grpc_server => result?,
    }
    Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "API server stopped").into())
}
