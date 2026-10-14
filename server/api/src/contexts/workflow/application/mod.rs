use std::sync::Arc;

use crate::contexts::identity::domain::UserId;
use crate::contexts::workspace::domain::{WorkspaceId, WorkspaceRepository};
use crate::contexts::workflow::domain::{RunOwnership, WorkflowRunId, WorkflowRunRepository};
use crate::shared::error::{AppError, Result};

pub struct WorkflowService {
    workflow_repo: Arc<dyn WorkflowRunRepository>,
    workspace_repo: Arc<dyn WorkspaceRepository>,
}

impl WorkflowService {
    pub fn new(
        workflow_repo: Arc<dyn WorkflowRunRepository>,
        workspace_repo: Arc<dyn WorkspaceRepository>,
    ) -> Self {
        Self {
            workflow_repo,
            workspace_repo,
        }
    }

    pub async fn authorize_run(
        &self,
        run_id: &WorkflowRunId,
        user_id: &UserId,
    ) -> Result<RunOwnership> {
        let not_found_err = AppError::NotFound(format!("Workflow run {} not found", run_id.0));

        let ws_id = self
            .workflow_repo
            .find_workspace_id(run_id)
            .await?
            .ok_or_else(|| not_found_err.clone())?;

        let workspace = self
            .workspace_repo
            .find_by_id(&ws_id)
            .await?
            .ok_or_else(|| not_found_err.clone())?;

        if workspace.user_id != *user_id {
            return Err(not_found_err);
        }

        Ok(RunOwnership::new(run_id.clone(), ws_id, user_id.clone()))
    }

    pub async fn list_user_run_ids(
        &self,
        user_id: &UserId,
        workspace_id: Option<&WorkspaceId>,
        status: Option<&str>,
        run_type: Option<&str>,
        limit: usize,
        cursor: Option<&str>,
    ) -> Result<(Vec<WorkflowRunId>, Option<String>)> {
        if let Some(ws_id) = workspace_id {
            let ws = self
                .workspace_repo
                .find_by_id(ws_id)
                .await?
                .ok_or_else(|| AppError::NotFound(format!("Workspace {} not found", ws_id.0)))?;

            if ws.user_id != *user_id {
                return Err(AppError::Forbidden(
                    "Access denied: You do not own this workspace".into(),
                ));
            }
        }

        self.workflow_repo
            .list_user_run_ids(user_id, workspace_id, status, run_type, limit, cursor)
            .await
    }
}
