USE NS mensetsu DB production;

-- 1. Workspace schema extensions
DEFINE FIELD OVERWRITE user_prompt ON TABLE workspace TYPE option<string>;
DEFINE FIELD OVERWRITE current_objective ON TABLE workspace TYPE option<string>;
DEFINE FIELD OVERWRITE workflow_mode ON TABLE workspace TYPE string DEFAULT 'hybrid' ASSERT $value IN ['hybrid', 'static', 'dynamic'];

-- 2. System LLM Provider (Admin managed)
DEFINE TABLE OVERWRITE system_llm_provider SCHEMAFULL;
DEFINE FIELD OVERWRITE provider_key ON TABLE system_llm_provider TYPE string;
DEFINE FIELD OVERWRITE name ON TABLE system_llm_provider TYPE string;
DEFINE FIELD OVERWRITE base_url ON TABLE system_llm_provider TYPE option<string>;
DEFINE FIELD OVERWRITE encrypted_api_key ON TABLE system_llm_provider TYPE option<string>;
DEFINE FIELD OVERWRITE is_enabled ON TABLE system_llm_provider TYPE bool DEFAULT true;
DEFINE FIELD OVERWRITE allowed_models ON TABLE system_llm_provider TYPE array<string> DEFAULT [];
DEFINE FIELD OVERWRITE created_at ON TABLE system_llm_provider TYPE datetime DEFAULT time::now();
DEFINE FIELD OVERWRITE updated_at ON TABLE system_llm_provider TYPE datetime DEFAULT time::now();
DEFINE INDEX IF NOT EXISTS idx_system_provider_key ON TABLE system_llm_provider COLUMNS provider_key UNIQUE;

-- 3. User LLM Provider (BYOK Precedence)
DEFINE TABLE OVERWRITE user_llm_provider SCHEMAFULL;
DEFINE FIELD OVERWRITE user_id ON TABLE user_llm_provider TYPE record<user>;
DEFINE FIELD OVERWRITE provider_key ON TABLE user_llm_provider TYPE string;
DEFINE FIELD OVERWRITE encrypted_api_key ON TABLE user_llm_provider TYPE option<string>;
DEFINE FIELD OVERWRITE base_url ON TABLE user_llm_provider TYPE option<string>;
DEFINE FIELD OVERWRITE created_at ON TABLE user_llm_provider TYPE datetime DEFAULT time::now();
DEFINE FIELD OVERWRITE updated_at ON TABLE user_llm_provider TYPE datetime DEFAULT time::now();
DEFINE INDEX IF NOT EXISTS idx_user_provider_override ON TABLE user_llm_provider COLUMNS user_id, provider_key UNIQUE;

-- 4. User Model Routing Matrix
DEFINE TABLE OVERWRITE user_model_routing SCHEMAFULL;
DEFINE FIELD OVERWRITE user_id ON TABLE user_model_routing TYPE record<user>;
DEFINE FIELD OVERWRITE slots ON TABLE user_model_routing TYPE object FLEXIBLE DEFAULT {};
DEFINE FIELD OVERWRITE created_at ON TABLE user_model_routing TYPE datetime DEFAULT time::now();
DEFINE FIELD OVERWRITE updated_at ON TABLE user_model_routing TYPE datetime DEFAULT time::now();
DEFINE INDEX IF NOT EXISTS idx_user_routing ON TABLE user_model_routing COLUMNS user_id UNIQUE;

-- 5. System Configuration
DEFINE TABLE OVERWRITE system_config SCHEMAFULL;
DEFINE FIELD OVERWRITE key ON TABLE system_config TYPE string;
DEFINE FIELD OVERWRITE value ON TABLE system_config TYPE any;
DEFINE FIELD OVERWRITE is_secret ON TABLE system_config TYPE bool DEFAULT false;
DEFINE FIELD OVERWRITE updated_by ON TABLE system_config TYPE option<record<user>>;
DEFINE FIELD OVERWRITE updated_at ON TABLE system_config TYPE datetime DEFAULT time::now();
DEFINE INDEX IF NOT EXISTS idx_system_config_key ON TABLE system_config COLUMNS key UNIQUE;

-- 6. Artifact Type extension
DEFINE FIELD OVERWRITE type ON TABLE artifact TYPE string ASSERT $value IN ['research_report', 'finding', 'evidence', 'question_set', 'recommendation', 'validation_result', 'routing_decision'];

-- 7. Data Cutover from legacy user_llm_config
FOR $cfg IN (SELECT * FROM user_llm_config WHERE is_shared = true) {
    UPSERT type::record('system_llm_provider', $cfg.provider) SET
        provider_key = $cfg.provider,
        name = string::concat(string::uppercase(string::slice($cfg.provider, 0, 1)), string::slice($cfg.provider, 1, 50)),
        base_url = $cfg.base_url,
        encrypted_api_key = $cfg.encrypted_api_key,
        is_enabled = true,
        allowed_models = [];
};

FOR $cfg IN (SELECT * FROM user_llm_config WHERE is_shared = false AND encrypted_api_key != NONE AND encrypted_api_key != "") {
    UPSERT type::record('user_llm_provider', string::concat(type::string($cfg.user_id), '_', $cfg.provider)) SET
        user_id = $cfg.user_id,
        provider_key = $cfg.provider,
        base_url = $cfg.base_url,
        encrypted_api_key = $cfg.encrypted_api_key;
};

-- Initialize default system config values if absent
UPSERT type::record('system_config', 'MAX_CONCURRENT_AGENTS') SET
    key = 'MAX_CONCURRENT_AGENTS',
    value = 5,
    is_secret = false,
    updated_at = time::now();

UPSERT type::record('system_config', 'MAX_CONCURRENT_AGENTS_PER_RUN') SET
    key = 'MAX_CONCURRENT_AGENTS_PER_RUN',
    value = 3,
    is_secret = false,
    updated_at = time::now();

UPSERT type::record('system_config', 'LLM_MAX_INFLIGHT') SET
    key = 'LLM_MAX_INFLIGHT',
    value = 10,
    is_secret = false,
    updated_at = time::now();

UPSERT type::record('system_config', 'ACTIVE_SEARCH_PROVIDER') SET
    key = 'ACTIVE_SEARCH_PROVIDER',
    value = 'searxng',
    is_secret = false,
    updated_at = time::now();
