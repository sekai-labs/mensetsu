use async_trait::async_trait;
use serde::Deserialize;

use crate::contexts::identity::domain::UserId;
use crate::contexts::workspace::domain::WorkspaceId;
use crate::contexts::workspace::infrastructure::db::Db;
use crate::contexts::workflow::domain::{WorkflowRunId, WorkflowRunRepository};
use crate::shared::error::{AppError, Result};

#[derive(Debug, Deserialize)]
struct RunListRow {
    run_id: String,
}

pub struct SurrealWorkflowRunRepository {
    client: Db,
}

impl SurrealWorkflowRunRepository {
    pub fn new(client: Db) -> Self {
        Self { client }
    }
}

#[async_trait]
impl WorkflowRunRepository for SurrealWorkflowRunRepository {
    async fn find_workspace_id(&self, run_id: &WorkflowRunId) -> Result<Option<WorkspaceId>> {
        let clean_run = run_id.0.replace("workflow_run:", "").replace('`', "");

        let query = "SELECT VALUE record::id(workspace_id) FROM ONLY type::record('workflow_run', $id);";
        let mut res = self
            .client
            .query(query)
            .bind(("id", clean_run.clone()))
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut item: Option<String> = res.take(0).map_err(|e| AppError::Database(e.to_string()))?;

        if item.is_none() {
            let fallback = "SELECT VALUE record::id(workspace_id) FROM ONLY workflow_run WHERE id = type::record('workflow_run', $id) OR id = $id;";
            let mut res_fallback = self
                .client
                .query(fallback)
                .bind(("id", clean_run))
                .await
                .map_err(|e| AppError::Database(e.to_string()))?;
            item = res_fallback.take(0).unwrap_or_default();
        }

        Ok(item.map(|ws| WorkspaceId(ws.replace("workspace:", "").replace('`', ""))))
    }

    async fn list_user_run_ids(
        &self,
        user_id: &UserId,
        workspace_id: Option<&WorkspaceId>,
        status: Option<&str>,
        run_type: Option<&str>,
        limit: usize,
        cursor: Option<&str>,
    ) -> Result<(Vec<WorkflowRunId>, Option<String>)> {
        let clean_uid = user_id.0.replace("user:", "").replace('`', "");
        let clean_ws = workspace_id.map(|w| w.0.replace("workspace:", "").replace('`', ""));
        let clean_cursor = cursor.map(|c| c.replace("workflow_run:", "").replace('`', ""));
        let fetch_limit = limit.saturating_add(1);

        let query = "
            LET $owned = (SELECT VALUE id FROM workspace WHERE user_id = type::record('user', $user_id));
            LET $cur = IF !$cursor { NONE } ELSE { (SELECT id, created_at FROM ONLY type::record('workflow_run', $cursor) WHERE workspace_id IN $owned) };
            RETURN !$cursor OR $cur != NONE;
            SELECT id, record::id(id) AS run_id, created_at FROM workflow_run
            WHERE workspace_id IN $owned
              AND (!$workspace_id OR workspace_id = type::record('workspace', $workspace_id))
              AND (!$status OR status = $status)
              AND (!$type OR `type` = $type)
              AND ($cur = NONE OR created_at < $cur.created_at OR (created_at = $cur.created_at AND id < $cur.id))
            ORDER BY created_at DESC, id DESC
            LIMIT $fetch_limit;
        ";

        let mut res = self
            .client
            .query(query)
            .bind(("user_id", clean_uid))
            .bind(("workspace_id", clean_ws))
            .bind(("status", status.map(String::from)))
            .bind(("type", run_type.map(String::from)))
            .bind(("cursor", clean_cursor))
            .bind(("fetch_limit", fetch_limit as i64))
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let cursor_valid: Option<bool> = res
            .take(2)
            .map_err(|e| AppError::Database(format!("Failed to resolve run cursor: {e}")))?;
        if cursor_valid != Some(true) {
            return Err(AppError::Validation("Invalid cursor".into()));
        }

        let values: Vec<serde_json::Value> = res
            .take(3)
            .map_err(|e| AppError::Database(format!("Failed to retrieve workflow runs: {e}")))?;
        let rows: Vec<RunListRow> = values
            .into_iter()
            .map(serde_json::from_value)
            .collect::<std::result::Result<_, _>>()
            .map_err(|e| AppError::Database(format!("Malformed workflow_run row: {e}")))?;
        let has_more = rows.len() > limit;
        let run_ids: Vec<WorkflowRunId> = rows
            .into_iter()
            .take(limit)
            .map(|r| WorkflowRunId(r.run_id))
            .collect();

        let next_cursor = if has_more {
            run_ids.last().map(|r| r.0.clone())
        } else {
            None
        };

        Ok((run_ids, next_cursor))
    }
}
