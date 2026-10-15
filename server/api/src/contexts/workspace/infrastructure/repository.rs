use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::contexts::identity::domain::UserId;
use crate::contexts::workspace::domain::{
    PipelinePhase, SeniorityLevel, Workspace, WorkspaceId, WorkspaceRepository, WorkspaceStatus,
};
use crate::contexts::workspace::infrastructure::db::Db;
use crate::shared::error::{AppError, Result};

#[derive(Debug, Serialize, Deserialize)]
struct WorkspaceDbRow {
    id: String,
    user_id: String,
    company_name: String,
    role_title: String,
    status: String,
    current_phase: String,
    progress_percent: u8,
    seniority: Option<String>,
    target_date: Option<String>,
    prep_time_weeks: Option<u8>,
    user_prompt: Option<String>,
    current_objective: Option<String>,
    workflow_mode: Option<String>,
    created_at: Option<String>,
    updated_at: Option<String>,
}

impl WorkspaceDbRow {
    fn into_domain(self) -> Workspace {
        let clean_id = self.id.replace("workspace:", "").replace('`', "");
        let clean_uid = self.user_id.replace("user:", "").replace('`', "");

        let status = match self.status.as_str() {
            "processing" => WorkspaceStatus::Processing,
            "ready" => WorkspaceStatus::Ready,
            "failed" => WorkspaceStatus::Failed,
            _ => WorkspaceStatus::Pending,
        };

        let current_phase = match self.current_phase.as_str() {
            "analyzing_resume" => PipelinePhase::AnalyzingResume,
            "reading_job_description" => PipelinePhase::ReadingJobDescription,
            "researching_company" => PipelinePhase::ResearchingCompany,
            "researching_engineering_environment" => PipelinePhase::ResearchingEngineeringEnvironment,
            "extracting_role_requirements" => PipelinePhase::ExtractingRoleRequirements,
            "mapping_resume_evidence" => PipelinePhase::MappingResumeEvidence,
            "constructing_knowledge_graph" => PipelinePhase::ConstructingKnowledgeGraph,
            "building_learning_plan" => PipelinePhase::BuildingLearningPlan,
            "generating_lessons" => PipelinePhase::GeneratingLessons,
            "building_interview_question_book" => PipelinePhase::BuildingInterviewQuestionBook,
            "final_validation" => PipelinePhase::FinalValidation,
            "ready" => PipelinePhase::Ready,
            "failed" => PipelinePhase::Failed,
            _ => PipelinePhase::PreparingInput,
        };

        let seniority = self.seniority.and_then(|s| match s.as_str() {
            "junior" => Some(SeniorityLevel::Junior),
            "mid" => Some(SeniorityLevel::Mid),
            "senior" => Some(SeniorityLevel::Senior),
            "staff" => Some(SeniorityLevel::Staff),
            "principal" => Some(SeniorityLevel::Principal),
            "lead" => Some(SeniorityLevel::Lead),
            _ => None,
        });

        let target_date = self
            .target_date
            .and_then(|d| DateTime::parse_from_rfc3339(&d).ok())
            .map(|dt| dt.with_timezone(&Utc));

        let created_at = self
            .created_at
            .and_then(|d| DateTime::parse_from_rfc3339(&d).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);

        let updated_at = self
            .updated_at
            .and_then(|d| DateTime::parse_from_rfc3339(&d).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);

        Workspace {
            id: WorkspaceId(clean_id),
            user_id: UserId(clean_uid),
            company_name: self.company_name,
            role_title: self.role_title,
            status,
            current_phase,
            progress_percent: self.progress_percent,
            seniority,
            target_date,
            prep_time_weeks: self.prep_time_weeks,
            user_prompt: self.user_prompt,
            current_objective: self.current_objective,
            workflow_mode: self.workflow_mode.unwrap_or_else(|| "hybrid".to_string()),
            created_at,
            updated_at,
        }
    }
}

pub struct SurrealWorkspaceRepository {
    client: Db,
}

impl SurrealWorkspaceRepository {
    pub fn new(client: Db) -> Self {
        Self { client }
    }
}

#[async_trait]
impl WorkspaceRepository for SurrealWorkspaceRepository {
    async fn find_by_id(&self, id: &WorkspaceId) -> Result<Option<Workspace>> {
        let clean_id = id.0.replace("workspace:", "").replace('`', "");
        let mut res = self
            .client
            .query("SELECT * FROM type::record('workspace', $id);")
            .bind(("id", clean_id.clone()))
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let item: Option<serde_json::Value> = res.take(0).map_err(|e| AppError::Database(e.to_string()))?;

        if let Some(val) = item {
            let row: WorkspaceDbRow = serde_json::from_value(val)
                .map_err(|e| AppError::Database(format!("Deserialization error: {e}")))?;
            Ok(Some(row.into_domain()))
        } else {
            Ok(None)
        }
    }

    async fn find_by_user_id(&self, user_id: &UserId) -> Result<Vec<Workspace>> {
        let clean_uid = user_id.0.replace("user:", "").replace('`', "");
        let mut res = self
            .client
            .query("SELECT * FROM workspace WHERE user_id = type::record('user', $user_id) ORDER BY created_at DESC;")
            .bind(("user_id", clean_uid))
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let values: Vec<serde_json::Value> = res.take(0).map_err(|e| AppError::Database(e.to_string()))?;

        let rows: Vec<Workspace> = values
            .into_iter()
            .filter_map(|val| serde_json::from_value::<WorkspaceDbRow>(val).ok())
            .map(|r| r.into_domain())
            .collect();

        Ok(rows)
    }

