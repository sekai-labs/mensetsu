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

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ProviderStatusItemDto {
    pub provider_key: String,
    pub name: String,
    pub base_url: Option<String>,
    pub active_key_source: String, // "user" | "system" | "none"
    pub is_user_override: bool,
    pub is_enabled: bool,
    pub allowed_models: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpsertUserProviderRequestDto {
    pub provider_key: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ModelRoutingSlotDto {
    pub provider_key: String,
    pub model: String,
    pub temperature: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ModelRoutingConfigDto {
    pub primary: ModelRoutingSlotDto,
    pub fallback: Option<ModelRoutingSlotDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserRoutingResponseDto {
    pub slots: std::collections::HashMap<String, ModelRoutingConfigDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateUserRoutingRequestDto {
    pub slots: std::collections::HashMap<String, ModelRoutingConfigDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DiscoverModelsRequestDto {
    pub provider_key: String,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
}

pub fn settings_routes(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/settings/profile", get(get_profile))
        .route("/api/v1/settings/llm", get(get_llm_settings).put(upsert_llm_setting).post(upsert_llm_setting))
        .route("/api/v1/settings/llm/{provider}", delete(delete_llm_setting))
        .route("/api/v1/settings/llm/providers", get(get_combined_providers).put(upsert_user_provider).post(upsert_user_provider))
        .route("/api/v1/settings/llm/providers/{provider_key}", delete(delete_user_provider))
        .route("/api/v1/settings/llm/routing", get(get_user_routing).put(update_user_routing))
        .route("/api/v1/settings/llm/models", axum::routing::post(discover_models))
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
#[utoipa::path(
    get,
    path = "/api/v1/settings/llm/providers",
    responses(
        (status = 200, description = "Combined LLM provider list with active key source", body = [ProviderStatusItemDto])
    )
)]
pub async fn get_combined_providers(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<Json<Vec<ProviderStatusItemDto>>> {
    let db = &state.db.client;

    let mut sys_res = db
        .query("SELECT * FROM system_llm_provider WHERE is_enabled = true;")
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let system_providers: Vec<serde_json::Value> = sys_res.take(0).unwrap_or_default();

    let mut user_res = db
        .query("SELECT * FROM user_llm_provider WHERE user_id = type::record('user', $uid);")
        .bind(("uid", auth_user.user_id.0.clone()))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let user_providers: Vec<serde_json::Value> = user_res.take(0).unwrap_or_default();

    let default_providers = [
        ("openai", "OpenAI"),
        ("anthropic", "Anthropic"),
        ("gemini", "Google Gemini"),
        ("openrouter", "OpenRouter"),
        ("ollama", "Ollama (Local)"),
    ];

    let mut result = Vec::new();
    let mut seen_keys = std::collections::HashSet::new();

    for sp in &system_providers {
        let pkey = sp["provider_key"].as_str().unwrap_or_default().to_string();
        let name = sp["name"].as_str().unwrap_or(&pkey).to_string();
        let base_url = sp["base_url"].as_str().map(|s| s.to_string());
        let allowed_models = sp["allowed_models"]
            .as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();

        seen_keys.insert(pkey.clone());
        let user_override = user_providers.iter().find(|up| up["provider_key"].as_str() == Some(&pkey));

        let (source, is_override) = if let Some(up) = user_override {
            let key_exists = up["encrypted_api_key"].as_str().map(|k| !k.is_empty()).unwrap_or(false);
            if key_exists {
                ("user".to_string(), true)
            } else {
                let sys_has_key = sp["encrypted_api_key"].as_str().map(|k| !k.is_empty()).unwrap_or(false);
                if sys_has_key { ("system".to_string(), false) } else { ("none".to_string(), false) }
            }
        } else {
            let sys_has_key = sp["encrypted_api_key"].as_str().map(|k| !k.is_empty()).unwrap_or(false);
            if sys_has_key { ("system".to_string(), false) } else { ("none".to_string(), false) }
        };

        result.push(ProviderStatusItemDto {
            provider_key: pkey,
            name,
            base_url,
            active_key_source: source,
            is_user_override: is_override,
            is_enabled: true,
            allowed_models,
        });
    }

    for (def_key, def_name) in default_providers {
        if !seen_keys.contains(def_key) {
            let user_override = user_providers.iter().find(|up| up["provider_key"].as_str() == Some(def_key));
            let (source, is_override) = if let Some(up) = user_override {
                let key_exists = up["encrypted_api_key"].as_str().map(|k| !k.is_empty()).unwrap_or(false);
                if key_exists { ("user".to_string(), true) } else { ("none".to_string(), false) }
            } else {
                ("none".to_string(), false)
            };
            let base_url = user_override.and_then(|up| up["base_url"].as_str().map(String::from));

            result.push(ProviderStatusItemDto {
                provider_key: def_key.to_string(),
                name: def_name.to_string(),
                base_url,
                active_key_source: source,
                is_user_override: is_override,
                is_enabled: true,
                allowed_models: Vec::new(),
            });
        }
    }
    Ok(Json(result))
}

#[utoipa::path(
    put,
    path = "/api/v1/settings/llm/providers",
    request_body = UpsertUserProviderRequestDto,
    responses(
        (status = 200, description = "User BYOK provider credentials updated")
    )
)]
pub async fn upsert_user_provider(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Json(payload): Json<UpsertUserProviderRequestDto>,
) -> Result<StatusCode> {
    let db = &state.db.client;
    let crypto = CryptoService::new(&state.config.encryption_key);
    let pkey = payload.provider_key.to_lowercase();

    let enc_key = match payload.api_key.as_deref() {
        Some(raw) if !raw.trim().is_empty() => Some(crypto.encrypt(raw.trim())?),
        _ => None,
    };

    let rec_id = format!("{}_{}", auth_user.user_id.0, pkey);
    let query = "
        UPSERT type::record('user_llm_provider', $id) SET
            user_id = type::record('user', $uid),
            provider_key = $pkey,
            encrypted_api_key = $enc_key,
            base_url = $base_url,
            updated_at = time::now();
    ";
    db.query(query)
        .bind(("id", rec_id))
        .bind(("uid", auth_user.user_id.0))
        .bind(("pkey", pkey))
        .bind(("enc_key", enc_key))
        .bind(("base_url", payload.base_url))
        .await
        .and_then(|r| r.check())
        .map_err(|e| AppError::Database(format!("Failed to save user provider: {e}")))?;

    Ok(StatusCode::OK)
}

#[utoipa::path(
    delete,
    path = "/api/v1/settings/llm/providers/{provider_key}",
    responses(
        (status = 200, description = "User provider override reverted to admin")
    )
)]
pub async fn delete_user_provider(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(provider_key): Path<String>,
) -> Result<StatusCode> {
    let db = &state.db.client;
    let pkey = provider_key.to_lowercase();
    let rec_id = format!("{}_{}", auth_user.user_id.0, pkey);

    db.query("DELETE type::record('user_llm_provider', $id);")
        .bind(("id", rec_id))
        .await
        .and_then(|r| r.check())
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(StatusCode::OK)
}

#[utoipa::path(
    get,
    path = "/api/v1/settings/llm/routing",
    responses(
        (status = 200, description = "User model routing matrix", body = UserRoutingResponseDto)
    )
)]
pub async fn get_user_routing(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<Json<UserRoutingResponseDto>> {
    let db = &state.db.client;
    let mut res = db
        .query("SELECT slots FROM type::record('user_model_routing', $uid);")
        .bind(("uid", auth_user.user_id.0.clone()))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let rows: Vec<serde_json::Value> = res.take(0).unwrap_or_default();
    let slots_val = rows.into_iter().next().and_then(|r| r.get("slots").cloned()).unwrap_or(serde_json::json!({}));

    let slots: std::collections::HashMap<String, ModelRoutingConfigDto> = serde_json::from_value(slots_val).unwrap_or_default();
    Ok(Json(UserRoutingResponseDto { slots }))
}

#[utoipa::path(
    put,
    path = "/api/v1/settings/llm/routing",
    request_body = UpdateUserRoutingRequestDto,
    responses(
        (status = 200, description = "Model routing matrix updated")
    )
)]
pub async fn update_user_routing(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Json(payload): Json<UpdateUserRoutingRequestDto>,
) -> Result<StatusCode> {
    let db = &state.db.client;
    let slots_val = serde_json::to_value(&payload.slots).map_err(|e| AppError::Validation(e.to_string()))?;

    let query = "
        UPSERT type::record('user_model_routing', $uid) SET
            user_id = type::record('user', $uid),
            slots = $slots,
            updated_at = time::now();
    ";
    db.query(query)
        .bind(("uid", auth_user.user_id.0.clone()))
        .bind(("slots", slots_val))
        .await
        .and_then(|r| r.check())
        .map_err(|e| AppError::Database(format!("Failed to update model routing: {e}")))?;

    Ok(StatusCode::OK)
}

