
DEFINE NAMESPACE IF NOT EXISTS mensetsu;
DEFINE DATABASE IF NOT EXISTS production;
USE NS mensetsu DB production;

DEFINE TABLE user SCHEMAFULL;
DEFINE FIELD email ON TABLE user TYPE string ASSERT string::is_email($value);
DEFINE FIELD name ON TABLE user TYPE string;
DEFINE FIELD password_hash ON TABLE user TYPE string;
DEFINE FIELD status ON TABLE user TYPE option<string>;
DEFINE FIELD role ON TABLE user TYPE string ASSERT $value IN ['admin', 'member'] DEFAULT 'member';
DEFINE FIELD totp_enabled ON TABLE user TYPE bool DEFAULT false;
DEFINE FIELD totp_secret ON TABLE user TYPE option<string>;
DEFINE FIELD created_at ON TABLE user TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at ON TABLE user TYPE datetime DEFAULT time::now();
DEFINE INDEX idx_user_email ON TABLE user COLUMNS email UNIQUE;

DEFINE TABLE auth_session SCHEMAFULL;
DEFINE FIELD user_id ON TABLE auth_session TYPE record<user>;
DEFINE FIELD refresh_token_hash ON TABLE auth_session TYPE string;
DEFINE FIELD user_agent ON TABLE auth_session TYPE option<string>;
DEFINE FIELD ip_address ON TABLE auth_session TYPE option<string>;
DEFINE FIELD expires_at ON TABLE auth_session TYPE datetime;
DEFINE FIELD created_at ON TABLE auth_session TYPE datetime DEFAULT time::now();
DEFINE FIELD last_used_at ON TABLE auth_session TYPE datetime DEFAULT time::now();
DEFINE FIELD revoked ON TABLE auth_session TYPE bool DEFAULT false;
DEFINE FIELD updated_at ON TABLE auth_session TYPE option<datetime>;
DEFINE INDEX idx_session_hash ON TABLE auth_session COLUMNS refresh_token_hash;
DEFINE INDEX idx_session_user ON TABLE auth_session COLUMNS user_id;

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
DEFINE TABLE workspace SCHEMAFULL;
DEFINE FIELD user_id ON TABLE workspace TYPE record<user>;
DEFINE FIELD company_name ON TABLE workspace TYPE string;
DEFINE FIELD role_title ON TABLE workspace TYPE string;
DEFINE FIELD status ON TABLE workspace TYPE string ASSERT $value IN ['pending', 'processing', 'ready', 'failed'];
DEFINE FIELD current_phase ON TABLE workspace TYPE string;
DEFINE FIELD progress_percent ON TABLE workspace TYPE number ASSERT $value >= 0 AND $value <= 100;
DEFINE FIELD seniority ON TABLE workspace TYPE option<string>;
DEFINE FIELD target_date ON TABLE workspace TYPE option<datetime>;
DEFINE FIELD prep_time_weeks ON TABLE workspace TYPE option<int>;
DEFINE FIELD created_at ON TABLE workspace TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at ON TABLE workspace TYPE datetime DEFAULT time::now();
DEFINE INDEX idx_workspace_user ON TABLE workspace COLUMNS user_id;

DEFINE TABLE resume SCHEMAFULL;
DEFINE FIELD workspace_id ON TABLE resume TYPE record<workspace>;
DEFINE FIELD candidate_name ON TABLE resume TYPE option<string>;
DEFINE FIELD summary ON TABLE resume TYPE option<string>;
DEFINE FIELD raw_text ON TABLE resume TYPE string;
DEFINE FIELD parsed_skills ON TABLE resume TYPE array<string>;
DEFINE FIELD experience ON TABLE resume TYPE array<object>;
DEFINE FIELD education ON TABLE resume TYPE array<object>;
DEFINE FIELD created_at ON TABLE resume TYPE datetime DEFAULT time::now();
DEFINE INDEX idx_resume_workspace ON TABLE resume COLUMNS workspace_id UNIQUE;

DEFINE TABLE job_description SCHEMAFULL;
DEFINE FIELD workspace_id ON TABLE job_description TYPE record<workspace>;
DEFINE FIELD raw_text ON TABLE job_description TYPE string;
DEFINE FIELD requirements ON TABLE job_description TYPE array<object>;
DEFINE FIELD created_at ON TABLE job_description TYPE datetime DEFAULT time::now();
DEFINE INDEX idx_jd_workspace ON TABLE job_description COLUMNS workspace_id UNIQUE;

