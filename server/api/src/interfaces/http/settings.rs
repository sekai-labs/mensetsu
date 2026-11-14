use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, SocketAddr};
use utoipa::ToSchema;
use validator::Validate;

use crate::{
    contexts::identity::domain::UserRole,
    interfaces::http::{extractors::authenticated_user::AuthenticatedUser, workspaces::AppState},
    shared::{encryption::CryptoService, error::AppError, Result},
};

#[derive(Debug, Clone, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct DiscoverModelsRequestDto {
    pub provider_key: String,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
}

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
    pub active_key_source: String,
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
#[serde(deny_unknown_fields)]
pub struct CandidateProfileDto {
    pub phone: Option<String>,
    pub location: Option<String>,
    pub linkedin_url: Option<String>,
    pub github_url: Option<String>,
    pub portfolio_url: Option<String>,
    pub target_role: Option<String>,
    pub required_keywords: Vec<String>,
    pub excluded_keywords: Vec<String>,
    pub target_locations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct UpdateCandidateProfileRequestDto {
    pub phone: Option<String>,
    pub location: Option<String>,
    pub linkedin_url: Option<String>,
    pub github_url: Option<String>,
    pub portfolio_url: Option<String>,
    pub target_role: Option<String>,
    pub required_keywords: Vec<String>,
    pub excluded_keywords: Vec<String>,
    pub target_locations: Vec<String>,
}

fn validate_candidate_profile(payload: &UpdateCandidateProfileRequestDto) -> Result<()> {
    for value in [
        &payload.phone,
        &payload.location,
        &payload.linkedin_url,
        &payload.github_url,
        &payload.portfolio_url,
        &payload.target_role,
    ] {
        if value.as_ref().is_some_and(|v| v.trim().is_empty()) {
            return Err(AppError::Validation(
                "Candidate profile values cannot be blank".into(),
            ));
        }
    }
    if payload.phone.as_ref().is_some_and(|phone| {
        !(7..=20).contains(&phone.chars().count())
            || phone.chars().filter(|c| c.is_ascii_digit()).count() < 7
            || phone
                .chars()
                .any(|c| !(c.is_ascii_digit() || "+()-. ".contains(c)))
    }) || payload
        .location
        .as_ref()
        .is_some_and(|v| v.chars().count() > 120)
        || payload
            .target_role
            .as_ref()
            .is_some_and(|v| v.chars().count() > 120)
    {
        return Err(AppError::Validation(
            "Invalid phone, location, or target role".into(),
        ));
    }
    for url in [
        &payload.linkedin_url,
        &payload.github_url,
        &payload.portfolio_url,
    ]
    .into_iter()
    .flatten()
    {
        if url.chars().count() > 2048
            || reqwest::Url::parse(url)
                .map(|parsed| {
                    parsed.scheme() != "https"
                        || !parsed.username().is_empty()
                        || parsed.password().is_some()
                })
                .unwrap_or(true)
        {
            return Err(AppError::Validation(
                "Profile URLs must be HTTPS without userinfo and at most 2048 characters".into(),
            ));
        }
    }
    let arrays = [
        &payload.required_keywords,
        &payload.excluded_keywords,
        &payload.target_locations,
    ];
    if arrays.iter().any(|items| {
        items.len() > 20
            || items
                .iter()
                .any(|v| !(1..=80).contains(&v.chars().count()) || v.trim().is_empty())
    }) {
        return Err(AppError::Validation(
            "Invalid profile preference arrays".into(),
        ));
    }
    let unique = |items: &[String]| {
        let mut seen = std::collections::HashSet::new();
        items.iter().all(|item| seen.insert(item.to_lowercase()))
    };
    if !arrays.iter().all(|items| unique(items))
        || payload.required_keywords.iter().any(|required| {
            payload
                .excluded_keywords
                .iter()
                .any(|excluded| required.to_lowercase() == excluded.to_lowercase())
        })
    {
        return Err(AppError::Validation(
            "Preferences must be case-insensitively unique with no required/excluded overlap"
                .into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod candidate_profile_validation_tests {
    use super::*;

    fn profile() -> UpdateCandidateProfileRequestDto {
        UpdateCandidateProfileRequestDto {
            phone: Some("+1 (555) 123-4567".into()),
            location: Some("Montreal".into()),
            linkedin_url: Some("https://linkedin.com/in/candidate".into()),
            github_url: None,
            portfolio_url: None,
            target_role: Some("Engineer".into()),
            required_keywords: vec!["Rust".into()],
            excluded_keywords: vec!["Java".into()],
            target_locations: vec!["Montreal".into()],
        }
    }

    #[test]
    fn candidate_profile_accepts_valid_data_and_rejects_invalid_constraints() {
        assert!(validate_candidate_profile(&profile()).is_ok());
        let mut invalid = profile();
        invalid.phone = Some("123-456".into());
        assert!(validate_candidate_profile(&invalid).is_err());
        let mut invalid = profile();
        invalid.target_role = Some("x".repeat(121));
        assert!(validate_candidate_profile(&invalid).is_err());
        let mut invalid = profile();
        invalid.linkedin_url = Some("http://linkedin.com/in/candidate".into());
        assert!(validate_candidate_profile(&invalid).is_err());
        let mut invalid = profile();
        invalid.required_keywords.push("rust".into());
        assert!(validate_candidate_profile(&invalid).is_err());
        let mut invalid = profile();
        invalid.excluded_keywords.push("RUST".into());
        assert!(validate_candidate_profile(&invalid).is_err());
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredCandidateProfile {
    phone: Option<String>,
    location: Option<String>,
    linkedin_url: Option<String>,
    github_url: Option<String>,
    portfolio_url: Option<String>,
    target_role: Option<String>,
    required_keywords: Vec<String>,
    excluded_keywords: Vec<String>,
    target_locations: Vec<String>,
}

pub fn settings_routes(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/settings/profile", get(get_profile))
        .route(
            "/api/v1/candidate-profile",
            get(get_candidate_profile).put(update_candidate_profile),
        )
        .route(
            "/api/v1/settings/llm",
            get(get_llm_settings)
                .put(upsert_llm_setting)
                .post(upsert_llm_setting),
        )
        .route(
            "/api/v1/settings/llm/{provider}",
            delete(delete_llm_setting),
        )
        .route(
            "/api/v1/settings/llm/providers",
            get(get_combined_providers)
                .put(upsert_user_provider)
                .post(upsert_user_provider),
        )
        .route(
            "/api/v1/settings/llm/providers/{provider_key}",
            delete(delete_user_provider),
        )
        .route(
            "/api/v1/settings/llm/routing",
            get(get_user_routing).put(update_user_routing),
        )
        .route(
            "/api/v1/settings/llm/models",
            axum::routing::post(discover_models),
        )
        .with_state(state)
}

fn mask_key(key: &str) -> String {
    let character_count = key.chars().count();
    if character_count <= 6 {
        return "***".to_string();
    }

    let last_four_start = key
        .char_indices()
        .rev()
        .nth(3)
        .map(|(index, _)| index)
        .unwrap_or(0);
    let prefix = if key.starts_with("sk-ant-") {
        "sk-ant-..."
    } else if key.starts_with("sk-") {
        "sk-..."
    } else {
        "..."
    };
    format!("{prefix}{}", &key[last_four_start..])
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
    path = "/api/v1/candidate-profile",
    responses((status = 200, body = CandidateProfileDto), (status = 401, description = "Unauthorized"))
)]
pub async fn get_candidate_profile(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<Json<CandidateProfileDto>> {
    let mut response = state.db.client
        .query("SELECT phone, location, linkedin_url, github_url, portfolio_url, target_role, required_keywords, excluded_keywords, target_locations FROM candidate_profile WHERE user_id = type::record('user', $uid) LIMIT 1;")
        .bind(("uid", auth_user.user_id.0))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let rows: Vec<serde_json::Value> = response
        .take(0)
        .map_err(|e| AppError::Database(e.to_string()))?;
    let profile = rows
        .into_iter()
        .next()
        .map(serde_json::from_value)
        .transpose()
        .map_err(|e| AppError::Database(format!("Malformed candidate profile: {e}")))?
        .unwrap_or(StoredCandidateProfile {
            phone: None,
            location: None,
            linkedin_url: None,
            github_url: None,
            portfolio_url: None,
            target_role: None,
            required_keywords: vec![],
            excluded_keywords: vec![],
            target_locations: vec![],
        });
    Ok(Json(CandidateProfileDto {
        phone: profile.phone,
        location: profile.location,
        linkedin_url: profile.linkedin_url,
        github_url: profile.github_url,
        portfolio_url: profile.portfolio_url,
        target_role: profile.target_role,
        required_keywords: profile.required_keywords,
        excluded_keywords: profile.excluded_keywords,
        target_locations: profile.target_locations,
    }))
}

#[utoipa::path(
    put,
    path = "/api/v1/candidate-profile",
    request_body = UpdateCandidateProfileRequestDto,
    responses((status = 200, body = CandidateProfileDto), (status = 400, description = "Invalid profile"), (status = 401, description = "Unauthorized"))
)]
pub async fn update_candidate_profile(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Json(payload): Json<UpdateCandidateProfileRequestDto>,
) -> Result<Json<CandidateProfileDto>> {
    validate_candidate_profile(&payload)?;
    let user_id = auth_user.user_id.0;
    let candidate_profile_id = format!(
        "candidate_{}",
        user_id.replace("user:", "").replace('`', "")
    );
    let mut response = state.db.client.query(
        "UPSERT type::record('candidate_profile', $id) SET user_id = type::record('user', $uid), phone = $phone, location = $location, linkedin_url = $linkedin, github_url = $github, portfolio_url = $portfolio, target_role = $role, required_keywords = $required, excluded_keywords = $excluded, target_locations = $locations, updated_at = time::now() RETURN AFTER;"
    )
        .bind(("uid", user_id))
        .bind(("id", candidate_profile_id))
        .bind(("phone", payload.phone)).bind(("location", payload.location))
        .bind(("linkedin", payload.linkedin_url)).bind(("github", payload.github_url))
        .bind(("portfolio", payload.portfolio_url)).bind(("role", payload.target_role))
        .bind(("required", payload.required_keywords)).bind(("excluded", payload.excluded_keywords))
        .bind(("locations", payload.target_locations))
        .await.map_err(|e| AppError::Database(e.to_string()))?;
    let rows: Vec<serde_json::Value> = response
        .take(0)
        .map_err(|e| AppError::Database(e.to_string()))?;
    let p: StoredCandidateProfile = rows
        .into_iter()
        .next()
        .ok_or_else(|| AppError::Database("Candidate profile upsert returned no profile".into()))
        .and_then(|value| {
            serde_json::from_value(value)
                .map_err(|e| AppError::Database(format!("Malformed candidate profile: {e}")))
        })?;
    Ok(Json(CandidateProfileDto {
        phone: p.phone,
        location: p.location,
        linkedin_url: p.linkedin_url,
        github_url: p.github_url,
        portfolio_url: p.portfolio_url,
        target_role: p.target_role,
        required_keywords: p.required_keywords,
        excluded_keywords: p.excluded_keywords,
        target_locations: p.target_locations,
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
        let user_cfg = user_configs
            .iter()
            .find(|c| c["provider"].as_str() == Some(p));
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

        let shared_cfg = shared_configs
            .iter()
            .find(|c| c["provider"].as_str() == Some(p));
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
        return Err(AppError::Validation(format!(
            "Unsupported provider '{provider}'"
        )));
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
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        seen_keys.insert(pkey.clone());
        let user_override = user_providers
            .iter()
            .find(|up| up["provider_key"].as_str() == Some(&pkey));

        let (source, is_override) = if let Some(up) = user_override {
            let key_exists = up["encrypted_api_key"]
                .as_str()
                .map(|k| !k.is_empty())
                .unwrap_or(false);
            if key_exists {
                ("user".to_string(), true)
            } else {
                let sys_has_key = sp["encrypted_api_key"]
                    .as_str()
                    .map(|k| !k.is_empty())
                    .unwrap_or(false);
                if sys_has_key {
                    ("system".to_string(), false)
                } else {
                    ("none".to_string(), false)
                }
            }
        } else {
            let sys_has_key = sp["encrypted_api_key"]
                .as_str()
                .map(|k| !k.is_empty())
                .unwrap_or(false);
            if sys_has_key {
                ("system".to_string(), false)
            } else {
                ("none".to_string(), false)
            }
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
            let user_override = user_providers
                .iter()
                .find(|up| up["provider_key"].as_str() == Some(def_key));
            let (source, is_override) = if let Some(up) = user_override {
                let key_exists = up["encrypted_api_key"]
                    .as_str()
                    .map(|k| !k.is_empty())
                    .unwrap_or(false);
                if key_exists {
                    ("user".to_string(), true)
                } else {
                    ("none".to_string(), false)
                }
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

fn resolve_user_provider_key(
    submitted_key: Option<&str>,
    existing_key: Option<&str>,
    crypto: &CryptoService,
) -> Result<Option<String>> {
    match submitted_key {
        Some(raw) if !raw.trim().is_empty() => Ok(Some(crypto.encrypt(raw.trim())?)),
        _ => Ok(existing_key.map(str::to_owned)),
    }
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
    let rec_id = format!("{}_{}", auth_user.user_id.0, pkey);

    let mut existing_result = db
        .query("SELECT encrypted_api_key, base_url FROM type::record('user_llm_provider', $id);")
        .bind(("id", rec_id.as_str()))
        .await
        .map_err(|e| AppError::Database(format!("Failed to read user provider: {e}")))?;
    let existing_records: Vec<serde_json::Value> = existing_result.take(0).unwrap_or_default();
    let existing_record = existing_records.first();
    let existing_key = existing_record.and_then(|record| record["encrypted_api_key"].as_str());
    let enc_key = resolve_user_provider_key(payload.api_key.as_deref(), existing_key, &crypto)?;

    let resolved_base_url = match payload.base_url {
        Some(url) if !url.trim().is_empty() => {
            let trimmed = url.trim();
            crate::shared::ssrf::resolve_and_verify_url(
                trimmed,
                false,
                &state.config.allowed_private_hosts,
            )
            .await?;
            Some(trimmed.to_string())
        }
        Some(_) => None,
        None => existing_record
            .and_then(|r| r.get("base_url"))
            .and_then(|v| v.as_str())
            .map(str::to_string),
    };

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
        .bind(("base_url", resolved_base_url))
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
    let slots_val = rows
        .into_iter()
        .next()
        .and_then(|r| r.get("slots").cloned())
        .unwrap_or(serde_json::json!({}));

    let slots: std::collections::HashMap<String, ModelRoutingConfigDto> =
        serde_json::from_value(slots_val).unwrap_or_default();
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
    let slots_val =
        serde_json::to_value(&payload.slots).map_err(|e| AppError::Validation(e.to_string()))?;

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

fn is_blocked_model_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            let octets = ip.octets();
            ip.is_private()
                || ip.is_loopback()
                || ip.is_link_local()
                || ip.is_unspecified()
                || ip.is_multicast()
                || ip.is_broadcast()
                || octets[0] == 0
                || (octets[0] == 100 && (64..=127).contains(&octets[1]))
                || (octets[0] == 192 && octets[1] == 0 && octets[2] == 0)
                || (octets[0] == 192 && octets[1] == 0 && octets[2] == 2)
                || (octets[0] == 198 && (octets[1] == 18 || octets[1] == 19))
                || (octets[0] == 198 && octets[1] == 51 && octets[2] == 100)
                || (octets[0] == 203 && octets[1] == 0 && octets[2] == 113)
                || octets[0] >= 240
        }
        IpAddr::V6(ip) => {
            let segments = ip.segments();
            ip.is_loopback()
                || ip.is_unspecified()
                || ip.is_unique_local()
                || ip.is_unicast_link_local()
                || ip.is_multicast()
                || (segments[0] & 0xffc0) == 0xfec0
                || (segments[0] == 0x2001 && segments[1] == 0x0db8)
                || ip
                    .to_ipv4_mapped()
                    .is_some_and(|mapped| is_blocked_model_ip(IpAddr::V4(mapped)))
        }
    }
}

async fn resolve_public_model_url(raw_url: &str) -> Result<(reqwest::Url, Vec<SocketAddr>)> {
    let mut url = reqwest::Url::parse(raw_url)
        .map_err(|_| AppError::Forbidden("Invalid model provider URL".into()))?;
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(AppError::Forbidden("Invalid model provider URL".into()));
    }

    let host = url
        .host_str()
        .ok_or_else(|| AppError::Forbidden("Invalid model provider URL".into()))?;
    let port = url
        .port_or_known_default()
        .ok_or_else(|| AppError::Forbidden("Invalid model provider URL".into()))?;
    let addresses = if let Ok(ip) = host.parse::<IpAddr>() {
        vec![SocketAddr::new(ip, port)]
    } else {
        tokio::net::lookup_host((host, port))
            .await
            .map_err(|_| AppError::Forbidden("Could not resolve model provider URL".into()))?
            .collect()
    };

    if addresses.is_empty() || addresses.iter().any(|addr| is_blocked_model_ip(addr.ip())) {
        return Err(AppError::Forbidden(
            "Private network URLs are forbidden".into(),
        ));
    }

    let path = format!("{}/models", url.path().trim_end_matches('/'));
    url.set_path(&path);
    Ok((url, addresses))
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
    let base_url =
        payload
            .base_url
            .unwrap_or_else(|| match payload.provider_key.to_lowercase().as_str() {
                "openai" => "https://api.openai.com/v1".to_string(),
                "openrouter" => "https://openrouter.ai/api/v1".to_string(),
                "anthropic" => "https://api.anthropic.com/v1".to_string(),
                _ => "https://api.openai.com/v1".to_string(),
            });
    let (models_url, addresses) = resolve_public_model_url(&base_url).await?;
    let host = models_url
        .host_str()
        .ok_or_else(|| AppError::Forbidden("Invalid model provider URL".into()))?;
    let client = reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .resolve_to_addrs(host, &addresses)
        .build()
        .map_err(|e| AppError::Internal(format!("Could not build model discovery client: {e}")))?;
    let mut req = client
        .get(models_url)
        .timeout(std::time::Duration::from_secs(5));
    if let Some(key) = payload.api_key.filter(|k| !k.is_empty()) {
        req = req.header("Authorization", format!("Bearer {key}"));
    }

    match req.send().await {
        Ok(res) if res.status().is_success() => {
            let val: serde_json::Value = res
                .json()
                .await
                .unwrap_or(serde_json::json!({ "data": [] }));
            Ok(Json(val))
        }
        _ => {
            let defaults = match payload.provider_key.to_lowercase().as_str() {
                "anthropic" => vec![
                    "claude-3-5-sonnet-20241022",
                    "claude-3-5-haiku-20241022",
                    "claude-3-opus-20240229",
                ],
                "openai" => vec!["gpt-4o", "gpt-4o-mini", "o1-mini", "o1-preview"],
                "gemini" => vec!["gemini-1.5-pro", "gemini-1.5-flash", "gemini-2.0-flash-exp"],
                _ => vec!["default-model"],
            };
            let data: Vec<serde_json::Value> = defaults
                .into_iter()
                .map(|m| serde_json::json!({ "id": m }))
                .collect();
            Ok(Json(serde_json::json!({ "data": data })))
        }
    }
}

#[cfg(test)]
mod model_discovery_security_tests {
    use super::is_blocked_model_ip;
    use std::net::IpAddr;

    #[test]
    fn rejects_private_and_link_local_addresses() {
        for address in [
            "127.0.0.1",
            "10.1.2.3",
            "192.168.1.2",
            "169.254.169.254",
            "100.64.0.1",
            "fc00::1",
            "fe80::1",
        ] {
            assert!(
                is_blocked_model_ip(address.parse::<IpAddr>().expect("valid fixture address")),
                "{address} must be rejected"
            );
        }
    }

    #[test]
    fn allows_public_addresses() {
        for address in ["1.1.1.1", "2606:4700:4700::1111"] {
            assert!(
                !is_blocked_model_ip(address.parse::<IpAddr>().expect("valid fixture address")),
                "{address} must be allowed"
            );
        }
    }

    #[tokio::test]
    async fn rejects_private_provider_url_before_connecting() {
        assert!(super::resolve_public_model_url("http://127.0.0.1/v1")
            .await
            .is_err());
        assert!(super::resolve_public_model_url("file:///etc/passwd")
            .await
            .is_err());
    }
}

#[cfg(test)]
mod provider_key_update_tests {
    use super::{resolve_user_provider_key, UpsertUserProviderRequestDto};
    use crate::shared::encryption::CryptoService;

    #[test]
    fn blank_or_omitted_key_preserves_existing_secret_and_replacement_updates_it() {
        let crypto = CryptoService::new("deterministic test encryption key");
        let existing = crypto.encrypt("existing test secret").unwrap();

        for body in [
            serde_json::json!({"provider_key": "openai"}),
            serde_json::json!({"provider_key": "openai", "api_key": ""}),
            serde_json::json!({"provider_key": "openai", "api_key": " \t\n "}),
        ] {
            let request: UpsertUserProviderRequestDto = serde_json::from_value(body).unwrap();
            let resolved =
                resolve_user_provider_key(request.api_key.as_deref(), Some(&existing), &crypto)
                    .unwrap();
            assert_eq!(resolved.as_deref(), Some(existing.as_str()));
        }

        let request: UpsertUserProviderRequestDto = serde_json::from_value(
            serde_json::json!({"provider_key": "openai", "api_key": "replacement test secret"}),
        )
        .unwrap();
        let replacement =
            resolve_user_provider_key(request.api_key.as_deref(), Some(&existing), &crypto)
                .unwrap()
                .unwrap();
        assert_ne!(replacement, existing);
        assert_eq!(
            crypto.decrypt(&replacement).unwrap(),
            "replacement test secret"
        );
    }
}
#[cfg(test)]
mod key_mask_tests {
    use super::mask_key;

    #[test]
    fn masks_unicode_keys_without_splitting_utf8() {
        assert_eq!(mask_key("sk-abcdef"), "sk-...cdef");
        assert_eq!(mask_key("sk-abcdef秘密"), "sk-...ef秘密");
    }
}
