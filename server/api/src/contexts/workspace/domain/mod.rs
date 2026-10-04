pub mod match_analysis;

pub use match_analysis::*;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::contexts::identity::domain::UserId;
use crate::shared::error::{AppError, Result};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct WorkspaceId(pub String);

impl Default for WorkspaceId {
    fn default() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceStatus {
    Pending,
    Processing,
    Ready,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PipelinePhase {
    PreparingInput,
    AnalyzingResume,
    ReadingJobDescription,
    ResearchingCompany,
    ResearchingEngineeringEnvironment,
    ExtractingRoleRequirements,
    MappingResumeEvidence,
    ConstructingKnowledgeGraph,
    BuildingLearningPlan,
    GeneratingLessons,
    BuildingInterviewQuestionBook,
    FinalValidation,
    Ready,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SeniorityLevel {
    Junior,
    Mid,
    Senior,
    Staff,
    Principal,
    Lead,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Workspace {
    pub id: WorkspaceId,
    pub user_id: UserId,
    pub company_name: String,
    pub role_title: String,
    pub status: WorkspaceStatus,
    pub current_phase: PipelinePhase,
    pub progress_percent: u8,
    pub seniority: Option<SeniorityLevel>,
    pub target_date: Option<DateTime<Utc>>,
    pub prep_time_weeks: Option<u8>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Workspace {
    pub fn new(
        user_id: UserId,
        company_name: String,
        role_title: String,
        seniority: Option<SeniorityLevel>,
        target_date: Option<DateTime<Utc>>,
        prep_time_weeks: Option<u8>,
    ) -> Result<Self> {
        let trimmed_company = company_name.trim();
        let trimmed_role = role_title.trim();

        if trimmed_company.is_empty() {
            return Err(AppError::Validation("Company name cannot be empty".into()));
        }
        if trimmed_role.is_empty() {
            return Err(AppError::Validation("Role title cannot be empty".into()));
        }

        let now = Utc::now();
        Ok(Self {
            id: WorkspaceId::default(),
            user_id,
            company_name: trimmed_company.to_string(),
            role_title: trimmed_role.to_string(),
            status: WorkspaceStatus::Pending,
            current_phase: PipelinePhase::PreparingInput,
            progress_percent: 0,
            seniority,
            target_date,
            prep_time_weeks,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn advance_progress(&mut self, phase: PipelinePhase, percent: u8) {
        self.current_phase = phase.clone();
        self.progress_percent = percent.min(100);
        self.updated_at = Utc::now();

        match phase {
            PipelinePhase::Ready => self.status = WorkspaceStatus::Ready,
            PipelinePhase::Failed => self.status = WorkspaceStatus::Failed,
            _ => self.status = WorkspaceStatus::Processing,
        }
    }
}
