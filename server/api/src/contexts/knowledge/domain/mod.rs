use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::contexts::workspace::domain::WorkspaceId;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SourceCitation {
    pub id: String,
    pub workspace_id: WorkspaceId,
    pub url: String,
    pub title: String,
    pub domain: String,
    pub snippet: String,
    pub source_type: String,
    pub confidence: f32,
    pub retrieved_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CompanyClaim {
    pub id: String,
    pub workspace_id: WorkspaceId,
    pub statement: String,
    pub claim_type: String,
    pub confidence: f32,
    pub source_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct KnowledgeSkillNode {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub category: String,
    pub candidate_status: String,
    pub importance: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct KnowledgeGraphResponse {
    pub nodes: Vec<KnowledgeSkillNode>,
    pub edges: Vec<KnowledgeEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct KnowledgeEdge {
    pub source: String,
    pub target: String,
    pub relation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CompanyResearch {
    pub workspace_id: WorkspaceId,
    pub company_name: String,
    pub overview: String,
    pub business_model: String,
    pub engineering_culture: Vec<String>,
    pub technology_stack: Vec<String>,
    pub recent_developments: Vec<String>,
    pub interview_process_insights: Vec<String>,
    pub sources: Vec<SourceCitation>,
}