DEFINE TABLE match_analysis SCHEMAFULL;
DEFINE FIELD workspace_id ON TABLE match_analysis TYPE record<workspace>;
DEFINE FIELD overall_match_ratio ON TABLE match_analysis TYPE number;
DEFINE FIELD strengths ON TABLE match_analysis TYPE array<object>;
DEFINE FIELD gaps ON TABLE match_analysis TYPE array<object>;
DEFINE FIELD risks ON TABLE match_analysis TYPE array<string>;
DEFINE FIELD wording_improvements ON TABLE match_analysis TYPE array<object>;
DEFINE FIELD created_at ON TABLE match_analysis TYPE datetime DEFAULT time::now();
DEFINE INDEX idx_match_workspace ON TABLE match_analysis COLUMNS workspace_id UNIQUE;

DEFINE TABLE source SCHEMAFULL;
DEFINE FIELD workspace_id ON TABLE source TYPE record<workspace>;
DEFINE FIELD url ON TABLE source TYPE string;
DEFINE FIELD title ON TABLE source TYPE string;
DEFINE FIELD domain ON TABLE source TYPE string;
DEFINE FIELD snippet ON TABLE source TYPE string;
DEFINE FIELD source_type ON TABLE source TYPE string;
DEFINE FIELD confidence ON TABLE source TYPE number;
DEFINE FIELD retrieved_at ON TABLE source TYPE datetime DEFAULT time::now();
DEFINE INDEX idx_source_workspace ON TABLE source COLUMNS workspace_id;

DEFINE TABLE claim SCHEMAFULL;
DEFINE FIELD workspace_id ON TABLE claim TYPE record<workspace>;
DEFINE FIELD statement ON TABLE claim TYPE string;
DEFINE FIELD claim_type ON TABLE claim TYPE string;
DEFINE FIELD confidence ON TABLE claim TYPE number;
DEFINE FIELD source_id ON TABLE claim TYPE record<source>;
DEFINE FIELD created_at ON TABLE claim TYPE datetime DEFAULT time::now();
DEFINE INDEX idx_claim_workspace ON TABLE claim COLUMNS workspace_id;

DEFINE TABLE skill SCHEMAFULL;
DEFINE FIELD workspace_id ON TABLE skill TYPE option<record<workspace>>;
DEFINE FIELD name ON TABLE skill TYPE string;
DEFINE FIELD slug ON TABLE skill TYPE string;
DEFINE FIELD category ON TABLE skill TYPE string;
DEFINE FIELD candidate_status ON TABLE skill TYPE option<string>;
DEFINE FIELD importance ON TABLE skill TYPE option<string>;
DEFINE FIELD description ON TABLE skill TYPE option<string>;
DEFINE INDEX idx_skill_ws ON TABLE skill COLUMNS workspace_id;

DEFINE TABLE skill_relation SCHEMAFULL;
DEFINE FIELD workspace_id ON TABLE skill_relation TYPE option<record<workspace>>;
DEFINE FIELD source ON TABLE skill_relation TYPE string;
DEFINE FIELD target ON TABLE skill_relation TYPE string;
DEFINE FIELD relation ON TABLE skill_relation TYPE string;
DEFINE INDEX idx_skill_rel_ws ON TABLE skill_relation COLUMNS workspace_id;
DEFINE TABLE concept SCHEMAFULL;
DEFINE FIELD name ON TABLE concept TYPE string;
DEFINE FIELD description ON TABLE concept TYPE string;

DEFINE TABLE company_entity SCHEMAFULL;
DEFINE FIELD name ON TABLE company_entity TYPE string;
DEFINE FIELD domain ON TABLE company_entity TYPE option<string>;
DEFINE FIELD overview ON TABLE company_entity TYPE string;
DEFINE FIELD business_model ON TABLE company_entity TYPE string;

DEFINE TABLE rel_requires TYPE RELATION IN role_entity OUT skill SCHEMAFULL;
DEFINE FIELD importance ON TABLE rel_requires TYPE string ASSERT $value IN ['mandatory', 'preferred'];

DEFINE TABLE rel_has_skill TYPE RELATION IN resume OUT skill SCHEMAFULL;
DEFINE FIELD confidence ON TABLE rel_has_skill TYPE number;
DEFINE FIELD evidence_quote ON TABLE rel_has_skill TYPE string;

DEFINE TABLE rel_depends_on TYPE RELATION IN skill OUT skill SCHEMAFULL;
DEFINE FIELD order ON TABLE rel_depends_on TYPE int;

DEFINE TABLE rel_supports TYPE RELATION IN source OUT claim SCHEMAFULL;
DEFINE FIELD quote ON TABLE rel_supports TYPE string;

DEFINE TABLE rel_describes TYPE RELATION IN claim OUT company_entity SCHEMAFULL;

