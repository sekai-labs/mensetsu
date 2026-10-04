
USE NS mensetsu DB production;

DEFINE FIELD role ON TABLE user TYPE string ASSERT $value IN ['admin', 'member'] DEFAULT 'member';
DEFINE FIELD totp_enabled ON TABLE user TYPE bool DEFAULT false;
DEFINE FIELD totp_secret ON TABLE user TYPE option<string>;

DEFINE TABLE user_llm_config SCHEMAFULL;
DEFINE FIELD user_id ON TABLE user_llm_config TYPE record<user>;
DEFINE FIELD provider ON TABLE user_llm_config TYPE string ASSERT $value IN ['openai', 'anthropic', 'openrouter', 'ollama', 'custom'];
DEFINE FIELD encrypted_api_key ON TABLE user_llm_config TYPE option<string>;
DEFINE FIELD base_url ON TABLE user_llm_config TYPE option<string>;
DEFINE FIELD model_chat ON TABLE user_llm_config TYPE option<string>;
DEFINE FIELD model_search ON TABLE user_llm_config TYPE option<string>;
DEFINE FIELD model_embed ON TABLE user_llm_config TYPE option<string>;
DEFINE FIELD is_shared ON TABLE user_llm_config TYPE bool DEFAULT false;
DEFINE FIELD created_at ON TABLE user_llm_config TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at ON TABLE user_llm_config TYPE datetime DEFAULT time::now();
DEFINE INDEX idx_user_provider ON TABLE user_llm_config COLUMNS user_id, provider UNIQUE;

DEFINE FIELD video_url ON TABLE learning_unit TYPE option<string>;
DEFINE FIELD video_title ON TABLE learning_unit TYPE option<string>;
DEFINE FIELD chapters ON TABLE learning_unit TYPE option<array<object>>;
DEFINE FIELD completed ON TABLE learning_unit TYPE bool DEFAULT false;

DEFINE TABLE company_research SCHEMAFULL;
DEFINE FIELD workspace_id ON TABLE company_research TYPE record<workspace>;
DEFINE FIELD company_name ON TABLE company_research TYPE string;
DEFINE FIELD dossier ON TABLE company_research TYPE object;
DEFINE FIELD created_at ON TABLE company_research TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at ON TABLE company_research TYPE datetime DEFAULT time::now();
DEFINE INDEX idx_company_research_workspace ON TABLE company_research COLUMNS workspace_id UNIQUE;
