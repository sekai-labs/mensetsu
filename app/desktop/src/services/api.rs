use std::{path::Path, sync::Arc};
use tokio::sync::{Mutex, RwLock};
use mensetsu_contracts::apis::configuration::Configuration;
use mensetsu_contracts::apis::{auth_api, resume_uploads_api, workspaces_api, workspaces_learning_handlers_api, workspaces_research_handlers_api, settings_api};
use mensetsu_contracts::models;
use crate::services::secure_store::KeyringStore;

#[derive(Clone)]
pub struct ApiService {
    base_url: Arc<std::sync::RwLock<String>>,
    // Access token kept in memory only
    access_token: Arc<RwLock<Option<String>>>,
    // Refresh token serialized lock and persistent store
    refresh_lock: Arc<Mutex<()>>,
    keyring: Arc<KeyringStore>,
}

impl PartialEq for ApiService {
    fn eq(&self, other: &Self) -> bool {
        let self_url = self.base_url.read().map(|b| b.clone()).unwrap_or_default();
        let other_url = other.base_url.read().map(|b| b.clone()).unwrap_or_default();
        self_url == other_url
    }
}

#[derive(Debug, thiserror::Error, Clone)]
pub enum ApiError {
    #[error("Authentication failed: {0}")]
    Auth(String),
    #[error("API error: {0}")]
    Api(String),
    #[error("Network error: {0}")]
    Network(String),
    #[error("Requires 2FA")]
    Requires2Fa { temp_token: String },
    #[error("Not authenticated")]
    NotAuthenticated,
    #[error("Secure storage error: {0}")]
    Keyring(String),
}

