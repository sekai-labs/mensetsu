use std::collections::HashMap;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::Digest;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::{
    contexts::workspace::domain::WorkspaceStatus,
    interfaces::http::{
        applications::{
            application_response_from_value, clean_id, completed_upload_filename,
            load_owned_application, validate_email, ApplicationMethod, ApplicationStatus,
            JobApplicationResponseDto,
        },
        extractors::authenticated_user::AuthenticatedUser,
        workspaces::AppState,
    },
    shared::{error::AppError, Result},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SavedJobStatus {
    AwaitingApproval,
    Approved,
    Dismissed,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SavedJobResponseDto {
    pub id: String,
    pub source_url: String,
    pub canonical_url: String,
    pub source_site: String,
    pub status: SavedJobStatus,
    pub role_title: Option<String>,
    pub company_name: Option<String>,
    pub job_description: Option<String>,
    pub location: Option<String>,
    pub application_url: Option<String>,
    pub contact_email: Option<String>,
    pub job_application_id: Option<String>,
    #[serde(default)]
    pub application_status: Option<ApplicationStatus>,
    #[serde(default)]
    pub workspace_status: Option<WorkspaceStatus>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SavedJobsPageResponseDto {
    pub items: Vec<SavedJobResponseDto>,
    #[serde(default)]
    pub next_cursor: Option<String>,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct SavedJobsQuery {
    pub limit: Option<u32>,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateSavedJobRequestDto {
    pub source_url: String,
    pub role_title: Option<String>,
    pub company_name: Option<String>,
    pub job_description: Option<String>,
    pub location: Option<String>,
    pub application_url: Option<String>,
    pub contact_email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct PrepareSavedJobRequestDto {
    pub resume_upload_id: String,
    pub application_method: ApplicationMethod,
    pub contact_email: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SavedJobSite {
    Linkedin,
    Topcv,
}

impl SavedJobSite {
    fn as_str(self) -> &'static str {
        match self {
            Self::Linkedin => "linkedin",
            Self::Topcv => "topcv",
        }
    }

    fn hosts(self) -> &'static [&'static str] {
        match self {
            Self::Linkedin => &["linkedin.com", "www.linkedin.com"],
            Self::Topcv => &["topcv.vn", "www.topcv.vn"],
        }
    }
}

fn detect_site(raw_url: &str) -> Result<SavedJobSite> {
    let url = reqwest::Url::parse(raw_url)
        .map_err(|_| AppError::Validation("Invalid source URL".into()))?;
    let host = url.host_str().unwrap_or("").to_ascii_lowercase();
    if SavedJobSite::Linkedin.hosts().contains(&host.as_str()) {
        return Ok(SavedJobSite::Linkedin);
    }
    if SavedJobSite::Topcv.hosts().contains(&host.as_str()) {
        return Ok(SavedJobSite::Topcv);
    }
    Err(AppError::Validation(
        "Source URL must be from LinkedIn or TopCV".into(),
    ))
}

fn canonical_saved_url(site: SavedJobSite, raw: &str) -> Result<String> {
    if raw.chars().count() > 2048 {
        return Err(AppError::Validation(
            "Source URL exceeds 2048 characters".into(),
        ));
    }
    let mut url =
        reqwest::Url::parse(raw).map_err(|_| AppError::Validation("Invalid source URL".into()))?;
    let host = url.host_str().unwrap_or("").to_ascii_lowercase();
    if url.scheme() != "https"
        || !url.username().is_empty()
        || url.password().is_some()
        || url.port().is_some()
        || !site.hosts().contains(&host.as_str())
    {
        return Err(AppError::Validation("Unsupported source URL".into()));
    }
    let parts: Vec<_> = url
        .path_segments()
        .into_iter()
        .flatten()
        .filter(|p| !p.is_empty())
        .collect();
    let stable = match site {
        SavedJobSite::Linkedin => {
            if parts.len() < 3
                || parts[0] != "jobs"
                || parts[1] != "view"
                || !parts[2].chars().all(|c| c.is_ascii_digit())
            {
                return Err(AppError::Validation(
                    "LinkedIn URL must use /jobs/view/{id}".into(),
                ));
            }
            format!("/jobs/view/{}", parts[2])
        }
        SavedJobSite::Topcv => {
            let first = parts.first().copied().unwrap_or("");
            if let Some(slug) = first.strip_prefix("viec-lam-") {
                if slug.is_empty() {
                    return Err(AppError::Validation(
                        "TopCV URL must contain a stable job path or ID".into(),
                    ));
                }
                format!("/{first}")
            } else if first == "viec-lam" {
                let job = parts.get(1).copied().filter(|id| !id.is_empty());
                let job = job.ok_or_else(|| {
                    AppError::Validation("TopCV URL must contain a stable job path or ID".into())
                })?;
                format!("/viec-lam/{job}")
            } else {
                return Err(AppError::Validation(
                    "TopCV URL must contain a stable job path or ID".into(),
                ));
            }
        }
    };
    let normalized_host = match site {
        SavedJobSite::Linkedin => "linkedin.com",
        SavedJobSite::Topcv => "topcv.vn",
    };
    url.set_host(Some(normalized_host))
        .map_err(|_| AppError::Validation("Invalid canonical host".into()))?;
    url.set_path(&stable);
    url.set_query(None);
    url.set_fragment(None);
    Ok(url.to_string().trim_end_matches('/').to_string())
}

fn is_public_https_host(host: &str) -> bool {
    let normalized = host.trim_end_matches('.').to_ascii_lowercase();
    let ip_candidate = normalized
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .unwrap_or(&normalized);
    !normalized.is_empty()
        && normalized.contains('.')
        && ip_candidate.parse::<std::net::IpAddr>().is_err()
        && !normalized.ends_with(".localhost")
        && !normalized.ends_with(".local")
        && !normalized.ends_with(".internal")
}

fn validate_application_url(raw: &str) -> Result<()> {
    if raw.chars().count() > 2048 {
        return Err(AppError::Validation(
            "application_url exceeds 2048 characters".into(),
        ));
    }
    let url = reqwest::Url::parse(raw)
        .map_err(|_| AppError::Validation("Invalid application_url".into()))?;
    let host = url.host_str().unwrap_or("");
    if url.scheme() != "https"
        || !is_public_https_host(host)
        || !url.username().is_empty()
        || url.password().is_some()
        || url.port().is_some()
    {
        return Err(AppError::Validation(
            "application_url must be a credential-free public HTTPS URL".into(),
        ));
    }
    Ok(())
}

fn build_prepared_fields(
    first_name: String,
    last_name: String,
    user_email: String,
    profile_obj: Option<&serde_json::Value>,
) -> serde_json::Value {
    let mut fields = serde_json::Map::new();
    fields.insert("first_name".into(), first_name.into());
    fields.insert("last_name".into(), last_name.into());
    fields.insert("email".into(), user_email.into());
    for key in [
        "phone",
        "location",
        "linkedin_url",
        "github_url",
        "portfolio_url",
    ] {
        if let Some(value) = profile_obj
            .and_then(|profile| profile.get(key))
            .and_then(|value| value.as_str())
        {
            fields.insert(key.to_string(), value.into());
        }
    }
    serde_json::Value::Object(fields)
}

fn saved_job_from_value(value: serde_json::Value) -> Result<SavedJobResponseDto> {
    let id_raw = value
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let id = clean_id(&id_raw);
    let job_application_id = value
        .get("job_application_id")
        .and_then(|v| v.as_str())
        .map(clean_id);
    let mut dto: SavedJobResponseDto = serde_json::from_value(value)
        .map_err(|e| AppError::Database(format!("Malformed saved job: {e}")))?;
    dto.id = id;
    dto.job_application_id = job_application_id;
    Ok(dto)
}

fn saved_job_status_value(status: SavedJobStatus) -> &'static str {
    match status {
        SavedJobStatus::AwaitingApproval => "awaiting_approval",
        SavedJobStatus::Approved => "approved",
        SavedJobStatus::Dismissed => "dismissed",
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MergedSavedJobFacts {
    source_url: String,
    role_title: Option<String>,
    company_name: Option<String>,
    job_description: Option<String>,
    location: Option<String>,
    application_url: Option<String>,
    contact_email: Option<String>,
}

impl MergedSavedJobFacts {
    fn differs_from(&self, existing: &SavedJobResponseDto) -> bool {
        self.source_url != existing.source_url
            || self.role_title != existing.role_title
            || self.company_name != existing.company_name
            || self.job_description != existing.job_description
            || self.location != existing.location
            || self.application_url != existing.application_url
            || self.contact_email != existing.contact_email
    }
}

fn nonempty_saved_job_fact(value: &Option<String>) -> Option<String> {
    value
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn merge_saved_job_facts(
    existing: &SavedJobResponseDto,
    incoming: &CreateSavedJobRequestDto,
) -> (MergedSavedJobFacts, bool) {
    let source_url = {
        let value = incoming.source_url.trim();
        if value.is_empty() {
            existing.source_url.clone()
        } else {
            value.to_string()
        }
    };
    let merged = MergedSavedJobFacts {
        source_url,
        role_title: nonempty_saved_job_fact(&incoming.role_title)
            .or_else(|| existing.role_title.clone()),
        company_name: nonempty_saved_job_fact(&incoming.company_name)
            .or_else(|| existing.company_name.clone()),
        job_description: nonempty_saved_job_fact(&incoming.job_description)
            .or_else(|| existing.job_description.clone()),
        location: nonempty_saved_job_fact(&incoming.location).or_else(|| existing.location.clone()),
        application_url: nonempty_saved_job_fact(&incoming.application_url)
            .or_else(|| existing.application_url.clone()),
        contact_email: nonempty_saved_job_fact(&incoming.contact_email)
            .or_else(|| existing.contact_email.clone()),
    };
    let material_changed = merged.role_title != existing.role_title
        || merged.company_name != existing.company_name
        || merged.job_description != existing.job_description
        || merged.location != existing.location
        || merged.application_url != existing.application_url
        || merged.contact_email != existing.contact_email;
    (merged, material_changed)
}

fn duplicate_save_transition(
    current: SavedJobStatus,
    material_changed: bool,
    application: Option<ApplicationStatus>,
) -> Result<(SavedJobStatus, bool)> {
    let terminal = matches!(
        application,
        Some(
            ApplicationStatus::Submitting
                | ApplicationStatus::Submitted
                | ApplicationStatus::Unknown
        )
    );
    let reset = current == SavedJobStatus::Dismissed
        || (current == SavedJobStatus::Approved && material_changed);
    if (material_changed || reset) && terminal {
        return Err(AppError::Conflict(
            "Saved job cannot be changed because its linked application is submitted or in progress"
                .into(),
        ));
    }
    let mark_review = matches!(
        application,
        Some(
            ApplicationStatus::Prepared
                | ApplicationStatus::Approved
                | ApplicationStatus::NeedsReview
        )
    );
    if reset {
        return Ok((SavedJobStatus::AwaitingApproval, mark_review));
    }
    if material_changed {
        return Ok((current, mark_review));
    }
    Ok((current, false))
}

fn transaction_result(mut response: surrealdb::IndexedResults, context: &str) -> Result<()> {
    if let Some((index, error)) = response.take_errors().into_iter().next() {
        return Err(AppError::Conflict(format!(
            "{context} (statement {index}): {error}"
        )));
    }
    Ok(())
}

async fn load_owned_saved_job(
    state: &AppState,
    id: &str,
    uid: &str,
) -> Result<SavedJobResponseDto> {
    let clean = clean_id(id);
    Uuid::parse_str(&clean).map_err(|_| AppError::Validation("Invalid saved job ID".into()))?;
    let uid_clean = clean_id(uid);
    let mut r = state
        .db
        .client
        .query("SELECT id, source_url, canonical_url, source_site, status, role_title, company_name, job_description, location, application_url, contact_email, job_application_id, created_at, updated_at FROM type::record('saved_job',$id) WHERE user_id=type::record('user',$uid);")
        .bind(("id", clean.clone()))
        .bind(("uid", uid_clean.clone()))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let rows: Vec<serde_json::Value> = r.take(0).map_err(|e| AppError::Database(e.to_string()))?;
    let value = rows
        .into_iter()
        .next()
        .ok_or_else(|| AppError::NotFound("Saved job not found".into()))?;
    let dto = saved_job_from_value(value)?;
    let mut jobs = [dto];
    attach_application_lifecycle(state, &uid_clean, &mut jobs).await?;
    jobs.into_iter()
        .next()
        .ok_or_else(|| AppError::NotFound("Saved job not found".into()))
}

async fn attach_application_lifecycle(
    state: &AppState,
    uid: &str,
    jobs: &mut [SavedJobResponseDto],
) -> Result<()> {
    let mut ids: Vec<String> = Vec::new();
    for job in jobs.iter() {
        if let Some(app_id) = job.job_application_id.as_deref() {
            let clean = clean_id(app_id);
            if Uuid::parse_str(&clean).is_ok() && !ids.contains(&clean) {
                ids.push(clean);
            }
        }
    }
    if ids.is_empty() {
        return Ok(());
    }
    let placeholders = ids
        .iter()
        .enumerate()
        .map(|(index, _)| format!("type::record('job_application',$id{index})"))
        .collect::<Vec<_>>()
        .join(",");
    let query = format!(
        "SELECT id, status, workspace_id.status AS workspace_status FROM job_application WHERE user_id=type::record('user',$uid) AND id IN [{placeholders}];"
    );
    let mut request = state.db.client.query(query).bind(("uid", clean_id(uid)));
    for (index, id) in ids.iter().enumerate() {
        request = request.bind((format!("id{index}"), id.clone()));
    }
    let mut r = request
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let rows: Vec<serde_json::Value> = r.take(0).map_err(|e| AppError::Database(e.to_string()))?;
    let mut statuses: HashMap<String, (ApplicationStatus, Option<WorkspaceStatus>)> =
        HashMap::new();
    for value in rows {
        let Some(id) = value.get("id").and_then(|v| v.as_str()).map(clean_id) else {
            continue;
        };
        let status = value
            .get("status")
            .cloned()
            .ok_or_else(|| AppError::Database("Malformed application status".into()))?;
        let status: ApplicationStatus = serde_json::from_value(status)
            .map_err(|e| AppError::Database(format!("Malformed application status: {e}")))?;
        let workspace_status = value
            .get("workspace_status")
            .cloned()
            .map(serde_json::from_value::<Option<WorkspaceStatus>>)
            .transpose()
            .map_err(|e| AppError::Database(format!("Malformed workspace status: {e}")))?
            .flatten();
        statuses.insert(id, (status, workspace_status));
    }
    for job in jobs.iter_mut() {
        if let Some(app_id) = job.job_application_id.as_deref() {
            if let Some((status, workspace_status)) = statuses.get(&clean_id(app_id)) {
                job.application_status = Some(*status);
                job.workspace_status = *workspace_status;
            }
        }
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
struct LinkedApplicationRow {
    id: String,
    status: ApplicationStatus,
}

async fn load_linked_application(
    state: &AppState,
    uid: &str,
    app_id: &str,
) -> Result<Option<(String, ApplicationStatus)>> {
    let app = clean_id(app_id);
    Uuid::parse_str(&app)
        .map_err(|_| AppError::Database("Malformed linked application ID".into()))?;
    let mut r = state
        .db
        .client
        .query("SELECT id, status FROM type::record('job_application',$id) WHERE user_id=type::record('user',$uid);")
        .bind(("id", app))
        .bind(("uid", clean_id(uid)))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let rows: Vec<serde_json::Value> = r.take(0).map_err(|e| AppError::Database(e.to_string()))?;
    rows.into_iter()
        .next()
        .map(|value| {
            let row: LinkedApplicationRow = serde_json::from_value(value)
                .map_err(|e| AppError::Database(format!("Malformed application status: {e}")))?;
            Ok((clean_id(&row.id), row.status))
        })
        .transpose()
}

async fn load_canonical_application(
    state: &AppState,
    uid: &str,
    canonical_url: &str,
) -> Result<Option<(String, ApplicationStatus)>> {
    let mut r = state
        .db
        .client
        .query("SELECT id, status FROM job_application WHERE user_id=type::record('user',$uid) AND canonical_url=$url LIMIT 1;")
        .bind(("uid", clean_id(uid)))
        .bind(("url", canonical_url.to_string()))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let rows: Vec<serde_json::Value> = r.take(0).map_err(|e| AppError::Database(e.to_string()))?;
    rows.into_iter()
        .next()
        .map(|value| {
            let row: LinkedApplicationRow = serde_json::from_value(value)
                .map_err(|e| AppError::Database(format!("Malformed application status: {e}")))?;
            Ok((clean_id(&row.id), row.status))
        })
        .transpose()
}

async fn load_canonical_application_response(
    state: &AppState,
    uid: &str,
    canonical_url: &str,
) -> Result<Option<JobApplicationResponseDto>> {
    let mut r = state
        .db
        .client
        .query("SELECT id, site, canonical_url, source_url, company_name, role_title, job_description, resume_upload_id, resume_upload_id.filename AS resume_filename, prepared_fields, revision, status, application_method, contact_email, workspace_id, confirmation_url, created_at, updated_at FROM job_application WHERE user_id=type::record('user',$uid) AND canonical_url=$url LIMIT 1;")
        .bind(("uid", clean_id(uid)))
        .bind(("url", canonical_url.to_string()))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let rows: Vec<serde_json::Value> = r.take(0).map_err(|e| AppError::Database(e.to_string()))?;
    rows.into_iter()
        .next()
        .map(application_response_from_value)
        .transpose()
}

async fn merge_existing_saved_job(
    state: &AppState,
    id: &str,
    uid: &str,
    incoming: &CreateSavedJobRequestDto,
) -> Result<SavedJobResponseDto> {
    let existing = load_owned_saved_job(state, id, uid).await?;
    let (merged, material_changed) = merge_saved_job_facts(&existing, incoming);
    let linked = if let Some(app_id) = existing.job_application_id.as_deref() {
        load_linked_application(state, uid, app_id).await?
    } else {
        None
    };
    let target = if linked.is_some() {
        linked
    } else {
        load_canonical_application(state, uid, &existing.canonical_url).await?
    };
    let (target_app_id, application_status) = match target {
        Some((app_id, status)) => (Some(app_id), Some(status)),
        None => (None, None),
    };
    let (next_status, mark_review) =
        duplicate_save_transition(existing.status, material_changed, application_status)?;
    if !merged.differs_from(&existing) && next_status == existing.status && !mark_review {
        return Ok(existing);
    }
    let review_app_id = if mark_review { target_app_id } else { None };
    let query = "
        BEGIN TRANSACTION;
        LET $saved = (UPDATE type::record('saved_job',$id) SET
            source_url=$source_url, role_title=$role_title, company_name=$company_name,
            job_description=$job_description, location=$location, application_url=$application_url,
            contact_email=$contact_email, status=$status, updated_at=time::now()
            WHERE user_id=type::record('user',$uid) AND status=$previous_status RETURN AFTER);
        IF array::len($saved) != 1 { THROW 'saved_job_state_conflict'; };
        LET $reviewed = IF $mark_review = true {
            (UPDATE type::record('job_application',$app_id) SET status='needs_review', updated_at=time::now()
                WHERE user_id=type::record('user',$uid) AND status IN ['prepared','approved','needs_review'] RETURN AFTER)
        } ELSE { [] };
        IF $mark_review = true AND array::len($reviewed) != 1 { THROW 'application_state_conflict'; };
        COMMIT TRANSACTION;
    ";
    state
        .db
        .client
        .query(query)
        .bind(("id", clean_id(id)))
        .bind(("uid", clean_id(uid)))
        .bind(("source_url", merged.source_url))
        .bind(("role_title", merged.role_title))
        .bind(("company_name", merged.company_name))
        .bind(("job_description", merged.job_description))
        .bind(("location", merged.location))
        .bind(("application_url", merged.application_url))
        .bind(("contact_email", merged.contact_email))
        .bind(("status", saved_job_status_value(next_status)))
        .bind(("previous_status", saved_job_status_value(existing.status)))
        .bind(("mark_review", mark_review))
        .bind(("app_id", review_app_id))
        .await
        .map_err(|e| AppError::Database(e.to_string()))
        .and_then(|response| transaction_result(response, "Saved job merge conflict"))?;
    load_owned_saved_job(state, id, uid).await
}

fn evaluate_prepare_idempotency(
    existing: &JobApplicationResponseDto,
    requested_upload_id: &str,
    requested_method: ApplicationMethod,
    requested_contact_email: Option<&str>,
) -> Result<(), AppError> {
    let clean_upload = clean_id(requested_upload_id);
    let resume_changed = clean_id(&existing.resume_upload_id) != clean_upload;
    let existing_method = existing
        .application_method
        .unwrap_or(ApplicationMethod::Web);
    let method_changed = existing_method != requested_method;
    let recipient_changed = requested_method == ApplicationMethod::Email && {
        let existing_recipient = existing
            .contact_email
            .as_deref()
            .map(str::trim)
            .filter(|email| validate_email(email).is_ok());
        let requested_recipient = requested_contact_email
            .map(str::trim)
            .filter(|email| validate_email(email).is_ok());
        existing_recipient != requested_recipient
    };

    if resume_changed || method_changed || recipient_changed {
        return Err(AppError::Conflict(
            "Application already exists with different resume, method, or recipient".into(),
        ));
    }
    Ok(())
}

fn approve_transition(status: SavedJobStatus) -> Result<bool> {
    match status {
        SavedJobStatus::Approved => Ok(false),
        SavedJobStatus::AwaitingApproval => Ok(true),
        SavedJobStatus::Dismissed => Err(AppError::Conflict(
            "Saved job cannot be approved from current status".into(),
        )),
    }
}

fn dismiss_transition(status: SavedJobStatus) -> Result<bool> {
    match status {
        SavedJobStatus::Dismissed => Ok(false),
        SavedJobStatus::AwaitingApproval | SavedJobStatus::Approved => Ok(true),
    }
}

fn require_approved_for_prepare(status: SavedJobStatus) -> Result<()> {
    if status == SavedJobStatus::Approved {
        Ok(())
    } else {
        Err(AppError::Conflict(
            "Saved job must be approved before preparing".into(),
        ))
    }
}

fn resolve_prepare_contact_email(
    method: ApplicationMethod,
    request_email: Option<&str>,
    saved_email: Option<&str>,
) -> Result<Option<String>> {
    let req_clean = request_email.map(str::trim).filter(|e| !e.is_empty());
    let saved_clean = saved_email.map(str::trim).filter(|e| !e.is_empty());

    let valid_saved = saved_clean.filter(|e| validate_email(e).is_ok());
    let valid_request = req_clean.filter(|e| validate_email(e).is_ok());
    let has_valid_email = valid_saved.is_some() || valid_request.is_some();

    match method {
        ApplicationMethod::Web => {
            if has_valid_email {
                Err(AppError::Conflict(
                    "Email application method is required when a valid contact email is available"
                        .into(),
                ))
            } else {
                Ok(None)
            }
        }
        ApplicationMethod::Email => {
            let email = req_clean.or(saved_clean).ok_or_else(|| {
                AppError::Validation("contact_email required for email application method".into())
            })?;
            validate_email(email)?;
            Ok(Some(email.to_string()))
        }
    }
}

fn compute_application_revision(
    canonical_url: &str,
    resume_upload_id: &str,
    company_name: &str,
    role_title: &str,
    job_description: &str,
    prepared_fields: &serde_json::Value,
    application_method: &str,
    contact_email: Option<&str>,
) -> String {
    let snapshot = serde_json::json!({
        "canonical_url": canonical_url,
        "resume_upload_id": resume_upload_id,
        "company_name": company_name,
        "role_title": role_title,
        "job_description": job_description,
        "prepared_fields": prepared_fields,
        "application_method": application_method,
        "contact_email": contact_email,
    });
    hex::encode(sha2::Sha256::digest(
        serde_json::to_vec(&snapshot).unwrap_or_default(),
    ))
}

pub fn saved_job_routes(state: AppState) -> Router {
    Router::new()
        .route(
            "/api/v1/saved-jobs",
            get(list_saved_jobs).post(create_saved_job),
        )
        .route("/api/v1/saved-jobs/{id}/approve", post(approve_saved_job))
        .route("/api/v1/saved-jobs/{id}/dismiss", post(dismiss_saved_job))
        .route("/api/v1/saved-jobs/{id}/prepare", post(prepare_saved_job))
        .with_state(state)
}

fn normalize_saved_jobs_limit(limit: Option<u32>) -> Result<usize> {
    let limit = limit.unwrap_or(25);
    if limit == 0 {
        return Err(AppError::Validation(
            "limit must be greater than zero".into(),
        ));
    }
    Ok(limit.min(50) as usize)
}

#[utoipa::path(get, path = "/api/v1/saved-jobs", params(SavedJobsQuery), responses((status = 200, body = SavedJobsPageResponseDto), (status = 400)))]
pub async fn list_saved_jobs(
    State(s): State<AppState>,
    a: AuthenticatedUser,
    Query(q): Query<SavedJobsQuery>,
) -> Result<Json<SavedJobsPageResponseDto>> {
    let uid = clean_id(&a.user_id.0);
    let limit = normalize_saved_jobs_limit(q.limit)?;
    let cursor = q.cursor.as_deref().map(clean_id);
    if let Some(value) = cursor.as_deref() {
        Uuid::parse_str(value).map_err(|_| AppError::Validation("Invalid cursor".into()))?;
    }
    let fetch_limit = limit.saturating_add(1) as i64;
    let query = "
        LET $cur = IF !$cursor { NONE } ELSE { (SELECT id, created_at FROM ONLY type::record('saved_job', $cursor) WHERE user_id=type::record('user',$uid) AND status != 'dismissed') };
        RETURN !$cursor OR $cur != NONE;
        SELECT id, source_url, canonical_url, source_site, status, role_title, company_name, job_description, location, application_url, contact_email, job_application_id, created_at, updated_at FROM saved_job
        WHERE user_id=type::record('user',$uid) AND status != 'dismissed'
          AND ($cur = NONE OR created_at < $cur.created_at OR (created_at = $cur.created_at AND id < $cur.id))
        ORDER BY created_at DESC, id DESC
        LIMIT $fetch_limit;
    ";
    let mut r =
        s.db.client
            .query(query)
            .bind(("uid", uid.clone()))
            .bind(("cursor", cursor))
            .bind(("fetch_limit", fetch_limit))
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
    let cursor_valid: Option<bool> = r
        .take(1)
        .map_err(|e| AppError::Database(format!("Failed to resolve saved job cursor: {e}")))?;
    if cursor_valid != Some(true) {
        return Err(AppError::Validation("Invalid cursor".into()));
    }
    let rows: Vec<serde_json::Value> = r.take(2).map_err(|e| AppError::Database(e.to_string()))?;
    let has_more = rows.len() > limit;
    let mut items: Vec<SavedJobResponseDto> = rows
        .into_iter()
        .take(limit)
        .map(saved_job_from_value)
        .collect::<Result<Vec<_>>>()?;
    attach_application_lifecycle(&s, &uid, &mut items).await?;
    let next_cursor = if has_more {
        items.last().map(|job| job.id.clone())
    } else {
        None
    };
    Ok(Json(SavedJobsPageResponseDto { items, next_cursor }))
}

#[utoipa::path(post, path = "/api/v1/saved-jobs", request_body = CreateSavedJobRequestDto, responses((status = 201, body = SavedJobResponseDto), (status = 200, body = SavedJobResponseDto), (status = 400), (status = 409)))]
pub async fn create_saved_job(
    State(s): State<AppState>,
    a: AuthenticatedUser,
    Json(p): Json<CreateSavedJobRequestDto>,
) -> Result<(StatusCode, Json<SavedJobResponseDto>)> {
    let site = detect_site(&p.source_url)?;
    let canonical = canonical_saved_url(site, &p.source_url)?;
    let uid = clean_id(&a.user_id.0);

    if let Some(email) = &p.contact_email {
        validate_email(email)?;
    }
    if let Some(jd) = &p.job_description {
        if jd.chars().count() > 30000 {
            return Err(AppError::Validation(
                "job_description exceeds 30000 characters".into(),
            ));
        }
    }
    if let Some(role) = &p.role_title {
        if role.chars().count() > 200 {
            return Err(AppError::Validation(
                "role_title exceeds 200 characters".into(),
            ));
        }
    }
    if let Some(company) = &p.company_name {
        if company.chars().count() > 200 {
            return Err(AppError::Validation(
                "company_name exceeds 200 characters".into(),
            ));
        }
    }
    if let Some(loc) = &p.location {
        if loc.chars().count() > 200 {
            return Err(AppError::Validation(
                "location exceeds 200 characters".into(),
            ));
        }
    }
    if let Some(app_url) = &p.application_url {
        validate_application_url(app_url)?;
    }

    let mut existing = s
        .db
        .client
        .query("SELECT id FROM saved_job WHERE user_id=type::record('user',$uid) AND canonical_url=$url LIMIT 1;")
        .bind(("uid", uid.clone()))
        .bind(("url", canonical.clone()))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let ids: Vec<serde_json::Value> = existing
        .take(0)
        .map_err(|e| AppError::Database(e.to_string()))?;
    if let Some(v) = ids.first() {
        let ident = v
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        return Ok((
            StatusCode::OK,
            Json(merge_existing_saved_job(&s, &ident, &a.user_id.0, &p).await?),
        ));
    }

    let id = Uuid::new_v4().to_string();
    let result = s
        .db
        .client
        .query("CREATE type::record('saved_job',$id) SET user_id=type::record('user',$uid), source_url=$source_url, canonical_url=$canonical_url, source_site=$source_site, status='awaiting_approval', role_title=$role_title, company_name=$company_name, job_description=$job_description, location=$location, application_url=$application_url, contact_email=$contact_email, job_application_id=NONE, created_at=time::now(), updated_at=time::now() RETURN AFTER;")
        .bind(("id", id.clone()))
        .bind(("uid", uid.clone()))
        .bind(("source_url", p.source_url.clone()))
        .bind(("canonical_url", canonical.clone()))
        .bind(("source_site", site.as_str().to_string()))
        .bind(("role_title", p.role_title.clone()))
        .bind(("company_name", p.company_name.clone()))
        .bind(("job_description", p.job_description.clone()))
        .bind(("location", p.location.clone()))
        .bind(("application_url", p.application_url.clone()))
        .bind(("contact_email", p.contact_email.clone()))
        .await;

    let r = match result {
        Ok(r) => r,
        Err(error) => {
            let mut existing2 = s
                .db
                .client
                .query("SELECT id FROM saved_job WHERE user_id=type::record('user',$uid) AND canonical_url=$url LIMIT 1;")
                .bind(("uid", uid.clone()))
                .bind(("url", canonical.clone()))
                .await
                .map_err(|e| AppError::Database(e.to_string()))?;
            let ids2: Vec<serde_json::Value> = existing2
                .take(0)
                .map_err(|e| AppError::Database(e.to_string()))?;
            if let Some(v) = ids2
                .first()
                .and_then(|v| v.get("id"))
                .and_then(|v| v.as_str())
            {
                return Ok((
                    StatusCode::OK,
                    Json(merge_existing_saved_job(&s, v, &a.user_id.0, &p).await?),
                ));
            }
            return Err(AppError::Conflict(format!(
                "Saved job write conflict: {error}"
            )));
        }
    };
    if let Err(error) = r.check() {
        let mut existing3 = s
            .db
            .client
            .query("SELECT id FROM saved_job WHERE user_id=type::record('user',$uid) AND canonical_url=$url LIMIT 1;")
            .bind(("uid", uid))
            .bind(("url", canonical))
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        let ids3: Vec<serde_json::Value> = existing3
            .take(0)
            .map_err(|e| AppError::Database(e.to_string()))?;
        if let Some(v) = ids3
            .first()
            .and_then(|v| v.get("id"))
            .and_then(|v| v.as_str())
        {
            return Ok((
                StatusCode::OK,
                Json(merge_existing_saved_job(&s, v, &a.user_id.0, &p).await?),
            ));
        }
        return Err(AppError::Conflict(format!(
            "Saved job write conflict: {error}"
        )));
    }

    Ok((
        StatusCode::CREATED,
        Json(load_owned_saved_job(&s, &id, &a.user_id.0).await?),
    ))
}

#[utoipa::path(post, path = "/api/v1/saved-jobs/{id}/approve", params(("id" = String, Path)), responses((status = 200, body = SavedJobResponseDto), (status = 404), (status = 409)))]
pub async fn approve_saved_job(
    State(s): State<AppState>,
    a: AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<Json<SavedJobResponseDto>> {
    let saved = load_owned_saved_job(&s, &id, &a.user_id.0).await?;
    if !approve_transition(saved.status)? {
        return Ok(Json(saved));
    }
    let uid = clean_id(&a.user_id.0);
    let mut r = s
        .db
        .client
        .query("UPDATE type::record('saved_job',$id) SET status='approved', updated_at=time::now() WHERE user_id=type::record('user',$uid) AND status='awaiting_approval' RETURN AFTER;")
        .bind(("id", clean_id(&id)))
        .bind(("uid", uid))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let rows: Vec<serde_json::Value> = r.take(0).map_err(|e| AppError::Database(e.to_string()))?;
    if rows.is_empty() {
        let current = load_owned_saved_job(&s, &id, &a.user_id.0).await?;
        if current.status == SavedJobStatus::Approved {
            return Ok(Json(current));
        }
        return Err(AppError::Conflict("Saved job state changed".into()));
    }
    Ok(Json(load_owned_saved_job(&s, &id, &a.user_id.0).await?))
}

#[utoipa::path(post, path = "/api/v1/saved-jobs/{id}/dismiss", params(("id" = String, Path)), responses((status = 200, body = SavedJobResponseDto), (status = 404), (status = 409)))]
pub async fn dismiss_saved_job(
    State(s): State<AppState>,
    a: AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<Json<SavedJobResponseDto>> {
    let saved = load_owned_saved_job(&s, &id, &a.user_id.0).await?;
    if !dismiss_transition(saved.status)? {
        return Ok(Json(saved));
    }
    let uid = clean_id(&a.user_id.0);
    let query = "
        BEGIN TRANSACTION;
        LET $saved = (SELECT id, canonical_url, job_application_id FROM type::record('saved_job',$id)
            WHERE user_id=type::record('user',$uid) AND status IN ['awaiting_approval','approved'] LIMIT 1);
        IF array::len($saved) != 1 { THROW 'saved_job_state_conflict'; };
        LET $blocking = (SELECT id FROM job_application
            WHERE user_id=type::record('user',$uid)
              AND (id = $saved[0].job_application_id OR canonical_url = $saved[0].canonical_url)
            LIMIT 1);
        IF array::len($blocking) > 0 { THROW 'saved_job_has_application'; };
        LET $updated = (UPDATE type::record('saved_job',$id) SET status='dismissed', updated_at=time::now()
            WHERE user_id=type::record('user',$uid) AND status IN ['awaiting_approval','approved']
              AND job_application_id IS NONE RETURN AFTER);
        IF array::len($updated) != 1 { THROW 'saved_job_state_conflict'; };
        COMMIT TRANSACTION;
    ";
    let result =
        s.db.client
            .query(query)
            .bind(("id", clean_id(&id)))
            .bind(("uid", uid))
            .await
            .map_err(|e| AppError::Database(e.to_string()))
            .and_then(|response| transaction_result(response, "Saved job dismiss conflict"));
    if let Err(error) = result {
        let current = load_owned_saved_job(&s, &id, &a.user_id.0).await?;
        if current.status == SavedJobStatus::Dismissed {
            return Ok(Json(current));
        }
        return Err(error);
    }
    Ok(Json(load_owned_saved_job(&s, &id, &a.user_id.0).await?))
}

#[utoipa::path(post, path = "/api/v1/saved-jobs/{id}/prepare", params(("id" = String, Path)), request_body = PrepareSavedJobRequestDto, responses((status = 200, body = JobApplicationResponseDto), (status = 400), (status = 404), (status = 409)))]
pub async fn prepare_saved_job(
    State(s): State<AppState>,
    a: AuthenticatedUser,
    Path(id): Path<String>,
    Json(p): Json<PrepareSavedJobRequestDto>,
) -> Result<Json<JobApplicationResponseDto>> {
    let saved = load_owned_saved_job(&s, &id, &a.user_id.0).await?;
    require_approved_for_prepare(saved.status)?;
    if saved
        .role_title
        .as_ref()
        .map_or(true, |v| v.trim().is_empty())
        || saved
            .company_name
            .as_ref()
            .map_or(true, |v| v.trim().is_empty())
        || saved
            .job_description
            .as_ref()
            .map_or(true, |v| v.trim().is_empty())
    {
        return Err(AppError::Conflict(
            "Saved job must have role_title, company_name, and job_description before preparing"
                .into(),
        ));
    }

    let contact_email = resolve_prepare_contact_email(
        p.application_method,
        p.contact_email.as_deref(),
        saved.contact_email.as_deref(),
    )?;

    let uid = clean_id(&a.user_id.0);
    let upload = clean_id(&p.resume_upload_id);
    Uuid::parse_str(&upload)
        .map_err(|_| AppError::Validation("Invalid resume upload ID".into()))?;
    if p.application_method == ApplicationMethod::Web {
        if let Some(application_url) = saved.application_url.as_deref() {
            validate_application_url(application_url).map_err(|_| {
                AppError::Conflict("Saved job has an invalid application URL".into())
            })?;
        }
    }
    completed_upload_filename(&s, &a.user_id.0, &upload).await?;

    let method_str = match p.application_method {
        ApplicationMethod::Email => "email",
        ApplicationMethod::Web => "web",
    };
    let role_title = saved.role_title.clone().unwrap_or_default();
    let company_name = saved.company_name.clone().unwrap_or_default();
    let job_description = saved.job_description.clone().unwrap_or_default();

    if let Some(linked_id) = saved.job_application_id.as_deref() {
        let linked_id = clean_id(linked_id);
        let existing = load_owned_application(&s, &linked_id, &a.user_id.0).await?;
        if existing.status == ApplicationStatus::NeedsReview {
            return prepare_reviewed_application(
                &s,
                &a,
                &saved,
                &linked_id,
                &upload,
                detect_site(&saved.source_url)?.as_str(),
                method_str,
                contact_email.as_deref(),
            )
            .await
            .map(Json);
        }
        evaluate_prepare_idempotency(
            &existing,
            &upload,
            p.application_method,
            contact_email.as_deref(),
        )?;
        return Ok(Json(existing));
    }

    if let Some(existing) =
        load_canonical_application_response(&s, &uid, &saved.canonical_url).await?
    {
        if existing.status == ApplicationStatus::NeedsReview {
            return prepare_reviewed_application(
                &s,
                &a,
                &saved,
                &existing.id,
                &upload,
                detect_site(&saved.source_url)?.as_str(),
                method_str,
                contact_email.as_deref(),
            )
            .await
            .map(Json);
        }
        evaluate_prepare_idempotency(
            &existing,
            &upload,
            p.application_method,
            contact_email.as_deref(),
        )?;
        link_existing_application(&s, &id, &uid, &existing.id).await?;
        return Ok(Json(existing));
    }

    let site_str = detect_site(&saved.source_url)?.as_str();
    let prepared_fields = load_prepared_fields(&s, &a, &uid).await?;
    let revision = compute_application_revision(
        &saved.canonical_url,
        &upload,
        &company_name,
        &role_title,
        &job_description,
        &prepared_fields,
        method_str,
        contact_email.as_deref(),
    );
    let app_id = Uuid::new_v4().to_string();
    let input = SavedJobApplicationInput {
        saved_id: clean_id(&id),
        uid: uid.clone(),
        app_id: app_id.clone(),
        upload: upload.clone(),
        site: site_str,
        canonical_url: saved.canonical_url.clone(),
        source_url: saved.source_url.clone(),
        company_name: company_name.clone(),
        role_title: role_title.clone(),
        job_description: job_description.clone(),
        revision: revision.clone(),
        prepared_fields: prepared_fields.clone(),
        method: method_str,
        contact_email: contact_email.clone(),
    };
    match create_saved_job_application(&s, &input).await {
        Ok(()) => Ok(Json(
            load_owned_application(&s, &app_id, &a.user_id.0).await?,
        )),
        Err(error) => {
            if let Some(existing) =
                load_canonical_application_response(&s, &uid, &saved.canonical_url).await?
            {
                if existing.status == ApplicationStatus::NeedsReview {
                    return prepare_reviewed_application(
                        &s,
                        &a,
                        &saved,
                        &existing.id,
                        &upload,
                        site_str,
                        method_str,
                        contact_email.as_deref(),
                    )
                    .await
                    .map(Json);
                }
                evaluate_prepare_idempotency(
                    &existing,
                    &upload,
                    p.application_method,
                    contact_email.as_deref(),
                )?;
                link_existing_application(&s, &id, &uid, &existing.id).await?;
                return Ok(Json(existing));
            }
            Err(error)
        }
    }
}

struct SavedJobApplicationInput<'a> {
    saved_id: String,
    uid: String,
    app_id: String,
    upload: String,
    site: &'a str,
    canonical_url: String,
    source_url: String,
    company_name: String,
    role_title: String,
    job_description: String,
    revision: String,
    prepared_fields: serde_json::Value,
    method: &'a str,
    contact_email: Option<String>,
}

async fn load_prepared_fields(
    state: &AppState,
    auth: &AuthenticatedUser,
    uid: &str,
) -> Result<serde_json::Value> {
    let user = state.identity_service.get_user_by_id(&auth.user_id).await?;
    let mut name_parts = user.name.split_whitespace();
    let first_name = name_parts.next().unwrap_or_default().to_string();
    let last_name = name_parts.collect::<Vec<_>>().join(" ");
    if first_name.trim().is_empty() || last_name.trim().is_empty() || user.email.trim().is_empty() {
        return Err(AppError::Validation(
            "Profile lacks required application fields".into(),
        ));
    }
    let mut prof_res = state
        .db
        .client
        .query("SELECT phone,location,linkedin_url,github_url,portfolio_url FROM candidate_profile WHERE user_id=type::record('user',$uid) LIMIT 1;")
        .bind(("uid", uid.to_string()))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let profiles: Vec<serde_json::Value> = prof_res
        .take(0)
        .map_err(|e| AppError::Database(e.to_string()))?;
    let profile_obj = profiles.into_iter().next();
    Ok(build_prepared_fields(
        first_name,
        last_name,
        user.email,
        profile_obj.as_ref(),
    ))
}

async fn create_saved_job_application(
    state: &AppState,
    input: &SavedJobApplicationInput<'_>,
) -> Result<()> {
    let query = "
        BEGIN TRANSACTION;
        LET $saved = (UPDATE type::record('saved_job',$id) SET updated_at=time::now()
            WHERE user_id=type::record('user',$uid) AND status='approved' AND job_application_id IS NONE RETURN AFTER);
        IF array::len($saved) != 1 { THROW 'saved_job_state_conflict'; };
        LET $upload_row = (SELECT id FROM type::record('resume_upload',$upload)
            WHERE user_id=type::record('user',$uid) AND status='completed');
        IF array::len($upload_row) != 1 { THROW 'resume_upload_unavailable'; };
        LET $created = (CREATE type::record('job_application',$app_id) SET
            user_id=type::record('user',$uid), resume_upload_id=type::record('resume_upload',$upload),
            site=$site, canonical_url=$canonical_url, source_url=$source_url,
            company_name=$company_name, role_title=$role_title, job_description=$job_description,
            source_title=$role_title, retrieved_at=time::now(), revision=$revision,
            status='approved', prepared_fields=$prepared_fields, application_method=$method,
            contact_email=$contact_email, reason_code=NONE, confirmation_url=NONE,
            created_at=time::now(), updated_at=time::now() RETURN AFTER);
        IF array::len($created) != 1 { THROW 'application_create_conflict'; };
        LET $linked = (UPDATE type::record('saved_job',$id) SET
            job_application_id=type::record('job_application',$app_id), updated_at=time::now()
            WHERE user_id=type::record('user',$uid) AND status='approved' AND job_application_id IS NONE RETURN AFTER);
        IF array::len($linked) != 1 { THROW 'saved_job_link_conflict'; };
        COMMIT TRANSACTION;
    ";
    state
        .db
        .client
        .query(query)
        .bind(("id", input.saved_id.clone()))
        .bind(("uid", input.uid.clone()))
        .bind(("app_id", input.app_id.clone()))
        .bind(("upload", input.upload.clone()))
        .bind(("site", input.site.to_string()))
        .bind(("canonical_url", input.canonical_url.clone()))
        .bind(("source_url", input.source_url.clone()))
        .bind(("company_name", input.company_name.clone()))
        .bind(("role_title", input.role_title.clone()))
        .bind(("job_description", input.job_description.clone()))
        .bind(("revision", input.revision.clone()))
        .bind(("prepared_fields", input.prepared_fields.clone()))
        .bind(("method", input.method.to_string()))
        .bind(("contact_email", input.contact_email.clone()))
        .await
        .map_err(|e| AppError::Database(e.to_string()))
        .and_then(|response| transaction_result(response, "Saved job prepare conflict"))?;
    Ok(())
}

async fn prepare_reviewed_application(
    state: &AppState,
    auth: &AuthenticatedUser,
    saved: &SavedJobResponseDto,
    app_id: &str,
    upload: &str,
    site: &str,
    method: &str,
    contact_email: Option<&str>,
) -> Result<JobApplicationResponseDto> {
    let uid = clean_id(&auth.user_id.0);
    let role_title = saved.role_title.clone().unwrap_or_default();
    let company_name = saved.company_name.clone().unwrap_or_default();
    let job_description = saved.job_description.clone().unwrap_or_default();
    let prepared_fields = load_prepared_fields(state, auth, &uid).await?;
    let revision = compute_application_revision(
        &saved.canonical_url,
        upload,
        &company_name,
        &role_title,
        &job_description,
        &prepared_fields,
        method,
        contact_email,
    );
    let query = "
        BEGIN TRANSACTION;
        LET $saved = (UPDATE type::record('saved_job',$id) SET updated_at=time::now()
            WHERE user_id=type::record('user',$uid) AND status='approved'
              AND (job_application_id IS NONE OR job_application_id=type::record('job_application',$app_id))
            RETURN AFTER);
        IF array::len($saved) != 1 { THROW 'saved_job_state_conflict'; };
        LET $upload_row = (SELECT id FROM type::record('resume_upload',$upload)
            WHERE user_id=type::record('user',$uid) AND status='completed');
        IF array::len($upload_row) != 1 { THROW 'resume_upload_unavailable'; };
        LET $updated = (UPDATE type::record('job_application',$app_id) SET
            resume_upload_id=type::record('resume_upload',$upload), site=$site,
            canonical_url=$canonical_url, source_url=$source_url, company_name=$company_name,
            role_title=$role_title, job_description=$job_description, source_title=$role_title,
            retrieved_at=time::now(), revision=$revision, status='approved',
            prepared_fields=$prepared_fields, application_method=$method, contact_email=$contact_email,
            reason_code=NONE, confirmation_url=NONE, updated_at=time::now()
            WHERE user_id=type::record('user',$uid) AND status='needs_review' RETURN AFTER);
        IF array::len($updated) != 1 { THROW 'application_state_conflict'; };
        LET $linked = (UPDATE type::record('saved_job',$id) SET
            job_application_id=type::record('job_application',$app_id), updated_at=time::now()
            WHERE user_id=type::record('user',$uid) AND status='approved' RETURN AFTER);
        IF array::len($linked) != 1 { THROW 'saved_job_link_conflict'; };
        COMMIT TRANSACTION;
    ";
    state
        .db
        .client
        .query(query)
        .bind(("id", clean_id(&saved.id)))
        .bind(("uid", uid.clone()))
        .bind(("app_id", clean_id(app_id)))
        .bind(("upload", clean_id(upload)))
        .bind(("site", site.to_string()))
        .bind(("canonical_url", saved.canonical_url.clone()))
        .bind(("source_url", saved.source_url.clone()))
        .bind(("company_name", company_name))
        .bind(("role_title", role_title))
        .bind(("job_description", job_description))
        .bind(("revision", revision))
        .bind(("prepared_fields", prepared_fields))
        .bind(("method", method.to_string()))
        .bind(("contact_email", contact_email.map(str::to_string)))
        .await
        .map_err(|e| AppError::Database(e.to_string()))
        .and_then(|response| transaction_result(response, "Saved job prepare conflict"))?;
    load_owned_application(state, app_id, &auth.user_id.0).await
}

async fn link_existing_application(
    state: &AppState,
    saved_id: &str,
    uid: &str,
    app_id: &str,
) -> Result<()> {
    let mut r = state
        .db
        .client
        .query("UPDATE type::record('saved_job',$id) SET job_application_id=type::record('job_application',$app_id), updated_at=time::now() WHERE user_id=type::record('user',$uid) AND status='approved' AND job_application_id IS NONE RETURN AFTER;")
        .bind(("id", clean_id(saved_id)))
        .bind(("app_id", clean_id(app_id)))
        .bind(("uid", clean_id(uid)))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let rows: Vec<serde_json::Value> = r.take(0).map_err(|e| AppError::Database(e.to_string()))?;
    if rows.is_empty() {
        return Err(AppError::Conflict(
            "Saved job must be approved before preparing".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_saved_job() -> SavedJobResponseDto {
        SavedJobResponseDto {
            id: "saved_1".into(),
            source_url: "https://linkedin.com/jobs/view/123".into(),
            canonical_url: "https://linkedin.com/jobs/view/123".into(),
            source_site: "linkedin".into(),
            status: SavedJobStatus::AwaitingApproval,
            role_title: None,
            company_name: None,
            job_description: None,
            location: None,
            application_url: None,
            contact_email: None,
            job_application_id: None,
            application_status: None,
            workspace_status: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn dummy_job_application(
        upload_id: &str,
        method: Option<ApplicationMethod>,
        email: Option<&str>,
        status: crate::interfaces::http::applications::ApplicationStatus,
        workspace_id: Option<String>,
    ) -> JobApplicationResponseDto {
        JobApplicationResponseDto {
            id: "app_test_1".into(),
            site: crate::interfaces::http::applications::ApplicationSite::Linkedin,
            canonical_url: "https://linkedin.com/jobs/view/123".into(),
            source_url: "https://linkedin.com/jobs/view/123".into(),
            company_name: "Acme".into(),
            role_title: "Engineer".into(),
            job_description: "Build software".into(),
            resume_upload_id: upload_id.into(),
            resume_filename: "resume.pdf".into(),
            prepared_fields: crate::interfaces::http::applications::PreparedFieldsDto {
                first_name: "A".into(),
                last_name: "B".into(),
                email: "a@b.com".into(),
                phone: None,
                location: None,
                linkedin_url: None,
                github_url: None,
                portfolio_url: None,
            },
            revision: "rev123".into(),
            status,
            application_method: method,
            contact_email: email.map(|s| s.to_string()),
            workspace_id,
            confirmation_url: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn saved_job_rejects_non_linkedin_topcv_urls() {
        assert!(detect_site("https://greenhouse.io/jobs/123").is_err());
        assert!(detect_site("https://evil.example/jobs/123").is_err());
        assert!(detect_site("not-a-url").is_err());
    }

    #[test]
    fn saved_job_accepts_linkedin_and_topcv_urls() {
        assert!(detect_site("https://www.linkedin.com/jobs/view/12345").is_ok());
        assert!(detect_site("https://linkedin.com/jobs/view/12345").is_ok());
        assert!(detect_site("https://topcv.vn/viec-lam/123-ky-su-phan-mem.html").is_ok());
        assert!(detect_site("https://www.topcv.vn/viec-lam-456").is_ok());
    }

    #[test]
    fn canonical_saved_url_normalizes_linkedin() {
        assert_eq!(
            canonical_saved_url(
                SavedJobSite::Linkedin,
                "https://www.linkedin.com/jobs/view/987654?trk=feed#details"
            )
            .unwrap(),
            "https://linkedin.com/jobs/view/987654"
        );
    }

    #[test]
    fn canonical_saved_url_normalizes_topcv() {
        assert_eq!(
            canonical_saved_url(
                SavedJobSite::Topcv,
                "https://www.topcv.vn/viec-lam/12345-ky-su-phan-mem.html?ref=tracking"
            )
            .unwrap(),
            "https://topcv.vn/viec-lam/12345-ky-su-phan-mem.html"
        );
    }

    #[test]
    fn canonical_saved_url_reduces_topcv_trailing_paths() {
        assert_eq!(
            canonical_saved_url(
                SavedJobSite::Topcv,
                "https://www.topcv.vn/viec-lam/12345-ky-su-phan-mem.html/apply?ref=tracking"
            )
            .unwrap(),
            "https://topcv.vn/viec-lam/12345-ky-su-phan-mem.html"
        );
        assert_eq!(
            canonical_saved_url(
                SavedJobSite::Topcv,
                "https://topcv.vn/viec-lam-456/apply/now"
            )
            .unwrap(),
            "https://topcv.vn/viec-lam-456"
        );
        assert!(canonical_saved_url(SavedJobSite::Topcv, "https://topcv.vn/viec-lam/").is_err());
        assert!(canonical_saved_url(SavedJobSite::Topcv, "https://topcv.vn/viec-lam-/").is_err());
    }

    #[test]
    fn saved_jobs_limit_defaults_rejects_zero_and_caps() {
        assert_eq!(normalize_saved_jobs_limit(None).unwrap(), 25);
        assert!(matches!(
            normalize_saved_jobs_limit(Some(0)),
            Err(AppError::Validation(_))
        ));
        assert_eq!(normalize_saved_jobs_limit(Some(1)).unwrap(), 1);
        assert_eq!(normalize_saved_jobs_limit(Some(50)).unwrap(), 50);
        assert_eq!(normalize_saved_jobs_limit(Some(51)).unwrap(), 50);
    }

    #[test]
    fn application_url_requires_credential_free_https() {
        assert!(validate_application_url("https://jobs.lever.co/acme/123?job=1").is_ok());
        assert!(validate_application_url("http://jobs.lever.co/acme/123").is_err());
        assert!(validate_application_url("https://user:pass@jobs.lever.co/acme/123").is_err());
        assert!(validate_application_url("https://user@jobs.lever.co/acme/123").is_err());
        assert!(validate_application_url("https://jobs.lever.co:8443/acme/123").is_err());
        assert!(validate_application_url("not a url").is_err());
        assert!(validate_application_url("https://").is_err());
        assert!(validate_application_url("https://127.0.0.1/apply").is_err());
        assert!(validate_application_url("https://192.168.1.1/apply").is_err());
        assert!(validate_application_url("https://[::1]/apply").is_err());
        assert!(validate_application_url("https://foo.localhost/apply").is_err());
        assert!(validate_application_url("https://intranet/apply").is_err());
        assert!(validate_application_url("https://service.local/apply").is_err());
        assert!(validate_application_url("https://service.internal/apply").is_err());
        assert!(validate_application_url("https://boards.greenhouse.io/acme/jobs/1").is_ok());
    }

    #[test]
    fn prepared_fields_location_uses_candidate_profile_only() {
        let mut saved = dummy_saved_job();
        saved.location = Some("Ho Chi Minh City".into());
        let profile = serde_json::json!({"location": "Hanoi", "phone": "123"});
        let fields = build_prepared_fields(
            "Jane".into(),
            "Doe".into(),
            "jane@example.com".into(),
            Some(&profile),
        );
        assert_eq!(fields["location"].as_str(), Some("Hanoi"));
        assert_ne!(fields["location"].as_str(), saved.location.as_deref());
        assert_eq!(fields["phone"].as_str(), Some("123"));

        let no_profile =
            build_prepared_fields("Jane".into(), "Doe".into(), "jane@example.com".into(), None);
        assert!(no_profile["location"].is_null());
    }

    #[test]
    fn duplicate_merge_only_fills_nonempty_facts_and_marks_material_changes() {
        let existing = dummy_saved_job();
        let incoming = CreateSavedJobRequestDto {
            source_url: "https://linkedin.com/jobs/view/123?trk=feed".into(),
            role_title: Some("Engineer".into()),
            company_name: Some("   ".into()),
            job_description: Some("Build things".into()),
            location: None,
            application_url: Some("https://jobs.lever.co/acme/123".into()),
            contact_email: None,
        };
        let (merged, material_changed) = merge_saved_job_facts(&existing, &incoming);
        assert_eq!(merged.role_title.as_deref(), Some("Engineer"));
        assert_eq!(merged.company_name, existing.company_name);
        assert_eq!(merged.job_description.as_deref(), Some("Build things"));
        assert_eq!(merged.location, existing.location);
        assert_eq!(
            merged.source_url,
            "https://linkedin.com/jobs/view/123?trk=feed"
        );
        assert!(material_changed);

        let identical = CreateSavedJobRequestDto {
            source_url: existing.source_url.clone(),
            role_title: existing.role_title.clone(),
            company_name: existing.company_name.clone(),
            job_description: existing.job_description.clone(),
            location: existing.location.clone(),
            application_url: existing.application_url.clone(),
            contact_email: existing.contact_email.clone(),
        };
        let (merged_identical, material_identical) = merge_saved_job_facts(&existing, &identical);
        assert!(!merged_identical.differs_from(&existing));
        assert!(!material_identical);
    }

    #[test]
    fn duplicate_save_transition_reapproves_nonterminal_applications_and_blocks_terminal_ones() {
        let (status, mark_review) =
            duplicate_save_transition(SavedJobStatus::Approved, true, None).unwrap();
        assert_eq!(status, SavedJobStatus::AwaitingApproval);
        assert!(!mark_review);

        let (status, mark_review) = duplicate_save_transition(
            SavedJobStatus::Approved,
            true,
            Some(ApplicationStatus::Approved),
        )
        .unwrap();
        assert_eq!(status, SavedJobStatus::AwaitingApproval);
        assert!(mark_review);

        let (status, mark_review) = duplicate_save_transition(
            SavedJobStatus::AwaitingApproval,
            true,
            Some(ApplicationStatus::NeedsReview),
        )
        .unwrap();
        assert_eq!(status, SavedJobStatus::AwaitingApproval);
        assert!(mark_review);

        let (status, mark_review) = duplicate_save_transition(
            SavedJobStatus::Approved,
            false,
            Some(ApplicationStatus::Submitted),
        )
        .unwrap();
        assert_eq!(status, SavedJobStatus::Approved);
        assert!(!mark_review);

        assert!(matches!(
            duplicate_save_transition(
                SavedJobStatus::Approved,
                true,
                Some(ApplicationStatus::Submitted)
            ),
            Err(AppError::Conflict(_))
        ));

        let (status, mark_review) =
            duplicate_save_transition(SavedJobStatus::Dismissed, false, None).unwrap();
        assert_eq!(status, SavedJobStatus::AwaitingApproval);
        assert!(!mark_review);

        let (status, mark_review) = duplicate_save_transition(
            SavedJobStatus::Dismissed,
            false,
            Some(ApplicationStatus::Prepared),
        )
        .unwrap();
        assert_eq!(status, SavedJobStatus::AwaitingApproval);
        assert!(mark_review);

        for linked in [
            ApplicationStatus::Submitting,
            ApplicationStatus::Submitted,
            ApplicationStatus::Unknown,
        ] {
            assert!(matches!(
                duplicate_save_transition(SavedJobStatus::Dismissed, false, Some(linked)),
                Err(AppError::Conflict(_))
            ));
            assert!(matches!(
                duplicate_save_transition(SavedJobStatus::Approved, true, Some(linked)),
                Err(AppError::Conflict(_))
            ));
        }

        let (status, mark_review) =
            duplicate_save_transition(SavedJobStatus::AwaitingApproval, true, None).unwrap();
        assert_eq!(status, SavedJobStatus::AwaitingApproval);
        assert!(!mark_review);
    }

    #[test]
    fn canonical_saved_url_rejects_unsafe_urls() {
        assert!(
            canonical_saved_url(SavedJobSite::Linkedin, "http://linkedin.com/jobs/view/123")
                .is_err()
        );
        assert!(
            canonical_saved_url(SavedJobSite::Linkedin, "https://linkedin.com/jobs/search")
                .is_err()
        );
        assert!(canonical_saved_url(
            SavedJobSite::Linkedin,
            "https://linkedin.com.evil.test/jobs/view/123"
        )
        .is_err());
    }

    #[test]
    fn email_validation_accepts_valid_rejects_invalid() {
        assert!(validate_email("user@example.com").is_ok());
        assert!(validate_email("hr@company.vn").is_ok());
        assert!(validate_email("not-an-email").is_err());
        assert!(validate_email("@nodomain").is_err());
        assert!(validate_email("noatsign.com").is_err());
        assert!(validate_email("missing@tld").is_err());
        assert!(validate_email("").is_err());
    }

    #[test]
    fn email_validation_comprehensive_matrix() {
        assert!(validate_email("recruiter@tech.example.co").is_ok());
        assert!(validate_email("candidate+test@subdomain.org.vn").is_ok());
        assert!(validate_email("first.last@corporation.com").is_ok());
        assert!(validate_email("   ").is_err());
        assert!(validate_email("missing_domain@").is_err());
        assert!(validate_email("@missing_local.com").is_err());
        assert!(validate_email("spaces inside@domain.com").is_err());
        assert!(validate_email("user@domain with space.com").is_err());
        assert!(validate_email("user@@doubleat.com").is_err());
        assert!(validate_email("user@domain..double.com").is_err());
        assert!(validate_email(&format!("{}@example.com", "a".repeat(255))).is_err());
    }

    #[test]
    fn create_saved_job_request_rejects_unknown_fields() {
        assert!(
            serde_json::from_value::<CreateSavedJobRequestDto>(serde_json::json!({
                "source_url": "https://linkedin.com/jobs/view/123",
                "unknown_field": true
            }))
            .is_err()
        );
    }

    #[test]
    fn prepare_saved_job_request_rejects_unknown_fields() {
        assert!(
            serde_json::from_value::<PrepareSavedJobRequestDto>(serde_json::json!({
                "resume_upload_id": "abc",
                "application_method": "email",
                "extra": "unexpected"
            }))
            .is_err()
        );
    }

    #[test]
    fn prepare_contact_email_uses_email_first_precedence() {
        assert_eq!(
            resolve_prepare_contact_email(ApplicationMethod::Web, None, None).unwrap(),
            None
        );
        assert_eq!(
            resolve_prepare_contact_email(ApplicationMethod::Web, None, Some("invalid")).unwrap(),
            None
        );
        assert_eq!(
            resolve_prepare_contact_email(
                ApplicationMethod::Web,
                Some("invalid"),
                Some("also-invalid")
            )
            .unwrap(),
            None
        );
        assert!(matches!(
            resolve_prepare_contact_email(ApplicationMethod::Web, None, Some("hr@company.com")),
            Err(AppError::Conflict(_))
        ));
        assert!(matches!(
            resolve_prepare_contact_email(
                ApplicationMethod::Web,
                Some("recruiter@company.com"),
                None
            ),
            Err(AppError::Conflict(_))
        ));
        assert!(matches!(
            resolve_prepare_contact_email(
                ApplicationMethod::Web,
                Some("invalid"),
                Some("hr@company.com")
            ),
            Err(AppError::Conflict(_))
        ));
        assert_eq!(
            resolve_prepare_contact_email(ApplicationMethod::Email, None, Some(" hr@company.com "))
                .unwrap()
                .as_deref(),
            Some("hr@company.com")
        );
        assert_eq!(
            resolve_prepare_contact_email(
                ApplicationMethod::Email,
                Some("recruiter@company.com"),
                Some("hr@company.com")
            )
            .unwrap()
            .as_deref(),
            Some("recruiter@company.com")
        );
        assert!(resolve_prepare_contact_email(
            ApplicationMethod::Email,
            Some("invalid"),
            Some("hr@company.com")
        )
        .is_err());
        assert!(resolve_prepare_contact_email(ApplicationMethod::Email, None, None).is_err());
    }

    #[test]
    fn saved_job_transitions_enforce_approval_and_dismissal_rules() {
        assert!(approve_transition(SavedJobStatus::AwaitingApproval).unwrap());
        assert!(!approve_transition(SavedJobStatus::Approved).unwrap());
        assert!(matches!(
            approve_transition(SavedJobStatus::Dismissed),
            Err(AppError::Conflict(_))
        ));

        assert!(dismiss_transition(SavedJobStatus::AwaitingApproval).unwrap());
        assert!(dismiss_transition(SavedJobStatus::Approved).unwrap());
        assert!(!dismiss_transition(SavedJobStatus::Dismissed).unwrap());

        assert!(require_approved_for_prepare(SavedJobStatus::Approved).is_ok());
        assert!(matches!(
            require_approved_for_prepare(SavedJobStatus::AwaitingApproval),
            Err(AppError::Conflict(_))
        ));
        assert!(matches!(
            require_approved_for_prepare(SavedJobStatus::Dismissed),
            Err(AppError::Conflict(_))
        ));
    }

    #[test]
    fn prepare_saved_job_identical_retry_succeeds() {
        let app = dummy_job_application(
            "upload_123",
            Some(ApplicationMethod::Email),
            Some("hr@company.com"),
            crate::interfaces::http::applications::ApplicationStatus::Approved,
            None,
        );
        let result = evaluate_prepare_idempotency(
            &app,
            "upload_123",
            ApplicationMethod::Email,
            Some("hr@company.com"),
        );
        assert!(result.is_ok());

        let app_web = dummy_job_application(
            "upload_123",
            Some(ApplicationMethod::Web),
            None,
            crate::interfaces::http::applications::ApplicationStatus::Approved,
            None,
        );
        let result_web =
            evaluate_prepare_idempotency(&app_web, "upload_123", ApplicationMethod::Web, None);
        assert!(result_web.is_ok());
        let app_web_with_legacy_contact = dummy_job_application(
            "upload_123",
            Some(ApplicationMethod::Web),
            Some("invalid"),
            crate::interfaces::http::applications::ApplicationStatus::Approved,
            None,
        );
        assert!(evaluate_prepare_idempotency(
            &app_web_with_legacy_contact,
            "upload_123",
            ApplicationMethod::Web,
            None
        )
        .is_ok());
        let app_web_with_valid_legacy_contact = dummy_job_application(
            "upload_123",
            Some(ApplicationMethod::Web),
            Some("legacy@company.com"),
            crate::interfaces::http::applications::ApplicationStatus::Approved,
            None,
        );
        assert!(evaluate_prepare_idempotency(
            &app_web_with_valid_legacy_contact,
            "upload_123",
            ApplicationMethod::Web,
            None
        )
        .is_ok());

        let app_legacy = dummy_job_application(
            "upload_123",
            None,
            None,
            crate::interfaces::http::applications::ApplicationStatus::Prepared,
            None,
        );
        let result_legacy =
            evaluate_prepare_idempotency(&app_legacy, "upload_123", ApplicationMethod::Web, None);
        assert!(result_legacy.is_ok());
    }

    #[test]
    fn prepare_saved_job_changed_resume_conflicts() {
        let app = dummy_job_application(
            "upload_123",
            Some(ApplicationMethod::Email),
            Some("hr@company.com"),
            crate::interfaces::http::applications::ApplicationStatus::Approved,
            None,
        );
        let result = evaluate_prepare_idempotency(
            &app,
            "upload_different_456",
            ApplicationMethod::Email,
            Some("hr@company.com"),
        );
        assert!(result.is_err());
        match result {
            Err(AppError::Conflict(_)) => {}
            _ => panic!("Expected conflict error on changed resume"),
        }
    }

    #[test]
    fn prepare_saved_job_changed_method_conflicts() {
        let app = dummy_job_application(
            "upload_123",
            Some(ApplicationMethod::Email),
            Some("hr@company.com"),
            crate::interfaces::http::applications::ApplicationStatus::Approved,
            None,
        );
        let result = evaluate_prepare_idempotency(&app, "upload_123", ApplicationMethod::Web, None);
        assert!(result.is_err());
        match result {
            Err(AppError::Conflict(_)) => {}
            _ => panic!("Expected conflict error on changed method"),
        }

        let app_web = dummy_job_application(
            "upload_123",
            Some(ApplicationMethod::Web),
            None,
            crate::interfaces::http::applications::ApplicationStatus::Approved,
            None,
        );
        let result_web_to_email = evaluate_prepare_idempotency(
            &app_web,
            "upload_123",
            ApplicationMethod::Email,
            Some("hr@company.com"),
        );
        assert!(result_web_to_email.is_err());
    }

    #[test]
    fn prepare_saved_job_changed_recipient_for_email_conflicts() {
        let app = dummy_job_application(
            "upload_123",
            Some(ApplicationMethod::Email),
            Some("hr@company.com"),
            crate::interfaces::http::applications::ApplicationStatus::Approved,
            None,
        );
        let result = evaluate_prepare_idempotency(
            &app,
            "upload_123",
            ApplicationMethod::Email,
            Some("other_hr@company.com"),
        );
        assert!(result.is_err());
        match result {
            Err(AppError::Conflict(_)) => {}
            _ => panic!("Expected conflict error on changed email recipient"),
        }
    }

    #[test]
    fn prepare_saved_job_preserves_submitted_application() {
        let app_submitted = dummy_job_application(
            "upload_123",
            Some(ApplicationMethod::Email),
            Some("hr@company.com"),
            crate::interfaces::http::applications::ApplicationStatus::Submitted,
            Some("ws_submitted_999".into()),
        );

        let result = evaluate_prepare_idempotency(
            &app_submitted,
            "upload_123",
            ApplicationMethod::Email,
            Some("hr@company.com"),
        );
        assert!(result.is_ok());
        assert_eq!(
            app_submitted.status,
            crate::interfaces::http::applications::ApplicationStatus::Submitted
        );
        assert_eq!(app_submitted.workspace_id, Some("ws_submitted_999".into()));

        let changed_attempt = evaluate_prepare_idempotency(
            &app_submitted,
            "upload_new",
            ApplicationMethod::Email,
            Some("hr@company.com"),
        );
        assert!(changed_attempt.is_err());
        assert_eq!(
            app_submitted.status,
            crate::interfaces::http::applications::ApplicationStatus::Submitted
        );
    }

    #[test]
    fn revision_snapshot_includes_method_and_recipient() {
        let fields = serde_json::json!({
            "first_name": "Jane",
            "last_name": "Doe",
            "email": "jane@example.com"
        });

        let rev_email_1 = compute_application_revision(
            "https://linkedin.com/jobs/view/123",
            "upload_1",
            "Acme",
            "Engineer",
            "JD text",
            &fields,
            "email",
            Some("hr1@company.com"),
        );

        let rev_email_2 = compute_application_revision(
            "https://linkedin.com/jobs/view/123",
            "upload_1",
            "Acme",
            "Engineer",
            "JD text",
            &fields,
            "email",
            Some("hr2@company.com"),
        );

        let rev_web = compute_application_revision(
            "https://linkedin.com/jobs/view/123",
            "upload_1",
            "Acme",
            "Engineer",
            "JD text",
            &fields,
            "web",
            None,
        );

        let rev_email_1_retry = compute_application_revision(
            "https://linkedin.com/jobs/view/123",
            "upload_1",
            "Acme",
            "Engineer",
            "JD text",
            &fields,
            "email",
            Some("hr1@company.com"),
        );

        assert_eq!(rev_email_1, rev_email_1_retry);
        assert_ne!(rev_email_1, rev_email_2);
        assert_ne!(rev_email_1, rev_web);
    }

    #[test]
    fn recipient_immutability_across_idempotent_retries() {
        let app_email = dummy_job_application(
            "upload_123",
            Some(ApplicationMethod::Email),
            Some("hr@company.com"),
            crate::interfaces::http::applications::ApplicationStatus::Approved,
            None,
        );

        let identical_retry = evaluate_prepare_idempotency(
            &app_email,
            "upload_123",
            ApplicationMethod::Email,
            Some("hr@company.com"),
        );
        assert!(identical_retry.is_ok());

        let changed_recipient = evaluate_prepare_idempotency(
            &app_email,
            "upload_123",
            ApplicationMethod::Email,
            Some("other_hr@company.com"),
        );
        assert!(matches!(changed_recipient, Err(AppError::Conflict(_))));

        let missing_recipient =
            evaluate_prepare_idempotency(&app_email, "upload_123", ApplicationMethod::Email, None);
        assert!(matches!(missing_recipient, Err(AppError::Conflict(_))));

        let app_web = dummy_job_application(
            "upload_123",
            Some(ApplicationMethod::Web),
            None,
            crate::interfaces::http::applications::ApplicationStatus::Approved,
            None,
        );

        let web_retry =
            evaluate_prepare_idempotency(&app_web, "upload_123", ApplicationMethod::Web, None);
        assert!(web_retry.is_ok());

        let web_ignores_recipient_argument = evaluate_prepare_idempotency(
            &app_web,
            "upload_123",
            ApplicationMethod::Web,
            Some("hr@company.com"),
        );
        assert!(web_ignores_recipient_argument.is_ok());
    }
}
