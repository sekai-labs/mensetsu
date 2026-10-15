use axum::{extract::State, http::StatusCode, routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::{
    contexts::identity::domain::{User, UserRole},
    interfaces::http::workspaces::AppState,
    shared::error::AppError,
};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SetupStatusResponseDto {
    pub is_initialized: bool,
    pub has_admin: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct InitializeSetupRequestDto {
    #[validate(length(min = 1, message = "Admin name cannot be empty"))]
    pub admin_name: String,
    #[validate(email(message = "Invalid email format"))]
    pub admin_email: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub admin_password: String,
    pub platform_llm_provider: Option<String>,
    pub platform_llm_api_key: Option<String>,
    pub platform_llm_base_url: Option<String>,
}

pub fn setup_routes(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/setup/status", get(get_setup_status))
        .route("/api/v1/setup/initialize", axum::routing::post(initialize_setup))
        .with_state(state)
}

#[utoipa::path(
    get,
    path = "/api/v1/setup/status",
    responses(
        (status = 200, description = "Setup status", body = SetupStatusResponseDto)
    )
)]
pub async fn get_setup_status(
    State(state): State<AppState>,
) -> Result<Json<SetupStatusResponseDto>, AppError> {
    let db = &state.db.client;

    let mut cfg_res = db
        .query("SELECT * FROM `system_config` WHERE key = 'initialized';")
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let cfg_rows: Vec<serde_json::Value> = cfg_res.take(0).unwrap_or_default();
    let is_initialized = cfg_rows
        .into_iter()
        .next()
        .and_then(|r| r.get("value").cloned())
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let mut user_res = db
        .query("SELECT count() FROM user WHERE role = 'admin' GROUP ALL;")
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let user_rows: Vec<serde_json::Value> = user_res.take(0).unwrap_or_default();
    let admin_count = user_rows
        .into_iter()
        .next()
        .and_then(|r| r.get("count").cloned())
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    Ok(Json(SetupStatusResponseDto {
        is_initialized,
        has_admin: admin_count > 0,
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/setup/initialize",
    request_body = InitializeSetupRequestDto,
    responses(
        (status = 201, description = "Platform initialized", body = serde_json::Value),
        (status = 409, description = "Already initialized"),
        (status = 400, description = "Validation error")
    )
)]
pub async fn initialize_setup(
    State(state): State<AppState>,
    Json(payload): Json<InitializeSetupRequestDto>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let db = &state.db.client;

    let mut cfg_res = db
        .query("SELECT * FROM `system_config` WHERE key = 'initialized';")
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let cfg_rows: Vec<serde_json::Value> = cfg_res.take(0).unwrap_or_default();
    let already_initialized = cfg_rows
        .into_iter()
        .next()
        .and_then(|r| r.get("value").cloned())
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if already_initialized {
        return Err(AppError::Conflict("Platform has already been initialized".into()));
    }

    use crate::contexts::identity::domain::hasher::{Argon2PasswordHasher, PasswordHasher};
    let hasher = Argon2PasswordHasher::new();
    let password_hash = hasher.hash_password(&payload.admin_password)?;
    let email = crate::contexts::identity::domain::Email::new(&payload.admin_email)?;
    let admin_user = User::new(
        email,
        payload.admin_name.clone(),
        password_hash,
        UserRole::Admin,
    )?;
    let user_id = admin_user.id.0.clone();

    let insert_query = "
        UPSERT type::record('user', $id) SET
            name = $name,
            email = $email,
            password_hash = $password_hash,
            role = 'admin',
            totp_enabled = false,
            created_at = time::now(),
            updated_at = time::now();
    ";
    db.query(insert_query)
        .bind(("id", user_id.clone()))
        .bind(("name", admin_user.name))
        .bind(("email", admin_user.email))
        .bind(("password_hash", admin_user.password_hash))
        .await
        .and_then(|r| r.check())
        .map_err(|e| AppError::Database(format!("Failed to save admin user: {e}")))?;

    if let (Some(provider), Some(api_key)) = (&payload.platform_llm_provider, &payload.platform_llm_api_key) {
        if !provider.trim().is_empty() && !api_key.trim().is_empty() {
            let crypto = crate::shared::encryption::CryptoService::new(&state.config.encryption_key);
            let encrypted = crypto.encrypt(api_key.trim()).map_err(|e| AppError::Internal(e.to_string()))?;
            let prov_query = "
                UPSERT type::record('system_llm_provider', $provider) SET
                    provider_key = $provider,
                    name = $provider,
                    base_url = $base_url,
                    encrypted_api_key = $encrypted,
                    is_enabled = true,
                    allowed_models = [],
                    created_at = time::now(),
                    updated_at = time::now();
            ";
            db.query(prov_query)
                .bind(("provider", provider.trim().to_lowercase()))
                .bind(("base_url", payload.platform_llm_base_url.clone()))
                .bind(("encrypted", encrypted))
                .await
                .and_then(|r| r.check())
                .map_err(|e| AppError::Database(format!("Failed to save platform provider: {e}")))?;
        }
    }

    db.query("UPSERT `system_config`:initialized SET key = 'initialized', value = true, is_secret = false, updated_at = time::now();")
        .await
        .and_then(|r| r.check())
        .map_err(|e| AppError::Database(format!("Failed to mark system initialized: {e}")))?;

    Ok((StatusCode::CREATED, Json(serde_json::json!({
        "status": "initialized",
        "admin_user_id": user_id,
        "admin_email": payload.admin_email
    }))))
}
