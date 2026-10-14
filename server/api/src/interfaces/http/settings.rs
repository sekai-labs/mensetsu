use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::{
    contexts::identity::domain::UserRole,
    interfaces::http::{
        extractors::authenticated_user::AuthenticatedUser,
        workspaces::AppState,
    },
    shared::{encryption::CryptoService, error::AppError, Result},
};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserProfileResponseDto {
    pub id: String,
    pub email: String,
    pub name: String,
    pub role: UserRole,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LlmConfigItemDto {
    pub provider: String,
    pub base_url: Option<String>,
    pub model_chat: Option<String>,
    pub model_search: Option<String>,
    pub model_embed: Option<String>,
    pub is_shared: bool,
    pub has_key: bool,
    pub masked_key: Option<String>,
    pub is_inherited: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct UpsertLlmConfigRequestDto {
    pub provider: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub model_chat: Option<String>,
    pub model_search: Option<String>,
    pub model_embed: Option<String>,
    pub is_shared: Option<bool>,
}

pub fn settings_routes(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/settings/profile", get(get_profile))
        .route("/api/v1/settings/llm", get(get_llm_settings).put(upsert_llm_setting).post(upsert_llm_setting))
        .route("/api/v1/settings/llm/{provider}", delete(delete_llm_setting))
        .with_state(state)
}

fn mask_key(key: &str) -> String {
    if key.len() <= 6 {
        "***".to_string()
    } else {
        let last4 = &key[key.len() - 4..];
        let prefix = if key.starts_with("sk-ant-") {
            "sk-ant-..."
        } else if key.starts_with("sk-") {
            "sk-..."
        } else {
            "..."
        };
        format!("{prefix}{last4}")
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/settings/profile",
    responses(
        (status = 200, description = "Current user profile with role", body = UserProfileResponseDto),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn get_profile(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<Json<UserProfileResponseDto>> {
    let user = state
        .identity_service
        .get_user_by_id(&auth_user.user_id)
        .await?;

    Ok(Json(UserProfileResponseDto {
        id: user.id,
        email: user.email,
        name: user.name,
        role: user.role,
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/settings/llm",
    responses(
        (status = 200, description = "List of configured LLM providers", body = [LlmConfigItemDto]),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn get_llm_settings(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<Json<Vec<LlmConfigItemDto>>> {
    let user = state
        .identity_service
        .get_user_by_id(&auth_user.user_id)
        .await?;
    let db = &state.db.client;
    let crypto = CryptoService::new(&state.config.encryption_key);

    
    let mut res_user = db
        .query("SELECT * FROM user_llm_config WHERE user_id = type::record('user', $uid);")
        .bind(("uid", user.id.clone()))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let user_configs: Vec<serde_json::Value> = res_user.take(0).unwrap_or_default();

    
    let mut shared_configs: Vec<serde_json::Value> = Vec::new();
    if user.role != UserRole::Admin {
        let mut res_shared = db
            .query("SELECT * FROM user_llm_config WHERE is_shared = true;")
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        shared_configs = res_shared.take(0).unwrap_or_default();
    }

    let providers = ["openai", "anthropic", "openrouter", "ollama", "custom"];
    let mut result = Vec::new();

    for p in providers {
        
        let user_cfg = user_configs.iter().find(|c| c["provider"].as_str() == Some(p));
        if let Some(cfg) = user_cfg {
            let enc_key = cfg["encrypted_api_key"].as_str();
            let has_key = enc_key.map(|k| !k.is_empty()).unwrap_or(false);
            let masked = if let Some(ek) = enc_key {
                if let Ok(dec) = crypto.decrypt(ek) {
                    Some(mask_key(&dec))
                } else {
                    Some("sk-***[encrypted]".into())
                }
            } else {
                None
            };

            result.push(LlmConfigItemDto {
                provider: p.to_string(),
                base_url: cfg["base_url"].as_str().map(|s| s.to_string()),
                model_chat: cfg["model_chat"].as_str().map(|s| s.to_string()),
                model_search: cfg["model_search"].as_str().map(|s| s.to_string()),
                model_embed: cfg["model_embed"].as_str().map(|s| s.to_string()),
                is_shared: cfg["is_shared"].as_bool().unwrap_or(false),
                has_key,
                masked_key: masked,
                is_inherited: false,
            });
            continue;
        }

        
        let shared_cfg = shared_configs.iter().find(|c| c["provider"].as_str() == Some(p));
        if let Some(cfg) = shared_cfg {
            let enc_key = cfg["encrypted_api_key"].as_str();
            let has_key = enc_key.map(|k| !k.is_empty()).unwrap_or(false);
            let masked = if has_key {
                Some("sk-*** (Org Shared)".into())
            } else {
                None
            };

            result.push(LlmConfigItemDto {
                provider: p.to_string(),
                base_url: cfg["base_url"].as_str().map(|s| s.to_string()),
                model_chat: cfg["model_chat"].as_str().map(|s| s.to_string()),
                model_search: cfg["model_search"].as_str().map(|s| s.to_string()),
                model_embed: cfg["model_embed"].as_str().map(|s| s.to_string()),
                is_shared: true,
                has_key,
                masked_key: masked,
                is_inherited: true,
            });
            continue;
        }

        
        result.push(LlmConfigItemDto {
            provider: p.to_string(),
            base_url: None,
            model_chat: None,
            model_search: None,
            model_embed: None,
            is_shared: false,
            has_key: false,
            masked_key: None,
            is_inherited: false,
        });
    }

    Ok(Json(result))
}

#[utoipa::path(
    put,
    path = "/api/v1/settings/llm",
    request_body = UpsertLlmConfigRequestDto,
    responses(
        (status = 200, description = "LLM provider settings updated", body = LlmConfigItemDto),
        (status = 400, description = "Invalid payload"),
        (status = 403, description = "Forbidden to share configuration")
    )
)]
pub async fn upsert_llm_setting(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Json(payload): Json<UpsertLlmConfigRequestDto>,
) -> Result<Json<LlmConfigItemDto>> {
    let user = state
        .identity_service
        .get_user_by_id(&auth_user.user_id)
        .await?;

    let provider = payload.provider.to_lowercase();
    let valid_providers = ["openai", "anthropic", "openrouter", "ollama", "custom"];
    if !valid_providers.contains(&provider.as_str()) {
        return Err(AppError::Validation(format!("Unsupported provider '{provider}'")));
    }

    let db = &state.db.client;
    let crypto = CryptoService::new(&state.config.encryption_key);

    
    let mut check_res = db
        .query("SELECT * FROM user_llm_config WHERE user_id = type::record('user', $uid) AND provider = $provider;")
        .bind(("uid", user.id.clone()))
        .bind(("provider", provider.clone()))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let existing_records: Vec<serde_json::Value> = check_res.take(0).unwrap_or_default();
    let existing = existing_records.into_iter().next();

    let encrypted_key = match payload.api_key.as_deref() {
        Some(raw) if !raw.trim().is_empty() => Some(crypto.encrypt(raw.trim())?),
        _ => existing
            .as_ref()
            .and_then(|e| e["encrypted_api_key"].as_str().map(|s| s.to_string())),
    };

    let is_shared = if user.role == UserRole::Admin {
        payload.is_shared.unwrap_or(false)
    } else {
        false
    };

    let base_url = payload.base_url.or_else(|| {
        existing
            .as_ref()
            .and_then(|e| e["base_url"].as_str().map(|s| s.to_string()))
    });
    let model_chat = payload.model_chat.or_else(|| {
        existing
            .as_ref()
            .and_then(|e| e["model_chat"].as_str().map(|s| s.to_string()))
    });
    let model_search = payload.model_search.or_else(|| {
        existing
            .as_ref()
            .and_then(|e| e["model_search"].as_str().map(|s| s.to_string()))
    });
    let model_embed = payload.model_embed.or_else(|| {
        existing
            .as_ref()
            .and_then(|e| e["model_embed"].as_str().map(|s| s.to_string()))
    });

    let upsert_query = "
        UPSERT user_llm_config SET
            user_id = type::record('user', $uid),
            provider = $provider,
            encrypted_api_key = $enc_key,
            base_url = $base_url,
            model_chat = $model_chat,
            model_search = $model_search,
            model_embed = $model_embed,
            is_shared = $is_shared,
            updated_at = time::now()
        WHERE user_id = type::record('user', $uid) AND provider = $provider;
    ";

    let _ = db
        .query(upsert_query)
        .bind(("uid", user.id.clone()))
        .bind(("provider", provider.clone()))
        .bind(("enc_key", encrypted_key.clone()))
        .bind(("base_url", base_url.clone()))
        .bind(("model_chat", model_chat.clone()))
        .bind(("model_search", model_search.clone()))
        .bind(("model_embed", model_embed.clone()))
        .bind(("is_shared", is_shared))
        .await
        .map_err(|e| AppError::Database(format!("Failed to save LLM config: {e}")))?;

    let has_key = encrypted_key.is_some();
    let masked = if let Some(ek) = encrypted_key {
        if let Ok(dec) = crypto.decrypt(&ek) {
            Some(mask_key(&dec))
        } else {
            Some("sk-***".into())
        }
    } else {
        None
    };

    Ok(Json(LlmConfigItemDto {
        provider,
        base_url,
        model_chat,
        model_search,
        model_embed,
        is_shared,
        has_key,
        masked_key: masked,
        is_inherited: false,
    }))
}

#[utoipa::path(
    delete,
    path = "/api/v1/settings/llm/{provider}",
    responses(
        (status = 200, description = "LLM provider setting removed"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn delete_llm_setting(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(provider): Path<String>,
) -> Result<StatusCode> {
    let db = &state.db.client;
    let _ = db
        .query("DELETE user_llm_config WHERE user_id = type::record('user', $uid) AND provider = $provider;")
        .bind(("uid", auth_user.user_id.0))
        .bind(("provider", provider.to_lowercase()))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(StatusCode::OK)
}
