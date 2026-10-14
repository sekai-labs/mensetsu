use async_trait::async_trait;

use crate::contexts::identity::domain::UserId;
use crate::contexts::workspace::domain::WorkspaceId;
use crate::contexts::workflow::domain::WorkflowRunId;
use crate::shared::error::Result;

#[async_trait]
pub trait WorkflowRunRepository: Send + Sync {
    async fn find_workspace_id(&self, run_id: &WorkflowRunId) -> Result<Option<WorkspaceId>>;

    async fn list_user_run_ids(
        &self,
        user_id: &UserId,
        workspace_id: Option<&WorkspaceId>,
        status: Option<&str>,
        run_type: Option<&str>,
        limit: usize,
        cursor: Option<&str>,
    ) -> Result<(Vec<WorkflowRunId>, Option<String>)>;
}
