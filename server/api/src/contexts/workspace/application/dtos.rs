use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::contexts::workspace::domain::{PipelinePhase, SeniorityLevel, WorkspaceStatus};

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateWorkspaceRequestDto {
    pub prompt: Option<String>,
    pub company_name: Option<String>,
    pub role_title: Option<String>,
    pub workflow_mode: Option<String>,
    pub job_description: Option<String>,
    pub jd_url: Option<String>,
    #[serde(default)]
    pub company_urls: Vec<String>,
    pub seniority: Option<SeniorityLevel>,
    pub target_date: Option<chrono::DateTime<chrono::Utc>>,
    pub prep_time_weeks: Option<u8>,
    pub resume_text: Option<String>,
    pub resume_pdf_base64: Option<String>,
    pub resume_filename: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct PatchWorkspaceObjectiveDto {
    #[validate(length(min = 1, message = "Objective prompt must not be empty"))]
    pub prompt: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct WorkspaceResponseDto {
    pub id: String,
    pub user_id: String,
    pub company_name: String,
    pub role_title: String,
    pub status: WorkspaceStatus,
    pub current_phase: PipelinePhase,
    pub progress_percent: u8,
    pub seniority: Option<SeniorityLevel>,
    pub user_prompt: Option<String>,
    pub current_objective: Option<String>,
    pub workflow_mode: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct WorkspaceProgressEventDto {
    pub workspace_id: String,
    pub phase: PipelinePhase,
    pub percent: u8,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