#[utoipa::path(
    post,
    path = "/api/v1/settings/llm/models",
    request_body = DiscoverModelsRequestDto,
    responses(
        (status = 200, description = "Discovered models list", body = serde_json::Value)
    )
)]
pub async fn discover_models(
    State(_state): State<AppState>,
    _auth_user: AuthenticatedUser,
    Json(payload): Json<DiscoverModelsRequestDto>,
) -> Result<Json<serde_json::Value>> {
    let base_url = payload.base_url.unwrap_or_else(|| match payload.provider_key.to_lowercase().as_str() {
        "openai" => "https://api.openai.com/v1".to_string(),
        "openrouter" => "https://openrouter.ai/api/v1".to_string(),
        "anthropic" => "https://api.anthropic.com/v1".to_string(),
        _ => "https://api.openai.com/v1".to_string(),
    });

    if base_url.starts_with("http://127.") || base_url.starts_with("http://192.168.") || base_url.starts_with("http://10.") || base_url.starts_with("http://169.254.") {
        return Err(AppError::Forbidden("Private network URLs are forbidden".into()));
    }

    let models_url = format!("{}/models", base_url.trim_end_matches('/'));
    let client = reqwest::Client::new();
    let mut req = client.get(&models_url).timeout(std::time::Duration::from_secs(5));
    if let Some(key) = payload.api_key.filter(|k| !k.is_empty()) {
        req = req.header("Authorization", format!("Bearer {key}"));
    }

    match req.send().await {
        Ok(res) if res.status().is_success() => {
            let val: serde_json::Value = res.json().await.unwrap_or(serde_json::json!({ "data": [] }));
            Ok(Json(val))
        }
        _ => {
            let defaults = match payload.provider_key.to_lowercase().as_str() {
                "anthropic" => vec!["claude-3-5-sonnet-20241022", "claude-3-5-haiku-20241022", "claude-3-opus-20240229"],
                "openai" => vec!["gpt-4o", "gpt-4o-mini", "o1-mini", "o1-preview"],
                "gemini" => vec!["gemini-1.5-pro", "gemini-1.5-flash", "gemini-2.0-flash-exp"],
                _ => vec!["default-model"],
            };
            let data: Vec<serde_json::Value> = defaults.into_iter().map(|m| serde_json::json!({ "id": m })).collect();
            Ok(Json(serde_json::json!({ "data": data })))
        }
    }
}

