use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::contexts::workspace::domain::WorkspaceId;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct VisualDiagram {
    #[serde(rename = "type")]
    pub diagram_type: String,
    pub caption: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LessonChapter {
    #[serde(default)]
    pub id: Option<String>,
    pub timestamp: String,
    pub title: String,
    pub summary: Option<String>,
    #[serde(default)]
    pub focus_area: Option<String>,
    #[serde(default)]
    pub content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LearningUnit {
    pub id: String,
    pub workspace_id: WorkspaceId,
    pub skill: String,
    pub title: String,
    pub prerequisites: Vec<String>,
    pub estimated_minutes: u32,
    pub summary: String,
    pub content_markdown: String,
    pub diagram: Option<VisualDiagram>,
    pub video_url: Option<String>,
    pub video_title: Option<String>,
    pub chapters: Option<Vec<LessonChapter>>,
    #[serde(default)]
    pub completed: bool,
    pub created_at: DateTime<Utc>,
}
