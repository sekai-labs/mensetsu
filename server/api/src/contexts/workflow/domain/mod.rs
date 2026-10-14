pub mod repository;
pub use repository::*;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::contexts::identity::domain::UserId;
use crate::contexts::workspace::domain::WorkspaceId;
use crate::shared::error::{AppError, Result};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, ToSchema)]
pub struct WorkflowRunId(pub String);

impl Default for WorkflowRunId {
    fn default() -> Self {
        Self(format!("run_{}", Uuid::new_v4()))
    }
}

impl std::fmt::Display for WorkflowRunId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl WorkflowRunId {
    pub fn new(id: impl Into<String>) -> Result<Self> {
        let clean = id.into().trim().to_string();
        if clean.is_empty() {
            return Err(AppError::Validation("Workflow run ID cannot be empty".into()));
        }
        let clean = clean.replace("workflow_run:", "").replace('`', "");
        Ok(Self(clean))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct RunOwnership {
    pub run_id: WorkflowRunId,
    pub workspace_id: WorkspaceId,
    pub user_id: UserId,
}

impl RunOwnership {
    pub fn new(run_id: WorkflowRunId, workspace_id: WorkspaceId, user_id: UserId) -> Self {
        Self {
            run_id,
            workspace_id,
            user_id,
        }
    }
}
