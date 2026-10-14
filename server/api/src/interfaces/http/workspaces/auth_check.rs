use crate::shared::error::AppError;

pub async fn verify_workspace_owner(
    db: &crate::contexts::workspace::infrastructure::db::Db,
    ws_id: &str,
    user_id: &str,
) -> std::result::Result<(), AppError> {
    let clean_id = ws_id.replace("workspace:", "").replace('`', "");
    let mut res = db
        .query("SELECT * FROM type::record('workspace', $id);")
        .bind(("id", clean_id.clone()))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let mut item: Option<serde_json::Value> = res.take(0).map_err(|e| AppError::Database(e.to_string()))?;
    if item.is_none() {
        let mut res_raw = db
            .query("SELECT * FROM workspace WHERE id = type::record('workspace', $id) OR id = $id;")
            .bind(("id", clean_id.clone()))
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        item = res_raw.take(0).unwrap_or_default();
    }
    let w = item.ok_or_else(|| AppError::NotFound(format!("Workspace {ws_id} not found")))?;
    let owner = w["user_id"].as_str().unwrap_or("").replace("user:", "").replace('`', "");
    if owner != user_id {
        return Err(AppError::Forbidden("Access denied: You do not own this workspace".into()));
    }
    Ok(())
}