impl ApiService {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url: Arc::new(std::sync::RwLock::new(base_url)),
            access_token: Arc::new(RwLock::new(None)),
            refresh_lock: Arc::new(Mutex::new(())),
            keyring: Arc::new(KeyringStore::new()),
        }
    }

    pub fn get_base_url(&self) -> String {
        self.base_url.read().unwrap().clone()
    }

    pub async fn set_base_url(&self, new_url: String) -> Result<(), ApiError> {
        let _refresh_guard = self.refresh_lock.lock().await;
        self.clear_tokens().await?;
        let mut guard = self.base_url.write().unwrap();
        *guard = new_url;
        Ok(())
    }

    pub async fn is_logged_in(&self) -> bool {
        let guard = self.access_token.read().await;
        if guard.is_some() {
            return true;
        }
        drop(guard);
        self.keyring.get_refresh_token().ok().flatten().is_some()
    }

    pub async fn set_tokens(&self, access_token: String, refresh_token: String) -> Result<(), ApiError> {
        self.keyring.set_refresh_token(&refresh_token).map_err(|e| ApiError::Keyring(e.to_string()))?;
        let mut guard = self.access_token.write().await;
        *guard = Some(access_token);
        Ok(())
    }

    pub async fn clear_tokens(&self) -> Result<(), ApiError> {
        let mut guard = self.access_token.write().await;
        *guard = None;
        self.keyring.delete_refresh_token().map_err(|e| ApiError::Keyring(e.to_string()))?;
        Ok(())
    }

    fn config(&self, token: Option<&str>) -> Configuration {
        let mut cfg = Configuration::new();
        cfg.base_path = self.get_base_url();
        if let Some(t) = token {
            cfg.bearer_access_token = Some(t.to_string());
        }
        cfg
    }

    // Refresh rotating token with serialized lock
    pub async fn refresh_access_token(&self) -> Result<String, ApiError> {
        let _guard = self.refresh_lock.lock().await;

        let refresh_token = match self.keyring.get_refresh_token().map_err(|e| ApiError::Keyring(e.to_string()))? {
            Some(t) => t,
            None => return Err(ApiError::NotAuthenticated),
        };

        let cfg = self.config(None);
        let req = models::RefreshTokenRequestDto { refresh_token: Some(Some(refresh_token)) };

        match auth_api::refresh(&cfg, Some(req)).await {
            Ok(tokens) => {
                self.keyring.set_refresh_token(&tokens.refresh_token).map_err(|e| ApiError::Keyring(e.to_string()))?;
                let mut tok_guard = self.access_token.write().await;
                *tok_guard = Some(tokens.access_token.clone());
                Ok(tokens.access_token)
            }
            Err(mensetsu_contracts::apis::Error::ResponseError(resp))
                if resp.status == reqwest::StatusCode::UNAUTHORIZED => {
                let mut guard = self.access_token.write().await;
                *guard = None;
                let _ = self.keyring.delete_refresh_token();
                Err(ApiError::Auth(format!("Refresh rejected (HTTP {})", resp.status)))
            }
            Err(e) => Err(ApiError::Auth(format!("Refresh failed: {e}"))),
        }
    }

    pub async fn login(&self, email: &str, password: &str) -> Result<(), ApiError> {
        let cfg = self.config(None);
        let req = models::LoginRequestDto::new(email.to_string(), password.to_string());
        let res = auth_api::login(&cfg, req).await.map_err(|e| ApiError::Auth(format!("{e}")))?;

        if let Some(true) = res.requires_2fa {
            if let Some(Some(temp)) = res.temp_token {
                return Err(ApiError::Requires2Fa { temp_token: temp });
            }
        }

        if let (Some(Some(access)), Some(Some(refresh))) = (res.access_token, res.refresh_token) {
            self.set_tokens(access, refresh).await?;
            Ok(())
        } else {
            Err(ApiError::Auth("Invalid login response: missing tokens".to_string()))
        }
    }

    pub async fn login_2fa(&self, temp_token: &str, code: &str) -> Result<(), ApiError> {
        let cfg = self.config(None);
        let req = models::Login2faRequestDto::new(code.to_string(), temp_token.to_string());
        let res = auth_api::login2fa(&cfg, req).await.map_err(|e| ApiError::Auth(format!("{e}")))?;
        self.set_tokens(res.access_token, res.refresh_token).await?;
        Ok(())
    }

    pub async fn register(&self, name: &str, email: &str, password: &str) -> Result<(), ApiError> {
        let cfg = self.config(None);
        let req = models::RegisterRequestDto::new(email.to_string(), name.to_string(), password.to_string());
        let res = auth_api::register(&cfg, req).await.map_err(|e| ApiError::Auth(format!("{e}")))?;
        self.set_tokens(res.access_token, res.refresh_token).await?;
        Ok(())
    }

    pub async fn logout(&self) -> Result<(), ApiError> {
        let token = {
            let guard = self.access_token.read().await;
            guard.clone()
        };
        let token = match token {
            Some(token) => token,
            None => self.refresh_access_token().await?,
        };
        let cfg = self.config(Some(&token));
        let server_result = auth_api::logout(&cfg).await;
        self.clear_tokens().await?;
        server_result.map_err(|e| ApiError::Auth(format!("Logout failed: {e}")))?;
        Ok(())
    }

    pub async fn list_workspaces(&self) -> Result<Vec<models::WorkspaceResponseDto>, ApiError> {
        let token = {
            let guard = self.access_token.read().await;
            guard.clone()
        };
        let token = match token {
            Some(t) => t,
            None => self.refresh_access_token().await?,
        };
        let cfg = self.config(Some(&token));
        match workspaces_api::list_workspaces(&cfg).await {
            Ok(list) => Ok(list),
            Err(mensetsu_contracts::apis::Error::ResponseError(resp)) if resp.status == reqwest::StatusCode::UNAUTHORIZED => {
                let new_token = self.refresh_access_token().await?;
                let cfg_retry = self.config(Some(&new_token));
                workspaces_api::list_workspaces(&cfg_retry).await.map_err(|e| ApiError::Api(format!("{e}")))
            }
            Err(e) => Err(ApiError::Api(format!("{e}"))),
        }
    }

    pub async fn get_workspace(&self, id: &str) -> Result<models::WorkspaceResponseDto, ApiError> {
        let token = {
            let guard = self.access_token.read().await;
            guard.clone()
        };
        let token = match token {
            Some(t) => t,
            None => self.refresh_access_token().await?,
        };
        let cfg = self.config(Some(&token));
        match workspaces_api::get_workspace(&cfg, id).await {
            Ok(ws) => Ok(ws),
            Err(mensetsu_contracts::apis::Error::ResponseError(resp)) if resp.status == reqwest::StatusCode::UNAUTHORIZED => {
                let new_token = self.refresh_access_token().await?;
                let cfg_retry = self.config(Some(&new_token));
                workspaces_api::get_workspace(&cfg_retry, id).await.map_err(|e| ApiError::Api(format!("{e}")))
            }
            Err(e) => Err(ApiError::Api(format!("{e}"))),
        }
    }

    pub async fn create_workspace(&self, company_name: &str, role_title: &str, resume_text: &str, job_description: &str) -> Result<models::WorkspaceResponseDto, ApiError> {
        let mut req = models::CreateWorkspaceRequestDto::default();
        req.company_name = Some(Some(company_name.to_string()));
        req.role_title = Some(Some(role_title.to_string()));
        req.resume_text = Some(Some(resume_text.to_string()));
        req.job_description = Some(Some(job_description.to_string()));
        self.create_workspace_request(req).await
    }

    pub async fn create_workspace_with_pdf(
        &self,
        company_name: &str,
        role_title: &str,
        job_description: &str,
        pdf_path: &Path,
    ) -> Result<models::WorkspaceResponseDto, ApiError> {
        let filename = pdf_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("resume.pdf")
            .to_string();
        let pdf_bytes = tokio::fs::read(pdf_path)
            .await
            .map_err(|error| ApiError::Network(format!("Failed to read PDF file: {error}")))?;
        let size_bytes = pdf_bytes.len() as i64;

        let token = match self.access_token.read().await.clone() {
            Some(token) => token,
            None => self.refresh_access_token().await?,
        };

        let upload_req = models::CreateResumeUploadRequestDto {
            content_type: "application/pdf".to_string(),
            filename,
            size_bytes,
        };
        let upload = match resume_uploads_api::create_resume_upload(&self.config(Some(&token)), upload_req.clone()).await {
            Ok(upload) => upload,
            Err(mensetsu_contracts::apis::Error::ResponseError(response))
                if response.status == reqwest::StatusCode::UNAUTHORIZED => {
                let token = self.refresh_access_token().await?;
                resume_uploads_api::create_resume_upload(&self.config(Some(&token)), upload_req)
                    .await.map_err(|error| ApiError::Api(format!("{error}")))?
            }
            Err(error) => return Err(ApiError::Api(format!("{error}"))),
        };
        let upload_response = reqwest::Client::new()
            .put(&upload.upload_url)
            .header(reqwest::header::CONTENT_TYPE, "application/pdf")
            .body(pdf_bytes)
            .send()
            .await
            .map_err(|error| ApiError::Network(format!("PDF upload failed: {error}")))?;

        if !upload_response.status().is_success() {
            return Err(ApiError::Network(format!(
                "PDF storage upload rejected (HTTP {})",
                upload_response.status()
            )));
        }

        let mut request = models::CreateWorkspaceRequestDto::default();
        request.company_name = Some(Some(company_name.to_owned()));
        request.role_title = Some(Some(role_title.to_owned()));
        request.job_description = Some(Some(job_description.to_owned()));
        request.resume_upload_id = Some(Some(upload.upload_id));
        self.create_workspace_request(request).await
    }

    async fn create_workspace_request(&self, req: models::CreateWorkspaceRequestDto) -> Result<models::WorkspaceResponseDto, ApiError> {
        let token = match self.access_token.read().await.clone() {
            Some(token) => token,
            None => self.refresh_access_token().await?,
        };
        match workspaces_api::create_workspace(&self.config(Some(&token)), req.clone()).await {
            Ok(workspace) => Ok(workspace),
            Err(mensetsu_contracts::apis::Error::ResponseError(response))
                if response.status == reqwest::StatusCode::UNAUTHORIZED => {
                let token = self.refresh_access_token().await?;
                workspaces_api::create_workspace(&self.config(Some(&token)), req)
                    .await.map_err(|error| ApiError::Api(format!("{error}")))
            }
            Err(error) => Err(ApiError::Api(format!("{error}"))),
        }
    }

    pub async fn get_workspace_match(&self, id: &str) -> Result<models::MatchAnalysis, ApiError> {
        let token = {
            let guard = self.access_token.read().await;
            guard.clone()
        };
        let token = match token {
            Some(t) => t,
            None => self.refresh_access_token().await?,
        };
        let cfg = self.config(Some(&token));
        let res = match workspaces_learning_handlers_api::get_workspace_match(&cfg, id).await {
            Ok(v) => Ok(v),
            Err(mensetsu_contracts::apis::Error::ResponseError(resp)) if resp.status == reqwest::StatusCode::UNAUTHORIZED => {
                let new_token = self.refresh_access_token().await?;
                let cfg_retry = self.config(Some(&new_token));
                workspaces_learning_handlers_api::get_workspace_match(&cfg_retry, id).await.map_err(|e| ApiError::Api(format!("{e}")))
            }
            Err(e) => Err(ApiError::Api(format!("{e}"))),
        }?;

        serde_json::from_value::<models::MatchAnalysis>(res).map_err(|e| ApiError::Api(format!("JSON decode error: {e}")))
    }

    pub async fn get_workspace_company(&self, id: &str) -> Result<serde_json::Value, ApiError> {
        let token = {
            let guard = self.access_token.read().await;
            guard.clone()
        };
        let token = match token {
            Some(t) => t,
            None => self.refresh_access_token().await?,
        };
        let cfg = self.config(Some(&token));
        match workspaces_research_handlers_api::get_workspace_company(&cfg, id).await {
            Ok(value) => Ok(value),
            Err(mensetsu_contracts::apis::Error::ResponseError(resp))
                if resp.status == reqwest::StatusCode::UNAUTHORIZED => {
                let new_token = self.refresh_access_token().await?;
                let cfg_retry = self.config(Some(&new_token));
                workspaces_research_handlers_api::get_workspace_company(&cfg_retry, id)
                    .await.map_err(|e| ApiError::Api(format!("{e}")))
            }
            Err(e) => Err(ApiError::Api(format!("{e}"))),
        }
    }

    pub async fn get_workspace_learning(&self, id: &str) -> Result<Vec<models::LearningUnit>, ApiError> {
        let token = {
            let guard = self.access_token.read().await;
            guard.clone()
        };
        let token = match token {
            Some(t) => t,
            None => self.refresh_access_token().await?,
        };
        let cfg = self.config(Some(&token));
        match workspaces_learning_handlers_api::get_workspace_learning(&cfg, id).await {
            Ok(v) => Ok(v),
            Err(mensetsu_contracts::apis::Error::ResponseError(resp)) if resp.status == reqwest::StatusCode::UNAUTHORIZED => {
                let new_token = self.refresh_access_token().await?;
                let cfg_retry = self.config(Some(&new_token));
                workspaces_learning_handlers_api::get_workspace_learning(&cfg_retry, id).await.map_err(|e| ApiError::Api(format!("{e}")))
            }
            Err(e) => Err(ApiError::Api(format!("{e}"))),
        }
    }

    pub async fn toggle_learning_unit(&self, id: &str, lesson_id: &str) -> Result<(), ApiError> {
        let token = {
            let guard = self.access_token.read().await;
            guard.clone()
        };
        let token = match token {
            Some(t) => t,
            None => self.refresh_access_token().await?,
        };
        let cfg = self.config(Some(&token));
        match workspaces_api::toggle_learning_unit_completed(&cfg, id, lesson_id).await {
            Ok(_) => Ok(()),
            Err(mensetsu_contracts::apis::Error::ResponseError(resp)) if resp.status == reqwest::StatusCode::UNAUTHORIZED => {
                let new_token = self.refresh_access_token().await?;
                let cfg_retry = self.config(Some(&new_token));
                workspaces_api::toggle_learning_unit_completed(&cfg_retry, id, lesson_id).await.map_err(|e| ApiError::Api(format!("{e}"))).map(|_| ())
            }
            Err(e) => Err(ApiError::Api(format!("{e}"))),
        }
    }

    pub async fn get_workspace_questions(&self, id: &str) -> Result<Vec<models::InterviewQuestion>, ApiError> {
        let token = {
            let guard = self.access_token.read().await;
            guard.clone()
        };
        let token = match token {
            Some(t) => t,
            None => self.refresh_access_token().await?,
        };
        let cfg = self.config(Some(&token));
        match workspaces_learning_handlers_api::get_workspace_questions(&cfg, id).await {
            Ok(v) => Ok(v),
            Err(mensetsu_contracts::apis::Error::ResponseError(resp)) if resp.status == reqwest::StatusCode::UNAUTHORIZED => {
                let new_token = self.refresh_access_token().await?;
                let cfg_retry = self.config(Some(&new_token));
                workspaces_learning_handlers_api::get_workspace_questions(&cfg_retry, id).await.map_err(|e| ApiError::Api(format!("{e}")))
            }
            Err(e) => Err(ApiError::Api(format!("{e}"))),
        }
    }

    pub async fn get_llm_settings(&self) -> Result<Vec<models::LlmConfigItemDto>, ApiError> {
        let token = {
            let guard = self.access_token.read().await;
            guard.clone()
        };
        let token = match token {
            Some(t) => t,
            None => self.refresh_access_token().await?,
        };
        let cfg = self.config(Some(&token));
        match settings_api::get_llm_settings(&cfg).await {
            Ok(v) => Ok(v),
            Err(mensetsu_contracts::apis::Error::ResponseError(resp)) if resp.status == reqwest::StatusCode::UNAUTHORIZED => {
                let new_token = self.refresh_access_token().await?;
                let cfg_retry = self.config(Some(&new_token));
                settings_api::get_llm_settings(&cfg_retry).await.map_err(|e| ApiError::Api(format!("{e}")))
            }
            Err(e) => Err(ApiError::Api(format!("{e}"))),
        }
    }

    pub async fn update_llm_setting(&self, req: models::UpsertLlmConfigRequestDto) -> Result<models::LlmConfigItemDto, ApiError> {
        let token = {
            let guard = self.access_token.read().await;
            guard.clone()
        };
        let token = match token {
            Some(t) => t,
            None => self.refresh_access_token().await?,
        };
        let cfg = self.config(Some(&token));
        match settings_api::upsert_llm_setting(&cfg, req.clone()).await {
            Ok(v) => Ok(v),
            Err(mensetsu_contracts::apis::Error::ResponseError(resp)) if resp.status == reqwest::StatusCode::UNAUTHORIZED => {
                let new_token = self.refresh_access_token().await?;
                let cfg_retry = self.config(Some(&new_token));
                settings_api::upsert_llm_setting(&cfg_retry, req).await.map_err(|e| ApiError::Api(format!("{e}")))
            }
            Err(e) => Err(ApiError::Api(format!("{e}"))),
        }
    }
}