DEFINE TABLE learning_unit SCHEMAFULL;
DEFINE FIELD workspace_id ON TABLE learning_unit TYPE record<workspace>;
DEFINE FIELD skill ON TABLE learning_unit TYPE string;
DEFINE FIELD title ON TABLE learning_unit TYPE string;
DEFINE FIELD prerequisites ON TABLE learning_unit TYPE array<string>;
DEFINE FIELD estimated_minutes ON TABLE learning_unit TYPE int;
DEFINE FIELD summary ON TABLE learning_unit TYPE string;
DEFINE FIELD content_markdown ON TABLE learning_unit TYPE string;
DEFINE FIELD diagram ON TABLE learning_unit TYPE option<object>;
DEFINE FIELD video_url ON TABLE learning_unit TYPE option<string>;
DEFINE FIELD video_title ON TABLE learning_unit TYPE option<string>;
DEFINE FIELD chapters ON TABLE learning_unit TYPE option<array<object>>;
DEFINE FIELD completed ON TABLE learning_unit TYPE bool DEFAULT false;
DEFINE FIELD created_at ON TABLE learning_unit TYPE datetime DEFAULT time::now();
DEFINE INDEX idx_learning_workspace ON TABLE learning_unit COLUMNS workspace_id;

DEFINE TABLE interview_question SCHEMAFULL;
DEFINE FIELD workspace_id ON TABLE interview_question TYPE record<workspace>;
DEFINE FIELD category ON TABLE interview_question TYPE string;
DEFINE FIELD question ON TABLE interview_question TYPE string;
DEFINE FIELD difficulty ON TABLE interview_question TYPE string ASSERT $value IN ['easy', 'medium', 'hard'];
DEFINE FIELD skills_tested ON TABLE interview_question TYPE array<string>;
DEFINE FIELD interviewer_intent ON TABLE interview_question TYPE string;
DEFINE FIELD answer_outline ON TABLE interview_question TYPE array<string>;
DEFINE FIELD strong_answer_rubric ON TABLE interview_question TYPE array<string>;
DEFINE FIELD common_weaknesses ON TABLE interview_question TYPE array<string>;
DEFINE FIELD follow_up_questions ON TABLE interview_question TYPE array<string>;
DEFINE FIELD created_at ON TABLE interview_question TYPE datetime DEFAULT time::now();
DEFINE INDEX idx_question_workspace ON TABLE interview_question COLUMNS workspace_id;

DEFINE TABLE company_research SCHEMAFULL;
DEFINE FIELD workspace_id ON TABLE company_research TYPE record<workspace>;
DEFINE FIELD company_name ON TABLE company_research TYPE string;
DEFINE FIELD dossier ON TABLE company_research TYPE object;
DEFINE FIELD created_at ON TABLE company_research TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at ON TABLE company_research TYPE datetime DEFAULT time::now();
DEFINE INDEX idx_company_research_workspace ON TABLE company_research COLUMNS workspace_id UNIQUE;

DEFINE TABLE pipeline_execution SCHEMAFULL;
DEFINE FIELD workspace_id ON TABLE pipeline_execution TYPE record<workspace>;
DEFINE FIELD status ON TABLE pipeline_execution TYPE string;
DEFINE FIELD current_step ON TABLE pipeline_execution TYPE string;
DEFINE FIELD total_steps ON TABLE pipeline_execution TYPE int DEFAULT 8;
DEFINE FIELD completed_steps ON TABLE pipeline_execution TYPE int DEFAULT 0;
DEFINE FIELD error_message ON TABLE pipeline_execution TYPE option<string>;
DEFINE FIELD started_at ON TABLE pipeline_execution TYPE datetime DEFAULT time::now();
DEFINE FIELD completed_at ON TABLE pipeline_execution TYPE option<datetime>;
DEFINE INDEX idx_pipeline_exec_ws ON TABLE pipeline_execution COLUMNS workspace_id;

DEFINE TABLE pipeline_log SCHEMAFULL;
DEFINE FIELD workspace_id ON TABLE pipeline_log TYPE record<workspace>;
DEFINE FIELD execution_id ON TABLE pipeline_log TYPE string;
DEFINE FIELD step ON TABLE pipeline_log TYPE string;
DEFINE FIELD level ON TABLE pipeline_log TYPE string ASSERT $value IN ['info', 'warn', 'error', 'debug'];
DEFINE FIELD message ON TABLE pipeline_log TYPE string;
DEFINE FIELD details ON TABLE pipeline_log TYPE option<object>;
DEFINE FIELD timestamp ON TABLE pipeline_log TYPE datetime DEFAULT time::now();
DEFINE INDEX idx_pipeline_log_ws ON TABLE pipeline_log COLUMNS workspace_id, timestamp;
