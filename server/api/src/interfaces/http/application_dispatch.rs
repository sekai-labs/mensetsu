use chrono::Utc;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    contexts::identity::domain::UserId,
    interfaces::{
        grpc::CreateRunPayload,
        http::{
            applications::clean_id,
            workspaces::{resolve_llm_config, AppState},
        },
    },
    shared::{error::AppError, Result},
};

const DISPATCH_BATCH_SIZE: i64 = 20;
const DISPATCH_LEASE_SECONDS: i64 = 90;
const DISPATCH_MAX_BACKOFF_SECONDS: i64 = 300;

#[derive(Debug, Clone, Deserialize)]
struct DispatchRow {
    id: String,
    workspace_id: String,
    user_id: String,
    application_id: String,
    attempts: i64,
}

struct ClaimedDispatch {
    row: DispatchRow,
    token: String,
}

#[derive(Debug, Deserialize)]
struct DispatchApplicationRow {
    company_name: String,
    role_title: String,
    job_description: String,
    source_url: String,
    resume_upload_id: String,
    resume_filename: Option<String>,
    status: String,
}

pub async fn drain_once(state: &AppState) -> Result<usize> {
    drain_dispatch_batch(state, None).await
}

pub async fn drain_once_for_workspace(state: &AppState, workspace_id: &str) -> Result<usize> {
    drain_dispatch_batch(state, Some(workspace_id)).await
}

async fn drain_dispatch_batch(state: &AppState, workspace_id: Option<&str>) -> Result<usize> {
    let query = if workspace_id.is_some() {
        "SELECT id, workspace_id, user_id, application_id, attempts, due_at FROM application_dispatch WHERE workspace_id=type::record('workspace',$workspace_id) AND ((state='pending' AND due_at<=time::now()) OR (state='leased' AND lease_until<=time::now())) ORDER BY due_at ASC LIMIT $limit;"
    } else {
        "SELECT id, workspace_id, user_id, application_id, attempts, due_at FROM application_dispatch WHERE (state='pending' AND due_at<=time::now()) OR (state='leased' AND lease_until<=time::now()) ORDER BY due_at ASC LIMIT $limit;"
    };
    let mut request = state
        .db
        .client
        .query(query)
        .bind(("limit", DISPATCH_BATCH_SIZE));
    if let Some(workspace_id) = workspace_id {
        request = request.bind(("workspace_id", clean_id(workspace_id)));
    }
    let mut result = request
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let values: Vec<serde_json::Value> = result
        .take(0)
        .map_err(|e| AppError::Database(e.to_string()))?;
    let rows = values
        .into_iter()
        .map(serde_json::from_value)
        .collect::<std::result::Result<Vec<DispatchRow>, _>>()
        .map_err(|e| AppError::Database(format!("Malformed application dispatch: {e}")))?;
    let mut delivered = 0;
    for row in rows {
        let Some(claim) = claim_dispatch(state, row).await? else {
            continue;
        };
        match deliver_claim(state, &claim).await {
            Ok(true) => delivered += 1,
            Ok(false) => {}
            Err(error) => {
                tracing::warn!(
                    dispatch_id = %clean_id(&claim.row.id),
                    error = %error,
                    "Application dispatch delivery failed"
                );
                record_failure(state, &claim, &error).await?;
            }
        }
    }
    Ok(delivered)
}

pub fn spawn_dispatcher(state: AppState) {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(std::time::Duration::from_secs(5));
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        loop {
            ticker.tick().await;
            match drain_once(&state).await {
                Ok(0) => {}
                Ok(delivered) => {
                    tracing::info!(
                        delivered = delivered,
                        "Application dispatch delivered pending work"
                    );
                }
                Err(error) => {
                    tracing::warn!(error = %error, "Application dispatch drain failed");
                }
            }
        }
    });
}

async fn claim_dispatch(state: &AppState, row: DispatchRow) -> Result<Option<ClaimedDispatch>> {
    let token = Uuid::new_v4().to_string();
    let lease_until = Utc::now() + chrono::Duration::seconds(DISPATCH_LEASE_SECONDS);
    let mut result = state
        .db
        .client
        .query(
            "UPDATE type::record('application_dispatch',$id) SET state='leased', lease_until=$lease_until, lease_token=$lease_token, updated_at=time::now() WHERE (state='pending' AND due_at<=time::now()) OR (state='leased' AND lease_until<=time::now()) RETURN AFTER;",
        )
        .bind(("id", clean_id(&row.id)))
        .bind(("lease_until", lease_until))
        .bind(("lease_token", token.clone()))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let claimed: Vec<serde_json::Value> = result
        .take(0)
        .map_err(|e| AppError::Database(e.to_string()))?;
    if claimed.is_empty() {
        return Ok(None);
    }
    Ok(Some(ClaimedDispatch { row, token }))
}

