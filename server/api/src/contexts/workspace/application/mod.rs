pub mod dtos;
pub use dtos::*;

use std::sync::Arc;
use crate::contexts::identity::domain::UserId;
use crate::contexts::workspace::domain::{Workspace, WorkspaceId, WorkspaceRepository};
use crate::shared::error::{AppError, Result};

pub struct WorkspaceService {
    repository: Arc<dyn WorkspaceRepository>,
}

impl WorkspaceService {
    pub fn new(repository: Arc<dyn WorkspaceRepository>) -> Self {
        Self { repository }
    }

    pub async fn create_workspace(
        &self,
        user_id: UserId,
        dto: CreateWorkspaceRequestDto,
    ) -> Result<Workspace> {
        let workspace = Workspace::new(
            user_id,
            dto.company_name,
            dto.role_title,
            dto.seniority,
            dto.target_date,
            dto.prep_time_weeks,
        )?;

        self.repository.save(&workspace).await?;
        Ok(workspace)
    }

    pub async fn get_workspace(
        &self,
        id: &WorkspaceId,
        user_id: &UserId,
    ) -> Result<Workspace> {
        let workspace = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Workspace {} not found", id.0)))?;

        if workspace.user_id != *user_id {
            return Err(AppError::Forbidden(
                "Access denied: You do not own this workspace".into(),
            ));
        }

        Ok(workspace)
    }

    pub async fn list_workspaces(&self, user_id: &UserId) -> Result<Vec<Workspace>> {
        self.repository.find_by_user_id(user_id).await
    }

    pub async fn update_progress(
        &self,
        id: &WorkspaceId,
        phase: crate::contexts::workspace::domain::PipelinePhase,
        percent: u8,
    ) -> Result<()> {
        self.repository.update_progress(id, phase, percent).await
    }
}