    async fn save(&self, workspace: &Workspace) -> Result<()> {
        let query = "
            UPSERT type::record('workspace', $id) SET
                user_id = type::record('user', $user_id),
                company_name = $company_name,
                role_title = $role_title,
                status = $status,
                current_phase = $current_phase,
                progress_percent = $progress_percent,
                seniority = $seniority,
                target_date = $target_date,
                prep_time_weeks = $prep_time_weeks,
                user_prompt = $user_prompt,
                current_objective = $current_objective,
                workflow_mode = $workflow_mode,
                created_at = type::datetime($created_at),
                updated_at = type::datetime($updated_at);
        ";

        let status_str = match workspace.status {
            WorkspaceStatus::Processing => "processing",
            WorkspaceStatus::Ready => "ready",
            WorkspaceStatus::Failed => "failed",
            WorkspaceStatus::Pending => "pending",
        };

        let phase_str = match workspace.current_phase {
            PipelinePhase::AnalyzingResume => "analyzing_resume",
            PipelinePhase::ReadingJobDescription => "reading_job_description",
            PipelinePhase::ResearchingCompany => "researching_company",
            PipelinePhase::ResearchingEngineeringEnvironment => "researching_engineering_environment",
            PipelinePhase::ExtractingRoleRequirements => "extracting_role_requirements",
            PipelinePhase::MappingResumeEvidence => "mapping_resume_evidence",
            PipelinePhase::ConstructingKnowledgeGraph => "constructing_knowledge_graph",
            PipelinePhase::BuildingLearningPlan => "building_learning_plan",
            PipelinePhase::GeneratingLessons => "generating_lessons",
            PipelinePhase::BuildingInterviewQuestionBook => "building_interview_question_book",
            PipelinePhase::FinalValidation => "final_validation",
            PipelinePhase::Ready => "ready",
            PipelinePhase::Failed => "failed",
            PipelinePhase::PreparingInput => "preparing_input",
        };

        let seniority_str = workspace
            .seniority
            .as_ref()
            .map(|s| serde_json::to_value(s).unwrap_or_default().as_str().unwrap_or("").to_string());

        let target_date_str = workspace.target_date.as_ref().map(|d| d.to_rfc3339());

        self.client
            .query(query)
            .bind(("id", workspace.id.0.clone()))
            .bind(("user_id", workspace.user_id.0.clone()))
            .bind(("company_name", workspace.company_name.clone()))
            .bind(("role_title", workspace.role_title.clone()))
            .bind(("status", status_str))
            .bind(("current_phase", phase_str))
            .bind(("progress_percent", workspace.progress_percent as i64))
            .bind(("seniority", seniority_str))
            .bind(("target_date", target_date_str))
            .bind(("prep_time_weeks", workspace.prep_time_weeks.map(|w| w as i64)))
            .bind(("user_prompt", workspace.user_prompt.clone()))
            .bind(("current_objective", workspace.current_objective.clone()))
            .bind(("workflow_mode", workspace.workflow_mode.clone()))
            .bind(("created_at", workspace.created_at.to_rfc3339()))
            .bind(("updated_at", workspace.updated_at.to_rfc3339()))
            .await
            .map_err(|e| AppError::Database(format!("Failed to save workspace: {e}")))?;
        Ok(())
    }

    async fn update_progress(
        &self,
        id: &WorkspaceId,
        phase: PipelinePhase,
        percent: u8,
    ) -> Result<()> {
        let phase_str = match phase {
            PipelinePhase::AnalyzingResume => "analyzing_resume",
            PipelinePhase::ReadingJobDescription => "reading_job_description",
            PipelinePhase::ResearchingCompany => "researching_company",
            PipelinePhase::ResearchingEngineeringEnvironment => "researching_engineering_environment",
            PipelinePhase::ExtractingRoleRequirements => "extracting_role_requirements",
            PipelinePhase::MappingResumeEvidence => "mapping_resume_evidence",
            PipelinePhase::ConstructingKnowledgeGraph => "constructing_knowledge_graph",
            PipelinePhase::BuildingLearningPlan => "building_learning_plan",
            PipelinePhase::GeneratingLessons => "generating_lessons",
            PipelinePhase::BuildingInterviewQuestionBook => "building_interview_question_book",
            PipelinePhase::FinalValidation => "final_validation",
            PipelinePhase::Ready => "ready",
            PipelinePhase::Failed => "failed",
            PipelinePhase::PreparingInput => "preparing_input",
        };

        let status_str = match phase {
            PipelinePhase::Ready => "ready",
            PipelinePhase::Failed => "failed",
            _ => "processing",
        };

        self.client
            .query("UPDATE type::record('workspace', $id) SET current_phase = $phase, status = $status, progress_percent = $percent, updated_at = time::now();")
            .bind(("id", id.0.clone()))
            .bind(("phase", phase_str))
            .bind(("status", status_str))
            .bind(("percent", percent as i64))
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }
}
