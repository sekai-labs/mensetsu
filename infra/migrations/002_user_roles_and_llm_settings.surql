
USE NS mensetsu DB production;

DEFINE FIELD OVERWRITE role ON TABLE user TYPE string ASSERT $value IN ['admin', 'member'] DEFAULT 'member';
DEFINE FIELD OVERWRITE totp_enabled ON TABLE user TYPE bool DEFAULT false;
DEFINE FIELD OVERWRITE totp_secret ON TABLE user TYPE option<string>;

UPDATE user SET role = 'member' WHERE role IS NONE;
UPDATE user SET totp_enabled = false WHERE totp_enabled IS NONE;

DEFINE TABLE IF NOT EXISTS user_llm_config SCHEMAFULL;
DEFINE FIELD OVERWRITE user_id ON TABLE user_llm_config TYPE record<user>;
DEFINE FIELD OVERWRITE provider ON TABLE user_llm_config TYPE string ASSERT $value IN ['openai', 'anthropic', 'openrouter', 'ollama', 'custom'];
DEFINE FIELD OVERWRITE encrypted_api_key ON TABLE user_llm_config TYPE option<string>;
DEFINE FIELD OVERWRITE base_url ON TABLE user_llm_config TYPE option<string>;
DEFINE FIELD OVERWRITE model_chat ON TABLE user_llm_config TYPE option<string>;
DEFINE FIELD OVERWRITE model_search ON TABLE user_llm_config TYPE option<string>;
DEFINE FIELD OVERWRITE model_embed ON TABLE user_llm_config TYPE option<string>;
DEFINE FIELD OVERWRITE is_shared ON TABLE user_llm_config TYPE bool DEFAULT false;
DEFINE FIELD OVERWRITE created_at ON TABLE user_llm_config TYPE datetime DEFAULT time::now();
DEFINE FIELD OVERWRITE updated_at ON TABLE user_llm_config TYPE datetime DEFAULT time::now();
DEFINE INDEX IF NOT EXISTS idx_user_provider ON TABLE user_llm_config COLUMNS user_id, provider UNIQUE;

DEFINE FIELD OVERWRITE video_url ON TABLE learning_unit TYPE option<string>;
DEFINE FIELD OVERWRITE video_title ON TABLE learning_unit TYPE option<string>;
DEFINE FIELD OVERWRITE chapters ON TABLE learning_unit TYPE option<array<object>>;
DEFINE FIELD OVERWRITE completed ON TABLE learning_unit TYPE bool DEFAULT false;

DEFINE TABLE IF NOT EXISTS company_research SCHEMAFULL;
DEFINE FIELD OVERWRITE workspace_id ON TABLE company_research TYPE record<workspace>;
DEFINE FIELD OVERWRITE company_name ON TABLE company_research TYPE string;
DEFINE FIELD OVERWRITE dossier ON TABLE company_research TYPE object;
DEFINE FIELD OVERWRITE created_at ON TABLE company_research TYPE datetime DEFAULT time::now();
DEFINE FIELD OVERWRITE updated_at ON TABLE company_research TYPE datetime DEFAULT time::now();
DEFINE INDEX IF NOT EXISTS idx_company_research_workspace ON TABLE company_research COLUMNS workspace_id UNIQUE;
