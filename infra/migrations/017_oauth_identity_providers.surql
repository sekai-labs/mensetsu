USE NS mensetsu DB production;

-- OAuth / OIDC identity providers configured by administrators.
DEFINE TABLE IF NOT EXISTS oauth_provider SCHEMAFULL;
DEFINE FIELD OVERWRITE provider_key ON TABLE oauth_provider TYPE string;
DEFINE FIELD OVERWRITE name ON TABLE oauth_provider TYPE string;
DEFINE FIELD OVERWRITE provider_type ON TABLE oauth_provider TYPE string ASSERT $value IN ['google', 'github', 'facebook', 'oidc', 'oauth2'];
DEFINE FIELD OVERWRITE issuer ON TABLE oauth_provider TYPE option<string>;
DEFINE FIELD OVERWRITE client_id ON TABLE oauth_provider TYPE string;
DEFINE FIELD OVERWRITE encrypted_client_secret ON TABLE oauth_provider TYPE string;
DEFINE FIELD OVERWRITE redirect_uri ON TABLE oauth_provider TYPE string;
DEFINE FIELD OVERWRITE scopes ON TABLE oauth_provider TYPE array<string> DEFAULT [];
DEFINE FIELD OVERWRITE authorization_endpoint ON TABLE oauth_provider TYPE option<string>;
DEFINE FIELD OVERWRITE token_endpoint ON TABLE oauth_provider TYPE option<string>;
DEFINE FIELD OVERWRITE userinfo_endpoint ON TABLE oauth_provider TYPE option<string>;
DEFINE FIELD OVERWRITE email_endpoint ON TABLE oauth_provider TYPE option<string>;
DEFINE FIELD OVERWRITE jwks_uri ON TABLE oauth_provider TYPE option<string>;
DEFINE FIELD OVERWRITE subject_claim ON TABLE oauth_provider TYPE option<string>;
DEFINE FIELD OVERWRITE email_claim ON TABLE oauth_provider TYPE option<string>;
DEFINE FIELD OVERWRITE name_claim ON TABLE oauth_provider TYPE option<string>;
DEFINE FIELD OVERWRITE email_verified_claim ON TABLE oauth_provider TYPE option<string>;
DEFINE FIELD OVERWRITE is_enabled ON TABLE oauth_provider TYPE bool DEFAULT true;
DEFINE FIELD OVERWRITE created_at ON TABLE oauth_provider TYPE datetime DEFAULT time::now();
DEFINE FIELD OVERWRITE updated_at ON TABLE oauth_provider TYPE datetime DEFAULT time::now();
DEFINE INDEX IF NOT EXISTS idx_oauth_provider_key ON TABLE oauth_provider COLUMNS provider_key UNIQUE;

-- User accounts are linked only by stable provider subject, never by email alone.
DEFINE TABLE IF NOT EXISTS user_oauth_account SCHEMAFULL;
DEFINE FIELD OVERWRITE user_id ON TABLE user_oauth_account TYPE record<user>;
DEFINE FIELD OVERWRITE provider_key ON TABLE user_oauth_account TYPE string;
DEFINE FIELD OVERWRITE provider_user_id ON TABLE user_oauth_account TYPE string;
DEFINE FIELD OVERWRITE email ON TABLE user_oauth_account TYPE option<string>;
DEFINE FIELD OVERWRITE email_verified ON TABLE user_oauth_account TYPE bool DEFAULT false;
DEFINE FIELD OVERWRITE display_name ON TABLE user_oauth_account TYPE option<string>;
DEFINE FIELD OVERWRITE created_at ON TABLE user_oauth_account TYPE datetime DEFAULT time::now();
DEFINE FIELD OVERWRITE last_login_at ON TABLE user_oauth_account TYPE datetime DEFAULT time::now();
DEFINE INDEX IF NOT EXISTS idx_user_oauth_provider_uid ON TABLE user_oauth_account COLUMNS provider_key, provider_user_id UNIQUE;
DEFINE INDEX IF NOT EXISTS idx_user_oauth_user ON TABLE user_oauth_account COLUMNS user_id;

-- Short-lived OAuth authorization state, PKCE verifier is encrypted at rest.
DEFINE TABLE IF NOT EXISTS oauth_state SCHEMAFULL;
DEFINE FIELD OVERWRITE state ON TABLE oauth_state TYPE string;
DEFINE FIELD OVERWRITE provider_key ON TABLE oauth_state TYPE string;
DEFINE FIELD OVERWRITE purpose ON TABLE oauth_state TYPE string ASSERT $value IN ['login', 'link'];
DEFINE FIELD OVERWRITE code_verifier ON TABLE oauth_state TYPE string;
DEFINE FIELD OVERWRITE nonce ON TABLE oauth_state TYPE option<string>;
DEFINE FIELD OVERWRITE redirect_uri ON TABLE oauth_state TYPE string;
DEFINE FIELD OVERWRITE link_user_id ON TABLE oauth_state TYPE option<record<user>>;
DEFINE FIELD OVERWRITE expires_at ON TABLE oauth_state TYPE datetime;
DEFINE FIELD OVERWRITE used ON TABLE oauth_state TYPE bool DEFAULT false;
DEFINE FIELD OVERWRITE created_at ON TABLE oauth_state TYPE datetime DEFAULT time::now();
DEFINE INDEX IF NOT EXISTS idx_oauth_state_val ON TABLE oauth_state COLUMNS state UNIQUE;
DEFINE INDEX IF NOT EXISTS idx_oauth_state_expires ON TABLE oauth_state COLUMNS expires_at;

-- Short-lived one-use authentication handoff; only encrypted payload is stored.
DEFINE TABLE IF NOT EXISTS auth_handoff SCHEMAFULL;
DEFINE FIELD OVERWRITE code ON TABLE auth_handoff TYPE string;
DEFINE FIELD OVERWRITE user_id ON TABLE auth_handoff TYPE record<user>;
DEFINE FIELD OVERWRITE encrypted_payload ON TABLE auth_handoff TYPE string;
DEFINE FIELD OVERWRITE expires_at ON TABLE auth_handoff TYPE datetime;
DEFINE FIELD OVERWRITE used ON TABLE auth_handoff TYPE bool DEFAULT false;
DEFINE FIELD OVERWRITE created_at ON TABLE auth_handoff TYPE datetime DEFAULT time::now();
DEFINE INDEX IF NOT EXISTS idx_auth_handoff_code ON TABLE auth_handoff COLUMNS code UNIQUE;
DEFINE INDEX IF NOT EXISTS idx_auth_handoff_expires ON TABLE auth_handoff COLUMNS expires_at;
