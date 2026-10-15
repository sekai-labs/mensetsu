use std::time::Duration;

use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::{RequestBuilder, StatusCode, Url};
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::interfaces::http::workflow_dtos::WorkflowRun;
use crate::shared::config::AppConfig;
use crate::shared::error::{AppError, Result};

pub const WORKFLOWS_BASE: &str = "/internal/v1/workflows";

#[derive(Debug, Clone, Serialize)]
pub struct CreateRunPayload {
    pub workspace_id: String,
    pub user_id: String,
    pub workflow_id: String,
    pub company_name: String,
    pub role_title: String,
    pub company_urls: Vec<String>,
    pub resume_text: String,
    pub resume_pdf_base64: Option<String>,
    pub resume_filename: Option<String>,
    pub job_description: String,
    pub jd_url: Option<String>,
    pub seniority: Option<String>,
    pub user_prompt: Option<String>,
    pub workflow_mode: Option<String>,
    pub llm_config: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResumePlanPayload {
    pub prompt: String,
}
#[derive(Clone)]
pub struct IntelligenceClient {
    client: reqwest::Client,
    workflows_url: Url,
}

impl IntelligenceClient {
    pub fn new(config: &AppConfig) -> Result<Self> {
        config.validate().map_err(AppError::Internal)?;

        let mut auth = HeaderValue::from_str(&format!("Bearer {}", config.internal_service_token))
            .map_err(|e| AppError::Internal(format!("Invalid INTERNAL_SERVICE_TOKEN: {e}")))?;
        auth.set_sensitive(true);
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, auth);

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .connect_timeout(Duration::from_secs(5))
            .build()
            .map_err(|e| AppError::Internal(format!("Failed to build HTTP client: {e}")))?;

        let base = format!("{}{}", config.intelligence_url.trim_end_matches('/'), WORKFLOWS_BASE);
        let workflows_url = Url::parse(&base)
            .map_err(|e| AppError::Internal(format!("Invalid INTELLIGENCE_URL: {e}")))?;

        Ok(Self { client, workflows_url })
    }

    fn endpoint(&self, segments: &[&str]) -> Result<Url> {
        if segments.iter().any(|s| s.is_empty() || *s == "." || *s == "..") {
            return Err(AppError::NotFound("Resource not found".into()));
        }
        let mut url = self.workflows_url.clone();
        url.path_segments_mut()
            .map_err(|_| AppError::Internal("INTELLIGENCE_URL cannot be a base URL".into()))?
            .pop_if_empty()
            .extend(segments);
        Ok(url)
    }

    pub async fn get_json<T: DeserializeOwned>(
        &self,
        segments: &[&str],
        query: &[(&str, String)],
    ) -> Result<T> {
        let url = self.endpoint(segments)?;
        send_json(self.client.get(url).query(query)).await
    }

    pub async fn post_json<T: DeserializeOwned>(&self, segments: &[&str]) -> Result<T> {
        let url = self.endpoint(segments)?;
        send_json(self.client.post(url)).await
    }

    pub async fn post_json_with_body<B: Serialize + ?Sized, T: DeserializeOwned>(
        &self,
        segments: &[&str],
        body: &B,
    ) -> Result<T> {
        let url = self.endpoint(segments)?;
        send_json(self.client.post(url).json(body)).await
    }

    pub async fn resume_plan(
        &self,
        run_id: &str,
        payload: &ResumePlanPayload,
    ) -> Result<serde_json::Value> {
        self.post_json_with_body(&["runs", run_id, "resume-plan"], payload).await
    }
    pub async fn create_run(
        &self,
        payload: &CreateRunPayload,
        idempotency_key: Option<&str>,
    ) -> Result<WorkflowRun> {
        let mut req = self.client.post(self.endpoint(&["runs"])?).json(payload);
        if let Some(key) = idempotency_key {
            req = req.header("Idempotency-Key", key);
        }
        send_json(req).await
    }

    pub async fn open_event_stream(&self, run_id: &str, after_seq: u64) -> Result<reqwest::Response> {
        let url = self.endpoint(&["runs", run_id, "events"])?;
        let resp = self
            .client
            .get(url)
            .query(&[("after_seq", after_seq)])
            .header(reqwest::header::ACCEPT, "text/event-stream")
            .send()
            .await
            .map_err(|e| AppError::ExternalService(format!("Intelligence service unreachable: {e}")))?;
        check_status(resp).await
    }
}

async fn check_status(resp: reqwest::Response) -> Result<reqwest::Response> {
    let status = resp.status();
    if status.is_success() {
        return Ok(resp);
    }
    let body = resp.text().await.unwrap_or_default();
    tracing::warn!(%status, body = %body, "intelligence service returned an error");
    Err(match status {
        StatusCode::NOT_FOUND => AppError::NotFound("Resource not found".into()),
        StatusCode::BAD_REQUEST => AppError::Validation(
            serde_json::from_str::<serde_json::Value>(&body)
                .ok()
                .and_then(|v| v["detail"].as_str().map(str::to_owned))
                .unwrap_or_else(|| "Intelligence service rejected the request".into()),
        ),
        StatusCode::SERVICE_UNAVAILABLE => {
            AppError::ServiceUnavailable("Intelligence service unavailable".into())
        }
        _ => AppError::ExternalService(format!("Intelligence service returned status {status}")),
    })
}

async fn send_json<T: DeserializeOwned>(req: RequestBuilder) -> Result<T> {
    let resp = req
        .send()
        .await
        .map_err(|e| AppError::ExternalService(format!("Intelligence service unreachable: {e}")))?;
    let bytes = check_status(resp)
        .await?
        .bytes()
        .await
        .map_err(|e| AppError::ExternalService(format!("Failed to read intelligence response: {e}")))?;
    serde_json::from_slice(&bytes).map_err(|e| {
        tracing::error!(error = %e, "intelligence response violated the workflows.v1 contract");
        AppError::ExternalService(format!("Intelligence response did not match contract: {e}"))
    })
}
