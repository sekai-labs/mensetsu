use async_trait::async_trait;
use crate::contexts::identity::domain::UserId;
use crate::contexts::workspace::domain::{Workspace, WorkspaceId};
use crate::shared::error::Result;

#[async_trait]
pub trait WorkspaceRepository: Send + Sync {
    async fn find_by_id(&self, id: &WorkspaceId) -> Result<Option<Workspace>>;
    async fn find_by_user_id(&self, user_id: &UserId) -> Result<Vec<Workspace>>;
    async fn save(&self, workspace: &Workspace) -> Result<()>;
    async fn update_progress(
        &self,
        id: &WorkspaceId,
        phase: crate::contexts::workspace::domain::PipelinePhase,
        percent: u8,
    ) -> Result<()>;
}
