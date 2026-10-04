use mensetsu_api::{
    contexts,
    interfaces::http::{create_router, workspaces::AppState},
    shared::config::AppConfig,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = AppConfig::from_env();
    tracing::info!("Connecting to SurrealDB at {}", config.surreal_url);

    let db = contexts::workspace::infrastructure::db::DatabasePool::connect(&config).await?;
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

    let state = AppState {
        db,
        config: config.clone(),
        identity_service,
    };
    let app = create_router(state);
    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Mensetsu Core API listening on {}", addr);
    tracing::info!("OpenAPI Swagger UI at http://{}/swagger-ui", addr);

    axum::serve(listener, app).await?;
    Ok(())
}