async fn deliver_claim(state: &AppState, claim: &ClaimedDispatch) -> Result<bool> {
    let app_id = clean_id(&claim.row.application_id);
    let uid = clean_id(&claim.row.user_id);
    let workspace_id = clean_id(&claim.row.workspace_id);
    let mut result = state
        .db
        .client
        .query(
            "SELECT company_name, role_title, job_description, source_url, resume_upload_id, resume_upload_id.filename AS resume_filename, status FROM type::record('job_application',$app_id) WHERE user_id=type::record('user',$uid) LIMIT 1;",
        )
        .bind(("app_id", app_id))
        .bind(("uid", uid.clone()))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let values: Vec<serde_json::Value> = result
        .take(0)
        .map_err(|e| AppError::Database(e.to_string()))?;
    let application: DispatchApplicationRow = values
        .into_iter()
        .next()
        .ok_or_else(|| AppError::Conflict("Dispatch application not found".into()))
        .and_then(|value| {
            serde_json::from_value(value)
                .map_err(|e| AppError::Database(format!("Malformed dispatch application: {e}")))
        })?;
    if application.status != "submitted" {
        return Err(AppError::Conflict(
            "Dispatch application is not submitted".into(),
        ));
    }
    let payload = CreateRunPayload {
        workspace_id: workspace_id.clone(),
        user_id: uid.clone(),
        workflow_id: "interview-preparation-hybrid".into(),
        company_name: application.company_name,
        role_title: application.role_title,
        company_urls: vec![application.source_url.clone()],
        resume_text: String::new(),
        resume_upload_id: Some(clean_id(&application.resume_upload_id)),
        resume_filename: application.resume_filename,
        job_description: application.job_description,
        jd_url: Some(application.source_url),
        seniority: None,
        user_prompt: None,
        workflow_mode: Some("hybrid".into()),
        llm_config: resolve_llm_config(state, &UserId(uid)).await?,
    };
    let run = state
        .grpc_client
        .create_run(&payload, Some(workspace_id.as_str()))
        .await?;
    mark_delivered(state, claim, &run.id).await
}

async fn mark_delivered(state: &AppState, claim: &ClaimedDispatch, run_id: &str) -> Result<bool> {
    let mut result = state
        .db
        .client
        .query(
            "UPDATE type::record('application_dispatch',$id) SET state='delivered', run_id=$run_id, lease_until=NONE, lease_token=NONE, last_error=NONE, updated_at=time::now() WHERE state='leased' AND lease_token=$lease_token RETURN AFTER;",
        )
        .bind(("id", clean_id(&claim.row.id)))
        .bind(("run_id", run_id.to_string()))
        .bind(("lease_token", claim.token.clone()))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let rows: Vec<serde_json::Value> = result
        .take(0)
        .map_err(|e| AppError::Database(e.to_string()))?;
    if rows.is_empty() {
        tracing::warn!(
            dispatch_id = %clean_id(&claim.row.id),
            "Application dispatch lease was lost before delivery acknowledgement"
        );
        return Ok(false);
    }
    Ok(true)
}

async fn record_failure(state: &AppState, claim: &ClaimedDispatch, error: &AppError) -> Result<()> {
    let attempts = claim.row.attempts.saturating_add(1);
    let exponent = attempts.clamp(0, 6) as u32;
    let backoff = (5_i64 * 2_i64.pow(exponent)).min(DISPATCH_MAX_BACKOFF_SECONDS);
    let due_at = Utc::now() + chrono::Duration::seconds(backoff);
    let last_error: String = error.to_string().chars().take(1024).collect();
    let mut result = state
        .db
        .client
        .query(
            "UPDATE type::record('application_dispatch',$id) SET state='pending', attempts=$attempts, due_at=$due_at, lease_until=NONE, lease_token=NONE, last_error=$last_error, updated_at=time::now() WHERE state='leased' AND lease_token=$lease_token RETURN AFTER;",
        )
        .bind(("id", clean_id(&claim.row.id)))
        .bind(("attempts", attempts))
        .bind(("due_at", due_at))
        .bind(("last_error", last_error))
        .bind(("lease_token", claim.token.clone()))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let rows: Vec<serde_json::Value> = result
        .take(0)
        .map_err(|e| AppError::Database(e.to_string()))?;
    if rows.is_empty() {
        tracing::warn!(
            dispatch_id = %clean_id(&claim.row.id),
            "Application dispatch lease was lost before retry scheduling"
        );
    }
    Ok(())
}
