import os
from typing import Any
from langchain_core.language_models.chat_models import BaseChatModel
from langchain_openai import ChatOpenAI

ROLE_SLOT_MAP = {
    "router": "router",
    "workflow-router": "router",
    "planner": "planner",
    "company-research": "research",
    "technical-research": "research",
    "job-analysis": "research",
    "systems-deep-dive": "research",
    "evidence-evaluator": "generation",
    "curriculum": "generation",
    "interview-generation": "generation",
}

def resolve_provider_credentials(
    provider_key: str,
    user_providers: dict[str, Any],
    system_providers: dict[str, Any],
    legacy_cfg: dict[str, Any] | None,
) -> tuple[str, str | None]:
    """
    Resolves (api_key, base_url) with BYOK precedence:
    1. User's personal key for provider_key if present and non-empty.
    2. Admin platform key for provider_key if present and non-empty.
    3. Legacy config if matches provider_key or as fallback.
    4. Environment variable fallback (OPENAI_API_KEY, etc).
    """
    norm_key = provider_key.lower()

    if norm_key in user_providers:
        u_cfg = user_providers[norm_key]
        u_key = u_cfg.get("api_key")
        if u_key and u_key.strip():
            return u_key.strip(), u_cfg.get("base_url")

    if norm_key in system_providers:
        s_cfg = system_providers[norm_key]
        s_key = s_cfg.get("api_key")
        if s_key and s_key.strip():
            return s_key.strip(), s_cfg.get("base_url")

    if legacy_cfg:
        leg_key = legacy_cfg.get("api_key")
        if leg_key and leg_key.strip():
            return leg_key.strip(), legacy_cfg.get("base_url")

    env_key = os.environ.get("OPENAI_API_KEY", "")
    env_base = os.environ.get("OPENAI_BASE_URL")
    return env_key, env_base


def create_model_instance(
    provider_key: str,
    model: str,
    temperature: float,
    api_key: str,
    base_url: str | None,
    timeout_s: float,
) -> BaseChatModel:
    norm_provider = provider_key.lower()
    norm_base = base_url or os.environ.get("OPENAI_BASE_URL")
    if not norm_base:
        if norm_provider == "openrouter":
            norm_base = "https://openrouter.ai/api/v1"
        elif norm_provider == "ollama":
            norm_base = "http://localhost:11434/v1"

    return ChatOpenAI(
        model=model,
        api_key=api_key or "sk-dummy-key",
        base_url=norm_base,
        temperature=temperature,
        timeout=timeout_s,
        max_retries=0,
    )


def resolve_chat_model_for_role(
    role: str,
    llm_config_payload: dict[str, Any] | None,
    timeout_s: float = 90.0,
) -> BaseChatModel:
    """
    Decoupled model resolution with fallback cascade per functional role:
    router, planner, research, generation.
    Binds primary.with_fallbacks([fallback_model]) if fallback is defined.
    """
    cfg = llm_config_payload or {}
    user_providers = cfg.get("user_providers") or {}
    system_providers = cfg.get("system_providers") or {}
    routing = cfg.get("model_routing") or {}
    legacy = cfg.get("legacy")

    slot_name = ROLE_SLOT_MAP.get(role, role)
    slot_config = routing.get(slot_name) or routing.get(role)

    primary_def = None
    fallback_def = None

    if slot_config and isinstance(slot_config, dict):
        primary_def = slot_config.get("primary")
        fallback_def = slot_config.get("fallback")

    if primary_def:
        p_provider = primary_def.get("provider_key", "openai")
        p_model = primary_def.get("model") or os.environ.get("OPENAI_MODEL", "gpt-4o")
        p_temp = float(primary_def.get("temperature", 0.2))
    else:
        p_provider = (legacy or {}).get("provider") or "openai"
        p_model = (legacy or {}).get("model_chat") or os.environ.get("OPENAI_MODEL", "gpt-4o")
        p_temp = 0.2

    p_key, p_base = resolve_provider_credentials(p_provider, user_providers, system_providers, legacy)
    primary_model = create_model_instance(p_provider, p_model, p_temp, p_key, p_base, timeout_s)

    if fallback_def and isinstance(fallback_def, dict) and fallback_def.get("model"):
        f_provider = fallback_def.get("provider_key", "openai")
        f_model = fallback_def.get("model")
        f_temp = float(fallback_def.get("temperature", 0.2))
        f_key, f_base = resolve_provider_credentials(f_provider, user_providers, system_providers, legacy)
        fallback_model = create_model_instance(f_provider, f_model, f_temp, f_key, f_base, timeout_s)
        return primary_model.with_fallbacks([fallback_model])

    return primary_model
