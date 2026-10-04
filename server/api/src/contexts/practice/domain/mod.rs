use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::contexts::workspace::domain::WorkspaceId;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum QuestionCategory {
    ResumeDeepDive,
    ProjectDeepDive,
    TechnicalFundamentals,
    TargetStack,
    SystemDesign,
    Debugging,
    Behavioral,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum QuestionDifficulty {
    Easy,
    Medium,
    Hard,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct InterviewQuestion {
    pub id: String,
    pub workspace_id: WorkspaceId,
    pub category: QuestionCategory,
    pub question: String,
    pub difficulty: QuestionDifficulty,
    pub skills_tested: Vec<String>,
    pub interviewer_intent: String,
    pub answer_outline: Vec<String>,
    pub strong_answer_rubric: Vec<String>,
    pub common_weaknesses: Vec<String>,
    pub follow_up_questions: Vec<String>,
    pub created_at: DateTime<Utc>,
}
