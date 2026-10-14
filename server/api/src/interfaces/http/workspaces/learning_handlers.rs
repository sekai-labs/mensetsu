use axum::{
    extract::{Path, State},
    Json,
};

use super::auth_check::verify_workspace_owner;
use super::AppState;
use crate::contexts::knowledge::domain::{KnowledgeEdge, KnowledgeGraphResponse, KnowledgeSkillNode};
use crate::contexts::learning::domain::LearningUnit;
use crate::contexts::practice::domain::InterviewQuestion;
use crate::interfaces::http::extractors::authenticated_user::AuthenticatedUser;
use crate::shared::error::AppError;
use crate::shared::record_id::normalize_record_fields;

fn parse_rows<T: serde::de::DeserializeOwned>(table: &str, rows: Vec<serde_json::Value>) -> Vec<T> {
    rows.into_iter()
        .filter_map(|row| {
            let record_id = row.get("id").and_then(|v| v.as_str()).unwrap_or("<missing id>").to_string();
            serde_json::from_value::<T>(row)
                .map_err(|e| tracing::warn!(table, record_id = %record_id, error = %e, "skipping row that violates the response contract"))
                .ok()
        })
        .collect()
}

#[derive(serde::Deserialize)]
struct SkillRow {
    name: String,
    slug: String,
    category: String,
    candidate_status: String,
    importance: String,
}

#[derive(serde::Deserialize)]
struct SkillRelationRow {
    source: String,
    target: String,
    relation: String,
}

pub async fn get_workspace_match(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(id): Path<String>,
) -> std::result::Result<Json<serde_json::Value>, AppError> {
    let clean_id = id.replace("workspace:", "").replace('`', "");
    verify_workspace_owner(&state.db.client, &clean_id, &auth_user.user_id.0).await?;
    let db = &state.db.client;
    let mut res = db
        .query("SELECT * FROM match_analysis WHERE workspace_id = type::record('workspace', $id) OR workspace_id = $id;")
        .bind(("id", clean_id.clone()))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let item: Option<serde_json::Value> = res.take(0).map_err(|e| AppError::Database(e.to_string()))?;
    if let Some(mut match_data) = item {
        normalize_record_fields(&mut match_data, &["id", "workspace_id"]);
        Ok(Json(match_data))
    } else {
        Ok(Json(serde_json::json!({
            "workspace_id": id,
            "overall_match_ratio": 0.0,
            "strengths": [],
            "gaps": [],
            "risks": [],
            "wording_improvements": []
        })))
    }
}

pub async fn get_workspace_knowledge(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(id): Path<String>,
) -> std::result::Result<Json<KnowledgeGraphResponse>, AppError> {
    verify_workspace_owner(&state.db.client, &id, &auth_user.user_id.0).await?;
    let db = &state.db.client;
    let mut res_skills = db
        .query("SELECT * FROM skill WHERE workspace_id = type::record('workspace', $id) OR workspace_id = $id;")
        .bind(("id", id.clone()))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let skills: Vec<serde_json::Value> = res_skills.take(0).map_err(|e| AppError::Database(e.to_string()))?;
    let nodes: Vec<KnowledgeSkillNode> = parse_rows::<SkillRow>("skill", skills)
        .into_iter()
        .map(|s| KnowledgeSkillNode {
            id: s.slug.clone(),
            name: s.name,
            slug: s.slug,
            category: s.category,
            candidate_status: s.candidate_status,
            importance: s.importance,
        })
        .collect();

    let mut res_edges = db
        .query("SELECT * FROM skill_relation WHERE workspace_id = type::record('workspace', $id) OR workspace_id = $id;")
        .bind(("id", id.clone()))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let raw_edges: Vec<serde_json::Value> = res_edges.take(0).map_err(|e| AppError::Database(e.to_string()))?;
    let edges: Vec<KnowledgeEdge> = parse_rows::<SkillRelationRow>("skill_relation", raw_edges)
        .into_iter()
        .map(|e| KnowledgeEdge { source: e.source, target: e.target, relation: e.relation })
        .collect();

    Ok(Json(KnowledgeGraphResponse { nodes, edges }))
}

pub async fn get_workspace_learning(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(id): Path<String>,
) -> std::result::Result<Json<Vec<LearningUnit>>, AppError> {
    verify_workspace_owner(&state.db.client, &id, &auth_user.user_id.0).await?;
    let db = &state.db.client;
    let mut res = db
        .query("SELECT * FROM learning_unit WHERE workspace_id = type::record('workspace', $id);")
        .bind(("id", id.clone()))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let mut units: Vec<serde_json::Value> = res.take(0).map_err(|e| AppError::Database(e.to_string()))?;
    for u in &mut units {
        normalize_record_fields(u, &["id", "workspace_id"]);
    }
    Ok(Json(parse_rows::<LearningUnit>("learning_unit", units)))
}

#[utoipa::path(
    put,
    path = "/api/v1/workspaces/{id}/learning/{lesson_id}/toggle",
    responses(
        (status = 200, description = "Learning unit completion toggled", body = serde_json::Value),
        (status = 404, description = "Learning unit not found")
    )
)]
pub async fn toggle_learning_unit_completed(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path((id, lesson_id)): Path<(String, String)>,
) -> std::result::Result<Json<serde_json::Value>, AppError> {
    verify_workspace_owner(&state.db.client, &id, &auth_user.user_id.0).await?;
    let db = &state.db.client;

    let clean_id = lesson_id.replace("learning_unit:", "").replace('`', "");
    let mut res = db
        .query("SELECT * FROM type::record('learning_unit', $lid) WHERE workspace_id = type::record('workspace', $wid);")
        .bind(("lid", clean_id.clone()))
        .bind(("wid", id.clone()))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let item: Option<serde_json::Value> = res.take(0).unwrap_or_default();
    let current_completed = item
        .as_ref()
        .and_then(|v| v.get("completed"))
        .and_then(|b| b.as_bool())
        .unwrap_or(false);

    let new_completed = !current_completed;

    let _ = db
        .query("UPDATE type::record('learning_unit', $lid) SET completed = $completed;")
        .bind(("lid", clean_id))
        .bind(("completed", new_completed))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(Json(serde_json::json!({
        "lesson_id": lesson_id,
        "completed": new_completed
    })))
}

pub async fn get_workspace_questions(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(id): Path<String>,
) -> std::result::Result<Json<Vec<InterviewQuestion>>, AppError> {
    let clean_id = id.replace("workspace:", "").replace('`', "");
    verify_workspace_owner(&state.db.client, &clean_id, &auth_user.user_id.0).await?;
    let db = &state.db.client;
    let mut res = db
        .query("SELECT * FROM interview_question WHERE workspace_id = type::record('workspace', $id) OR workspace_id = $id;")
        .bind(("id", clean_id.clone()))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let mut q_list: Vec<serde_json::Value> = res.take(0).map_err(|e| AppError::Database(e.to_string()))?;
    for q in &mut q_list {
        normalize_record_fields(q, &["id", "workspace_id"]);
    }
    Ok(Json(parse_rows::<InterviewQuestion>("interview_question", q_list)))
}
