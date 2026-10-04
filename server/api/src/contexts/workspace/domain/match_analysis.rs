use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::contexts::workspace::domain::WorkspaceId;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StrengthEvidence {
    pub skill: String,
    pub evidence_quote: String,
    pub relevance: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GapEvidence {
    pub skill: String,
    pub requirement_quote: String,
    pub importance: String,
    pub recommended_focus: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct WordingImprovement {
    pub original_text: String,
    pub suggested_revision: String,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MatchAnalysis {
    pub workspace_id: WorkspaceId,
    pub overall_match_ratio: f32,
    pub strengths: Vec<StrengthEvidence>,
    pub gaps: Vec<GapEvidence>,
    pub risks: Vec<String>,
    pub wording_improvements: Vec<WordingImprovement>,
}
