use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    interfaces::http::{extractors::authenticated_user::AuthenticatedUser, workspaces::AppState},
    shared::{error::AppError, Result},
};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ApplicationSite {
    Greenhouse,
    Topcv,
    Linkedin,
}
impl ApplicationSite {
    fn as_str(self) -> &'static str {
        match self {
            Self::Greenhouse => "greenhouse",
            Self::Topcv => "topcv",
            Self::Linkedin => "linkedin",
        }
    }
    fn hosts(self) -> &'static [&'static str] {
        match self {
            Self::Greenhouse => &["boards.greenhouse.io", "job-boards.greenhouse.io"],
            Self::Topcv => &["topcv.vn", "www.topcv.vn"],
            Self::Linkedin => &["linkedin.com", "www.linkedin.com"],
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ApplicationStatus {
    Prepared,
    Approved,
    Submitting,
    Submitted,
    NeedsReview,
    Unknown,
}
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct PreparedFieldsDto {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: Option<String>,
    pub location: Option<String>,
    pub linkedin_url: Option<String>,
    pub github_url: Option<String>,
    pub portfolio_url: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct PrepareApplicationRequestDto {
    pub site: ApplicationSite,
    pub source_url: String,
    pub company_name: String,
    pub role_title: String,
    pub job_description: String,
    pub resume_upload_id: String,
    pub application_method: Option<ApplicationMethod>,
    pub contact_email: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ApproveApplicationRequestDto {
    pub revision: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct BlockApplicationRequestDto {
    pub reason_code: BlockReasonCode,
}
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum BlockReasonCode {
    MissingRequiredField,
    UnsupportedForm,
    LoginRequired,
    Captcha,
    FormChanged,
}
impl BlockReasonCode {
    fn as_str(&self) -> &'static str {
        match self {
            Self::MissingRequiredField => "missing_required_field",
            Self::UnsupportedForm => "unsupported_form",
            Self::LoginRequired => "login_required",
            Self::Captcha => "captcha",
            Self::FormChanged => "form_changed",
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ResultApplicationRequestDto {
    pub outcome: ApplicationOutcome,
    pub confirmation_url: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ApplicationOutcome {
    Confirmed,
    Uncertain,
    NotSubmitted,
}
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ReconcileApplicationRequestDto {
    pub submitted: bool,
}
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ApplicationMethod {
    Email,
    Web,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct JobApplicationResponseDto {
    pub id: String,
    pub site: ApplicationSite,
    pub canonical_url: String,
    pub source_url: String,
    pub company_name: String,
    pub role_title: String,
    pub job_description: String,
    pub resume_upload_id: String,
    pub resume_filename: String,
    pub prepared_fields: PreparedFieldsDto,
    pub revision: String,
    pub status: ApplicationStatus,
    pub application_method: Option<ApplicationMethod>,
    pub contact_email: Option<String>,
    pub workspace_id: Option<String>,
    pub confirmation_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
#[derive(Debug, Deserialize)]
struct UploadRow {
    user_id: String,
    status: String,
}
#[derive(Debug, Deserialize)]
struct ProfileRow {
    phone: Option<String>,
    location: Option<String>,
    linkedin_url: Option<String>,
    github_url: Option<String>,
    portfolio_url: Option<String>,
}

pub fn application_routes(state: AppState) -> Router {
    Router::new()
        .route(
            "/api/v1/applications",
            get(list_applications).post(prepare_application),
        )
        .route("/api/v1/applications/{id}", get(get_application))
        .route(
            "/api/v1/applications/{id}/prepare",
            axum::routing::put(reprepare_application),
        )
        .route(
            "/api/v1/applications/{id}/approve",
            post(approve_application),
        )
        .route("/api/v1/applications/{id}/start", post(start_application))
        .route("/api/v1/applications/{id}/block", post(block_application))
        .route("/api/v1/applications/{id}/result", post(record_result))
        .route(
            "/api/v1/applications/{id}/reconcile",
            post(reconcile_application),
        )
        .with_state(state)
}

fn canonical_url(site: ApplicationSite, raw: &str) -> Result<String> {
    if raw.chars().count() > 2048 {
        return Err(AppError::Validation(
            "Application URL exceeds 2048 characters".into(),
        ));
    }
    let mut url = reqwest::Url::parse(raw)
        .map_err(|_| AppError::Validation("Invalid application URL".into()))?;
    let host = url.host_str().unwrap_or("").to_ascii_lowercase();
    if url.scheme() != "https"
        || !url.username().is_empty()
        || url.password().is_some()
        || url.port().is_some()
        || !site.hosts().contains(&host.as_str())
    {
        return Err(AppError::Validation("Unsupported application URL".into()));
    }
    let parts: Vec<_> = url
        .path_segments()
        .into_iter()
        .flatten()
        .filter(|p| !p.is_empty())
        .collect();
    let stable = match site {
        ApplicationSite::Greenhouse => {
            let board = parts
                .first()
                .filter(|p| {
                    p.chars()
                        .all(|c| c.is_ascii_alphanumeric() || "-_".contains(c))
                })
                .ok_or_else(|| {
                    AppError::Validation(
                        "Greenhouse URL must contain a board slug and job ID".into(),
                    )
                })?;
            let id = if parts.get(1) == Some(&"jobs") {
                parts.get(2)
            } else {
                parts.get(1)
            }
            .filter(|id| id.chars().all(|c| c.is_ascii_digit()))
            .ok_or_else(|| {
                AppError::Validation("Greenhouse URL must contain a stable job ID".into())
            })?;
            format!("/{board}/{id}")
        }
        ApplicationSite::Topcv => {
            let first = parts.first().copied().unwrap_or_default();
            if let Some(slug) = first.strip_prefix("viec-lam-") {
                if slug.is_empty() {
                    return Err(AppError::Validation(
                        "TopCV URL must contain a stable job path or ID".into(),
                    ));
                }
                format!("/{first}")
            } else if first == "viec-lam" {
                let id = parts.get(1).ok_or_else(|| {
                    AppError::Validation("TopCV URL must contain a stable job path or ID".into())
                })?;
                format!("/viec-lam/{id}")
            } else {
                return Err(AppError::Validation(
                    "TopCV URL must contain a stable job path or ID".into(),
                ));
            }
        }
        ApplicationSite::Linkedin => {
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
    };
    let normalized_host = match site {
        ApplicationSite::Greenhouse => "boards.greenhouse.io",
        ApplicationSite::Topcv => "topcv.vn",
        ApplicationSite::Linkedin => "linkedin.com",
    };
    url.set_host(Some(normalized_host))
        .map_err(|_| AppError::Validation("Invalid canonical host".into()))?;
    url.set_path(&stable);
    url.set_query(None);
    url.set_fragment(None);
    Ok(url.to_string().trim_end_matches('/').to_string())
}
fn validated_confirmation_url(
    site: ApplicationSite,
    raw: &str,
    canonical_job_url: &str,
) -> Result<String> {
    let mut url = reqwest::Url::parse(raw)
        .map_err(|_| AppError::Validation("Invalid confirmation URL".into()))?;
    let canonical_confirmation_job = canonical_url(site, raw)?;
    if canonical_confirmation_job != canonical_job_url {
        return Err(AppError::Validation(
            "Confirmation URL does not match the recorded job".into(),
        ));
    }
    let job_url = reqwest::Url::parse(canonical_job_url)
        .map_err(|_| AppError::Validation("Invalid recorded job URL".into()))?;
    let job_path = job_url.path().trim_end_matches('/');
    let confirmation_path = url.path().trim_end_matches('/');
    let suffix = confirmation_path.strip_prefix(job_path).ok_or_else(|| {
        AppError::Validation("Confirmation URL does not match the recorded job".into())
    })?;
    if (site == ApplicationSite::Greenhouse && suffix != "/confirmation")
        || (site != ApplicationSite::Greenhouse && !suffix.is_empty())
    {
        return Err(AppError::Validation(
            "Unsupported confirmation URL path".into(),
        ));
    }
    url.set_host(job_url.host_str())
        .map_err(|_| AppError::Validation("Invalid confirmation host".into()))?;
    url.set_query(None);
    url.set_fragment(None);
    Ok(url.to_string().trim_end_matches('/').to_string())
}

fn confirmation_url_for_result(
    application_method: Option<ApplicationMethod>,
    site: ApplicationSite,
    canonical_job_url: &str,
    confirmation_url: Option<&str>,
) -> Result<Option<String>> {
    if matches!(application_method, Some(ApplicationMethod::Email)) {
        confirmation_url
            .map(|raw| validated_confirmation_url(site, raw, canonical_job_url))
            .transpose()
    } else {
        let raw = confirmation_url
            .ok_or_else(|| AppError::Validation("Confirmation URL required".into()))?;
        Ok(Some(validated_confirmation_url(
            site,
            raw,
            canonical_job_url,
        )?))
    }
}
pub(crate) fn validate_email(email: &str) -> Result<()> {
    let trimmed = email.trim();
    if trimmed.is_empty() || trimmed.len() > 254 {
        return Err(AppError::Validation("Invalid contact email".into()));
    }
    let (local, domain) = trimmed
        .rsplit_once('@')
        .ok_or_else(|| AppError::Validation("Invalid contact email".into()))?;
    if local.is_empty()
        || domain.is_empty()
        || local.contains('@')
        || domain.contains('@')
        || !domain.contains('.')
        || domain.starts_with('.')
        || domain.ends_with('.')
        || domain.contains("..")
        || local.contains(char::is_whitespace)
        || domain.contains(char::is_whitespace)
    {
        return Err(AppError::Validation("Invalid contact email".into()));
    }
    Ok(())
}

fn validate_text(p: &PrepareApplicationRequestDto) -> Result<()> {
    if p.company_name.trim().is_empty()
        || p.role_title.trim().is_empty()
        || p.job_description.trim().is_empty()
        || p.company_name.chars().count() > 200
        || p.role_title.chars().count() > 200
        || p.job_description.chars().count() > 30000
    {
        return Err(AppError::Validation("Invalid application details".into()));
    }
    let application_method = p.application_method.unwrap_or(ApplicationMethod::Web);
    if application_method == ApplicationMethod::Email {
        let email = p.contact_email.as_deref().ok_or_else(|| {
            AppError::Validation("Contact email required for email application".into())
        })?;
        validate_email(email)?;
    } else if let Some(email) = p.contact_email.as_deref() {
        if validate_email(email).is_ok() {
            return Err(AppError::Validation(
                "Use email application when a contact email is available".into(),
            ));
        }
    }
    Ok(())
}

fn resolve_direct_contact_email(
    requested_method: ApplicationMethod,
    requested_email: Option<&str>,
    saved_email: Option<&str>,
) -> Result<(ApplicationMethod, Option<String>)> {
    let request_clean = requested_email.map(str::trim).filter(|e| !e.is_empty());
    let saved_clean = saved_email.map(str::trim).filter(|e| !e.is_empty());
    let valid_saved = saved_clean.filter(|e| validate_email(e).is_ok());
    let valid_request = request_clean.filter(|e| validate_email(e).is_ok());
    match requested_method {
        ApplicationMethod::Web => {
            if valid_saved.is_some() {
                Err(AppError::Conflict(
                    "Email application method is required when a valid contact email is available"
                        .into(),
                ))
            } else if valid_request.is_some() {
                Err(AppError::Validation(
                    "Use email application when a contact email is available".into(),
                ))
            } else {
                Ok((ApplicationMethod::Web, None))
            }
        }
        ApplicationMethod::Email => {
            if let Some(saved) = valid_saved {
                if let Some(requested) = request_clean {
                    if validate_email(requested).is_err() {
                        return Err(AppError::Validation("Invalid contact email".into()));
                    }
                    if requested != saved {
                        return Err(AppError::Conflict(
                            "Application already has a different recipient".into(),
                        ));
                    }
                }
                Ok((ApplicationMethod::Email, Some(saved.to_string())))
            } else {
                let email = request_clean.ok_or_else(|| {
                    AppError::Validation("Contact email required for email application".into())
                })?;
                validate_email(email)?;
                Ok((ApplicationMethod::Email, Some(email.to_string())))
            }
        }
    }
}

fn ensure_same_application_revision(
    existing_revision: &str,
    requested_revision: &str,
) -> Result<()> {
    if existing_revision == requested_revision {
        Ok(())
    } else {
        Err(AppError::Conflict(
            "Application already exists with different details".into(),
        ))
    }
}

fn require_application_write(rows: &[serde_json::Value]) -> Result<()> {
    if rows.is_empty() {
        Err(AppError::Conflict(
            "Application state changed during preparation".into(),
        ))
    } else {
        Ok(())
    }
}
pub(crate) fn clean_id(id: &str) -> String {
    crate::shared::record_id::clean_record_id(id)
}

pub(crate) fn application_response_from_value(
    value: serde_json::Value,
) -> Result<JobApplicationResponseDto> {
    let mut response: JobApplicationResponseDto = serde_json::from_value(value)
        .map_err(|e| AppError::Database(format!("Malformed application: {e}")))?;
    response.id = clean_id(&response.id);
    response.resume_upload_id = clean_id(&response.resume_upload_id);
    response.workspace_id = response.workspace_id.map(|id| clean_id(&id));
    Ok(response)
}

pub(crate) async fn load_owned_application(
    state: &AppState,
    id: &str,
    uid: &str,
) -> Result<JobApplicationResponseDto> {
    let id = clean_id(id);
    Uuid::parse_str(&id).map_err(|_| AppError::Validation("Invalid application ID".into()))?;
    let uid = clean_id(uid);
    let mut r=state.db.client.query("SELECT id, site, canonical_url, source_url, company_name, role_title, job_description, resume_upload_id, resume_upload_id.filename AS resume_filename, prepared_fields, revision, status, application_method, contact_email, workspace_id, confirmation_url, created_at, updated_at FROM type::record('job_application',$id) WHERE user_id=type::record('user',$uid);").bind(("id",id.clone())).bind(("uid",uid)).await.map_err(|e|AppError::Database(e.to_string()))?;
    let rows: Vec<serde_json::Value> = r.take(0).map_err(|e| AppError::Database(e.to_string()))?;
    let value = rows
        .into_iter()
        .next()
        .ok_or_else(|| AppError::NotFound("Application not found".into()))?;
    application_response_from_value(value)
}
#[utoipa::path(get,path="/api/v1/applications",responses((status=200,body=[JobApplicationResponseDto])))]
pub async fn list_applications(
    State(s): State<AppState>,
    a: AuthenticatedUser,
) -> Result<Json<Vec<JobApplicationResponseDto>>> {
    let uid = clean_id(&a.user_id.0);
    let mut r=s.db.client.query("SELECT id, site, canonical_url, source_url, company_name, role_title, job_description, resume_upload_id, resume_upload_id.filename AS resume_filename, prepared_fields, revision, status, application_method, contact_email, workspace_id, confirmation_url, created_at, updated_at FROM job_application WHERE user_id=type::record('user',$uid) ORDER BY created_at DESC;").bind(("uid",uid)).await.map_err(|e|AppError::Database(e.to_string()))?;
    let rows: Vec<serde_json::Value> = r.take(0).map_err(|e| AppError::Database(e.to_string()))?;
    rows.into_iter()
        .map(application_response_from_value)
        .collect::<Result<Vec<_>>>()
        .map(Json)
}
#[utoipa::path(get,path="/api/v1/applications/{id}",params(("id"=String,Path)),responses((status=200,body=JobApplicationResponseDto),(status=404)))]
pub async fn get_application(
    State(s): State<AppState>,
    a: AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<Json<JobApplicationResponseDto>> {
    Ok(Json(load_owned_application(&s, &id, &a.user_id.0).await?))
}
pub(crate) async fn completed_upload_filename(
    state: &AppState,
    user_id: &str,
    upload_id: &str,
) -> Result<()> {
    let id = clean_id(upload_id);
    Uuid::parse_str(&id).map_err(|_| AppError::Validation("Invalid resume upload ID".into()))?;
    let mut result = state
        .db
        .client
        .query(
            "SELECT record::id(user_id) AS user_id, status FROM resume_upload WHERE id = type::record('resume_upload',$id) LIMIT 1;",
        )
        .bind(("id", id))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let uploads: Vec<serde_json::Value> = result
        .take(0)
        .map_err(|e| AppError::Database(e.to_string()))?;
    let upload: UploadRow = uploads
        .into_iter()
        .next()
        .ok_or_else(|| AppError::Conflict("Resume upload is unavailable".into()))
        .and_then(|value| {
            serde_json::from_value(value)
                .map_err(|e| AppError::Database(format!("Malformed resume upload: {e}")))
        })?;
    if clean_id(&upload.user_id) != clean_id(user_id) {
        return Err(AppError::Forbidden(
            "Resume upload belongs to another user".into(),
        ));
    }
    if upload.status != "completed" {
        return Err(AppError::Conflict(
            "Resume upload is no longer completed".into(),
        ));
    }
    Ok(())
}

async fn prepare(
    state: AppState,
    auth: AuthenticatedUser,
    mut p: PrepareApplicationRequestDto,
    id: Option<String>,
) -> Result<(StatusCode, Json<JobApplicationResponseDto>)> {
    let is_new = id.is_none();
    let canonical = canonical_url(p.site, &p.source_url)?;
    let uid = auth.user_id.0.replace("user:", "").replace('`', "");
    let upload = p
        .resume_upload_id
        .strip_prefix("resume_upload:")
        .unwrap_or(&p.resume_upload_id)
        .replace('`', "");
    let mut saved = state
        .db
        .client
        .query("SELECT contact_email FROM saved_job WHERE user_id=type::record('user',$uid) AND canonical_url=$url LIMIT 1;")
        .bind(("uid", uid.clone()))
        .bind(("url", canonical.clone()))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let saved_rows: Vec<serde_json::Value> = saved
        .take(0)
        .map_err(|e| AppError::Database(e.to_string()))?;
    let saved_email = saved_rows
        .first()
        .and_then(|v| v.get("contact_email"))
        .and_then(|v| v.as_str())
        .map(str::to_owned);
    let (method, resolved_email) = resolve_direct_contact_email(
        p.application_method.unwrap_or(ApplicationMethod::Web),
        p.contact_email.as_deref(),
        saved_email.as_deref(),
    )?;
    p.application_method = Some(method);
    p.contact_email = resolved_email;
    validate_text(&p)?;
    let mut profile=state.db.client.query("SELECT phone,location,linkedin_url,github_url,portfolio_url FROM candidate_profile WHERE user_id=type::record('user',$uid) LIMIT 1;").bind(("uid",uid.clone())).await.map_err(|e|AppError::Database(e.to_string()))?;
    let profiles: Vec<serde_json::Value> = profile
        .take(0)
        .map_err(|e| AppError::Database(e.to_string()))?;
    let profile: Option<ProfileRow> = profiles
        .into_iter()
        .next()
        .map(serde_json::from_value)
        .transpose()
        .map_err(|e| AppError::Database(format!("Malformed candidate profile: {e}")))?;
    let pr = profile.as_ref();
    let user = state.identity_service.get_user_by_id(&auth.user_id).await?;
    let mut name_parts = user.name.split_whitespace();
    let first = name_parts.next().unwrap_or_default().to_string();
    let last = name_parts.collect::<Vec<_>>().join(" ");
    if first.is_empty() || last.is_empty() || user.email.is_empty() {
        return Err(AppError::Validation(
            "Profile lacks required application fields".into(),
        ));
    }
    let fields = PreparedFieldsDto {
        first_name: first,
        last_name: last,
        email: user.email,
        phone: pr.and_then(|x| x.phone.clone()),
        location: pr.and_then(|x| x.location.clone()),
        linkedin_url: pr.and_then(|x| x.linkedin_url.clone()),
        github_url: pr.and_then(|x| x.github_url.clone()),
        portfolio_url: pr.and_then(|x| x.portfolio_url.clone()),
    };
    let mut stored_fields = serde_json::to_value(&fields)
        .map_err(|e| AppError::Internal(format!("Could not encode prepared fields: {e}")))?;
    if let serde_json::Value::Object(values) = &mut stored_fields {
        values.retain(|_, value| !value.is_null());
    }
    let id = id.unwrap_or_else(|| Uuid::new_v4().to_string());
    let application_method = match p.application_method.unwrap_or(ApplicationMethod::Web) {
        ApplicationMethod::Email => "email",
        ApplicationMethod::Web => "web",
    };
    let contact_email = if application_method == "email" {
        p.contact_email.as_deref().map(str::trim).map(str::to_owned)
    } else {
        None
    };
    let snapshot = serde_json::json!({"canonical_url":canonical,"resume_upload_id":upload,"company_name":p.company_name,"role_title":p.role_title,"job_description":p.job_description,"prepared_fields":fields,"application_method":application_method,"contact_email":contact_email});
    let revision = hex::encode(Sha256::digest(
        serde_json::to_vec(&snapshot).unwrap_or_default(),
    ));
    if is_new {
        let mut existing=state.db.client.query("SELECT id FROM job_application WHERE user_id=type::record('user',$uid) AND canonical_url=$url LIMIT 1;").bind(("uid",uid.clone())).bind(("url",canonical.clone())).await.map_err(|e|AppError::Database(e.to_string()))?;
        let ids: Vec<serde_json::Value> = existing
            .take(0)
            .map_err(|e| AppError::Database(e.to_string()))?;
        if let Some(existing) = ids
            .first()
            .and_then(|v| v.get("id"))
            .and_then(|v| v.as_str())
        {
            let existing = load_owned_application(&state, existing, &auth.user_id.0).await?;
            ensure_same_application_revision(&existing.revision, &revision)?;
            return Ok((StatusCode::OK, Json(existing)));
        }
    }
    completed_upload_filename(&state, &auth.user_id.0, &upload).await?;
    let site = p.site.as_str();
    let query = if is_new {
        "CREATE type::record('job_application',$id) SET user_id=type::record('user',$uid),resume_upload_id=type::record('resume_upload',$upload),site=$site,canonical_url=$canonical,source_url=$source,company_name=$company,role_title=$role,job_description=$jd,source_title=$role,retrieved_at=time::now(),revision=$revision,status='prepared',prepared_fields=$fields,application_method=$application_method,contact_email=$contact_email,reason_code=NONE,confirmation_url=NONE,created_at=time::now(),updated_at=time::now() RETURN AFTER;"
    } else {
        "UPDATE type::record('job_application',$id) SET resume_upload_id=type::record('resume_upload',$upload),site=$site,canonical_url=$canonical,source_url=$source,company_name=$company,role_title=$role,job_description=$jd,source_title=$role,retrieved_at=time::now(),revision=$revision,status='prepared',prepared_fields=$fields,application_method=$application_method,contact_email=$contact_email,reason_code=NONE,confirmation_url=NONE,updated_at=time::now() WHERE user_id=type::record('user',$uid) AND status='needs_review' RETURN AFTER;"
    };
    let result = state
        .db
        .client
        .query(query)
        .bind(("id", id.clone()))
        .bind(("uid", uid))
        .bind(("upload", upload))
        .bind(("site", site.to_string()))
        .bind(("canonical", canonical.clone()))
        .bind(("source", p.source_url))
        .bind(("company", p.company_name.clone()))
        .bind(("role", p.role_title))
        .bind(("jd", p.job_description))
        .bind(("revision", revision.clone()))
        .bind(("fields", stored_fields))
        .bind(("application_method", application_method.to_string()))
        .bind(("contact_email", contact_email))
        .await;
    let r = match result {
        Ok(r) => r,
        Err(error) if is_new => {
            let mut existing=state.db.client.query("SELECT id FROM job_application WHERE user_id=type::record('user',$uid) AND canonical_url=$url LIMIT 1;").bind(("uid",auth.user_id.0.replace("user:","").replace('`',""))).bind(("url",canonical)).await.map_err(|e|AppError::Database(e.to_string()))?;
            let ids: Vec<serde_json::Value> = existing
                .take(0)
                .map_err(|e| AppError::Database(e.to_string()))?;
            if let Some(existing) = ids
                .first()
                .and_then(|v| v.get("id"))
                .and_then(|v| v.as_str())
            {
                let existing = load_owned_application(&state, existing, &auth.user_id.0).await?;
                ensure_same_application_revision(&existing.revision, &revision)?;
                return Ok((StatusCode::OK, Json(existing)));
            }
            return Err(AppError::Conflict(format!(
                "Application write conflict: {error}"
            )));
        }
        Err(error) => {
            return Err(AppError::Conflict(format!(
                "Application write conflict: {error}"
            )))
        }
    };
    let mut r = match r.check() {
        Ok(result) => result,
        Err(error) => {
            if is_new {
                let mut existing=state.db.client.query("SELECT id FROM job_application WHERE user_id=type::record('user',$uid) AND canonical_url=$url LIMIT 1;").bind(("uid",auth.user_id.0.replace("user:","").replace('`',""))).bind(("url",canonical)).await.map_err(|e|AppError::Database(e.to_string()))?;
                let ids: Vec<serde_json::Value> = existing
                    .take(0)
                    .map_err(|e| AppError::Database(e.to_string()))?;
                if let Some(existing) = ids
                    .first()
                    .and_then(|v| v.get("id"))
                    .and_then(|v| v.as_str())
                {
                    let existing =
                        load_owned_application(&state, existing, &auth.user_id.0).await?;
                    ensure_same_application_revision(&existing.revision, &revision)?;
                    return Ok((StatusCode::OK, Json(existing)));
                }
            }
            return Err(AppError::Conflict(format!(
                "Application write conflict: {error}"
            )));
        }
    };
    let rows: Vec<serde_json::Value> = r
        .take(0)
        .map_err(|error| AppError::Database(error.to_string()))?;
    require_application_write(&rows)?;
    let response = load_owned_application(&state, &id, &auth.user_id.0).await?;
    Ok((
        if is_new {
            StatusCode::CREATED
        } else {
            StatusCode::OK
        },
        Json(response),
    ))
}
#[utoipa::path(post,path="/api/v1/applications",request_body=PrepareApplicationRequestDto,responses((status=201,body=JobApplicationResponseDto),(status=400),(status=409)))]
pub async fn prepare_application(
    State(s): State<AppState>,
    a: AuthenticatedUser,
    Json(p): Json<PrepareApplicationRequestDto>,
) -> Result<(StatusCode, Json<JobApplicationResponseDto>)> {
    prepare(s, a, p, None).await
}
#[utoipa::path(put,path="/api/v1/applications/{id}/prepare",params(("id"=String,Path)),request_body=PrepareApplicationRequestDto,responses((status=200,body=JobApplicationResponseDto)))]
pub async fn reprepare_application(
    State(s): State<AppState>,
    a: AuthenticatedUser,
    Path(id): Path<String>,
    Json(p): Json<PrepareApplicationRequestDto>,
) -> Result<(StatusCode, Json<JobApplicationResponseDto>)> {
    let old = load_owned_application(&s, &id, &a.user_id.0).await?;
    if !matches!(old.status, ApplicationStatus::NeedsReview) {
        return Err(AppError::Conflict(
            "Application must need review before prepare".into(),
        ));
    }
    prepare(s, a, p, Some(clean_id(&id))).await
}

async fn transition(
    s: &AppState,
    a: &AuthenticatedUser,
    id: &str,
    from: &str,
    to: &str,
    extra: &str,
) -> Result<()> {
    let uid = clean_id(&a.user_id.0);
    let q=format!("UPDATE type::record('job_application',$id) SET status=$to,updated_at=time::now() {extra} WHERE user_id=type::record('user',$uid) AND status=$from RETURN AFTER;");
    let mut r =
        s.db.client
            .query(q)
            .bind(("id", clean_id(id)))
            .bind(("uid", uid))
            .bind(("from", from.to_string()))
            .bind(("to", to.to_string()))
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
    let rows: Vec<serde_json::Value> = r.take(0).map_err(|e| AppError::Database(e.to_string()))?;
    if rows.is_empty() {
        return Err(AppError::Conflict("Application state changed".into()));
    }
    Ok(())
}
async fn require_completed_resume(
    state: &AppState,
    auth: &AuthenticatedUser,
    upload_id: &str,
) -> Result<()> {
    completed_upload_filename(state, &auth.user_id.0, upload_id)
        .await
        .map(|_| ())
}
#[utoipa::path(post,path="/api/v1/applications/{id}/approve",params(("id"=String,Path)),request_body=ApproveApplicationRequestDto,responses((status=200,body=JobApplicationResponseDto),(status=409)))]
pub async fn approve_application(
    State(s): State<AppState>,
    a: AuthenticatedUser,
    Path(id): Path<String>,
    Json(p): Json<ApproveApplicationRequestDto>,
) -> Result<Json<JobApplicationResponseDto>> {
    let app = load_owned_application(&s, &id, &a.user_id.0).await?;
    require_completed_resume(&s, &a, &app.resume_upload_id).await?;
    let uid = clean_id(&a.user_id.0);
    let mut r=s.db.client.query("UPDATE type::record('job_application',$id) SET status='approved',updated_at=time::now() WHERE user_id=type::record('user',$uid) AND status='prepared' AND revision=$revision RETURN AFTER;").bind(("id",clean_id(&id))).bind(("uid",uid)).bind(("revision",p.revision)).await.map_err(|e|AppError::Database(e.to_string()))?;
    let rows: Vec<serde_json::Value> = r.take(0).map_err(|e| AppError::Database(e.to_string()))?;
    if rows.is_empty() {
        return Err(AppError::Conflict(
            "Stale revision or application state changed".into(),
        ));
    }
    Ok(Json(load_owned_application(&s, &id, &a.user_id.0).await?))
}
#[utoipa::path(post,path="/api/v1/applications/{id}/start",params(("id"=String,Path)),responses((status=200,body=JobApplicationResponseDto),(status=409)))]
pub async fn start_application(
    State(s): State<AppState>,
    a: AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<Json<JobApplicationResponseDto>> {
    let app = load_owned_application(&s, &id, &a.user_id.0).await?;
    require_completed_resume(&s, &a, &app.resume_upload_id).await?;
    transition(&s, &a, &id, "approved", "submitting", "").await?;
    Ok(Json(load_owned_application(&s, &id, &a.user_id.0).await?))
}
#[utoipa::path(post,path="/api/v1/applications/{id}/block",params(("id"=String,Path)),request_body=BlockApplicationRequestDto,responses((status=200,body=JobApplicationResponseDto),(status=409)))]
pub async fn block_application(
    State(s): State<AppState>,
    a: AuthenticatedUser,
    Path(id): Path<String>,
    Json(p): Json<BlockApplicationRequestDto>,
) -> Result<Json<JobApplicationResponseDto>> {
    let app = load_owned_application(&s, &id, &a.user_id.0).await?;
    if !matches!(
        app.status,
        ApplicationStatus::Prepared | ApplicationStatus::Approved
    ) {
        return Err(AppError::Conflict(
            "Application cannot be blocked in current state".into(),
        ));
    }
    let uid = clean_id(&a.user_id.0);
    let q="UPDATE type::record('job_application',$id) SET status='needs_review',reason_code=$reason,updated_at=time::now() WHERE user_id=type::record('user',$uid) AND status IN ['prepared','approved'] RETURN AFTER;";
    let mut r =
        s.db.client
            .query(q)
            .bind(("id", clean_id(&id)))
            .bind(("uid", uid))
            .bind(("reason", p.reason_code.as_str()))
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
    let rows: Vec<serde_json::Value> = r.take(0).map_err(|e| AppError::Database(e.to_string()))?;
    if rows.is_empty() {
        return Err(AppError::Conflict("Application state changed".into()));
    }
    Ok(Json(load_owned_application(&s, &id, &a.user_id.0).await?))
}
#[utoipa::path(post,path="/api/v1/applications/{id}/result",params(("id"=String,Path)),request_body=ResultApplicationRequestDto,responses((status=200,body=JobApplicationResponseDto),(status=409)))]
pub async fn record_result(
    State(s): State<AppState>,
    a: AuthenticatedUser,
    Path(id): Path<String>,
    Json(p): Json<ResultApplicationRequestDto>,
) -> Result<Json<JobApplicationResponseDto>> {
    let app = load_owned_application(&s, &id, &a.user_id.0).await?;
    if matches!(app.status, ApplicationStatus::Submitted) {
        if !matches!(p.outcome, ApplicationOutcome::Confirmed) {
            return Err(AppError::Conflict(
                "Submitted application cannot change its recorded outcome".into(),
            ));
        }
        ensure_submitted_dispatch(&s, &a, &app).await?;
        return Ok(Json(load_owned_application(&s, &id, &a.user_id.0).await?));
    }
    if !matches!(app.status, ApplicationStatus::Submitting) {
        return Err(AppError::Conflict("Application is not submitting".into()));
    }
    match p.outcome {
        ApplicationOutcome::Confirmed => {
            let confirmation = confirmation_url_for_result(
                app.application_method,
                app.site,
                &app.canonical_url,
                p.confirmation_url.as_deref(),
            )?;
            if let Err(error) = ensure_workspace(&s, &a, &app, confirmation).await {
                let current = load_owned_application(&s, &id, &a.user_id.0).await?;
                if matches!(current.status, ApplicationStatus::Submitted) {
                    return Ok(Json(current));
                }
                return Err(error);
            }
        }
        ApplicationOutcome::Uncertain => {
            let confirmation = p
                .confirmation_url
                .as_deref()
                .map(|v| validated_confirmation_url(app.site, v, &app.canonical_url))
                .transpose()?;
            let uid = clean_id(&a.user_id.0);
            let mut result=s.db.client.query("UPDATE type::record('job_application',$id) SET status='unknown',confirmation_url=$confirmation_url,updated_at=time::now() WHERE user_id=type::record('user',$uid) AND status='submitting' RETURN AFTER;").bind(("id",clean_id(&id))).bind(("uid",uid)).bind(("confirmation_url",confirmation)).await.map_err(|e|AppError::Database(e.to_string()))?;
            let rows: Vec<serde_json::Value> = result
                .take(0)
                .map_err(|e| AppError::Database(e.to_string()))?;
            if rows.is_empty() {
                return Err(AppError::Conflict("Application state changed".into()));
            }
        }
        ApplicationOutcome::NotSubmitted => {
            transition(&s, &a, &id, "submitting", "needs_review", "").await?
        }
    }
    Ok(Json(load_owned_application(&s, &id, &a.user_id.0).await?))
}
#[utoipa::path(post,path="/api/v1/applications/{id}/reconcile",params(("id"=String,Path)),request_body=ReconcileApplicationRequestDto,responses((status=200,body=JobApplicationResponseDto),(status=409)))]
pub async fn reconcile_application(
    State(s): State<AppState>,
    a: AuthenticatedUser,
    Path(id): Path<String>,
    Json(p): Json<ReconcileApplicationRequestDto>,
) -> Result<Json<JobApplicationResponseDto>> {
    let app = load_owned_application(&s, &id, &a.user_id.0).await?;
    if p.submitted {
        if matches!(app.status, ApplicationStatus::Submitted) {
            ensure_submitted_dispatch(&s, &a, &app).await?;
            return Ok(Json(load_owned_application(&s, &id, &a.user_id.0).await?));
        }
        if !matches!(app.status, ApplicationStatus::Unknown) {
            return Err(AppError::Conflict(
                "Application cannot be reconciled from current state".into(),
            ));
        }
        let confirmation = app.confirmation_url.clone();
        if let Err(error) = ensure_workspace(&s, &a, &app, confirmation).await {
            let current = load_owned_application(&s, &id, &a.user_id.0).await?;
            if matches!(current.status, ApplicationStatus::Submitted) {
                return Ok(Json(current));
            }
            return Err(error);
        }
    } else {
        transition(&s, &a, &id, "unknown", "needs_review", "").await?;
    }
    Ok(Json(load_owned_application(&s, &id, &a.user_id.0).await?))
}
pub(crate) async fn ensure_submitted_dispatch(
    state: &AppState,
    auth: &AuthenticatedUser,
    app: &JobApplicationResponseDto,
) -> Result<()> {
    let workspace_id = app
        .workspace_id
        .as_deref()
        .ok_or_else(|| AppError::Conflict("Submitted application has no workspace".into()))?;
    let ws_id = clean_id(workspace_id);
    Uuid::parse_str(&ws_id)
        .map_err(|_| AppError::Conflict("Submitted application has an invalid workspace".into()))?;
    let uid = clean_id(&auth.user_id.0);
    let query = "
        LET $workspace = (SELECT VALUE id FROM type::record('workspace', $workspace_id) WHERE user_id = type::record('user', $uid));
        IF array::len($workspace) != 1 { THROW 'workspace_missing'; };
        LET $has_run = (SELECT VALUE id FROM workflow_run WHERE workspace_id = type::record('workspace', $workspace_id) LIMIT 1);
        LET $has_dispatch = (SELECT VALUE id FROM type::record('application_dispatch', $workspace_id));
        IF array::len($has_run) = 0 AND array::len($has_dispatch) = 0 {
            UPSERT type::record('application_dispatch', $workspace_id) SET
                workspace_id = type::record('workspace', $workspace_id),
                user_id = type::record('user', $uid),
                application_id = type::record('job_application', $application_id),
                state = 'pending', attempts = 0, due_at = time::now(),
                lease_until = NONE, lease_token = NONE, run_id = NONE, last_error = NONE,
                created_at = time::now(), updated_at = time::now();
        };
        IF array::len($has_run) = 0 {
            UPDATE type::record('workspace', $workspace_id) SET status = 'pending',
                current_phase = 'preparing_input', progress_percent = 0, updated_at = time::now()
                WHERE status = 'failed';
        };
    ";
    state
        .db
        .client
        .query(query)
        .bind(("uid", uid))
        .bind(("workspace_id", ws_id))
        .bind(("application_id", clean_id(&app.id)))
        .await
        .and_then(|r| r.check())
        .map_err(|e| AppError::Conflict(format!("Dispatch recovery failed: {e}")))?;
    Ok(())
}

pub(crate) async fn ensure_workspace(
    state: &AppState,
    auth: &AuthenticatedUser,
    app: &JobApplicationResponseDto,
    confirmation_url: Option<String>,
) -> Result<String> {
    if let Some(workspace_id) = &app.workspace_id {
        let ws_id = clean_id(workspace_id);
        let uid = clean_id(&auth.user_id.0);
        let query = "
            BEGIN TRANSACTION;
            LET $application = (UPDATE type::record('job_application', $application_id)
                SET status = 'submitted', confirmation_url = $confirmation_url, updated_at = time::now()
                WHERE user_id = type::record('user', $uid) AND status IN ['submitting', 'unknown']
                RETURN AFTER);
            IF array::len($application) != 1 { THROW 'application_state_conflict'; };
            LET $workspace = (SELECT VALUE id FROM type::record('workspace', $workspace_id)
                WHERE user_id = type::record('user', $uid));
            IF array::len($workspace) != 1 { THROW 'workspace_missing'; };
            LET $has_run = (SELECT VALUE id FROM workflow_run
                WHERE workspace_id = type::record('workspace', $workspace_id) LIMIT 1);
            LET $has_dispatch = (SELECT VALUE id FROM type::record('application_dispatch', $workspace_id));
            IF array::len($has_run) = 0 AND array::len($has_dispatch) = 0 {
                UPSERT type::record('application_dispatch', $workspace_id) SET
                    workspace_id = type::record('workspace', $workspace_id),
                    user_id = type::record('user', $uid),
                    application_id = type::record('job_application', $application_id),
                    state = 'pending', attempts = 0, due_at = time::now(),
                    lease_until = NONE, lease_token = NONE, run_id = NONE, last_error = NONE,
                    created_at = time::now(), updated_at = time::now();
            };
            IF array::len($has_run) = 0 {
                UPDATE type::record('workspace', $workspace_id) SET status = 'pending',
                    current_phase = 'preparing_input', progress_percent = 0, updated_at = time::now()
                    WHERE status = 'failed';
            };
            COMMIT TRANSACTION;
        ";
        state
            .db
            .client
            .query(query)
            .bind(("application_id", clean_id(&app.id)))
            .bind(("workspace_id", ws_id))
            .bind(("uid", uid))
            .bind(("confirmation_url", confirmation_url))
            .await
            .and_then(|r| r.check())
            .map_err(|e| AppError::Conflict(format!("Submission recovery failed: {e}")))?;
        return Ok(workspace_id.clone());
    }
    let workspace_id = Uuid::new_v4().to_string();
    let resume_id = format!("res_{workspace_id}");
    let jd_id = format!("jd_{workspace_id}");
    let upload_id = app.resume_upload_id.clone();
    let uid = auth.user_id.0.replace("user:", "").replace('`', "");
    let query = "
        BEGIN TRANSACTION;
        LET $upload = (UPDATE type::record('resume_upload', $upload_id)
            SET updated_at = time::now()
            WHERE user_id = type::record('user', $uid) AND status = 'completed'
            RETURN AFTER);
        IF array::len($upload) != 1 { THROW 'resume_upload_write_conflict'; };
        LET $object_key = (SELECT VALUE object_key FROM ONLY type::record('resume_upload', $upload_id));
        CREATE type::record('workspace', $workspace_id) SET
            user_id = type::record('user', $uid), company_name = $company,
            role_title = $role, status = 'pending', current_phase = 'preparing_input',
            progress_percent = 0, workflow_mode = 'hybrid', company_urls = [$source],
            created_at = time::now(), updated_at = time::now();
        UPSERT type::record('resume', $resume_id) SET
            workspace_id = type::record('workspace', $workspace_id), raw_text = '',
            source_upload_id = type::record('resume_upload', $upload_id),
            source_object_key = $object_key, source_filename = $filename,
            parsed_skills = [], experience = [], education = [], created_at = time::now();
        UPSERT type::record('job_description', $jd_id) SET
            workspace_id = type::record('workspace', $workspace_id), raw_text = $jd,
            jd_url = $source, source_url = $source, canonical_url = $canonical,
            title = $role, retrieved_at = time::now(), extraction_method = 'extension_capture',
            requirements = [], created_at = time::now(), updated_at = time::now();
        LET $application = (UPDATE type::record('job_application', $application_id)
            SET status = 'submitted', workspace_id = type::record('workspace', $workspace_id),
                confirmation_url = $confirmation_url, updated_at = time::now()
            WHERE user_id = type::record('user', $uid) AND status IN ['submitting', 'unknown']
            RETURN AFTER);
        IF array::len($application) != 1 { THROW 'application_state_conflict'; };
        CREATE type::record('application_dispatch', $workspace_id) SET
            workspace_id = type::record('workspace', $workspace_id),
            user_id = type::record('user', $uid),
            application_id = type::record('job_application', $application_id),
            state = 'pending', attempts = 0, due_at = time::now(),
            created_at = time::now(), updated_at = time::now();
        COMMIT TRANSACTION;
    ";
    state
        .db
        .client
        .query(query)
        .bind(("upload_id", upload_id))
        .bind(("uid", uid))
        .bind(("workspace_id", workspace_id.clone()))
        .bind(("resume_id", resume_id))
        .bind(("jd_id", jd_id))
        .bind(("company", app.company_name.clone()))
        .bind(("role", app.role_title.clone()))
        .bind(("source", app.source_url.clone()))
        .bind(("jd", app.job_description.clone()))
        .bind(("filename", app.resume_filename.clone()))
        .bind(("application_id", clean_id(&app.id)))
        .bind(("canonical", app.canonical_url.clone()))
        .bind(("confirmation_url", confirmation_url))
        .await
        .and_then(|r| r.check())
        .map_err(|e| AppError::Conflict(format!("Submission transaction failed: {e}")))?;
    Ok(workspace_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_url_discards_tracking_and_fragments() {
        assert_eq!(
            canonical_url(
                ApplicationSite::Linkedin,
                "https://www.linkedin.com/jobs/view/987654?trk=feed#details"
            )
            .unwrap(),
            "https://linkedin.com/jobs/view/987654"
        );
        assert_eq!(
            canonical_url(
                ApplicationSite::Greenhouse,
                "https://boards.greenhouse.io/acme/jobs/123?gh_src=abc"
            )
            .unwrap(),
            "https://boards.greenhouse.io/acme/123"
        );
    }

    #[test]
    fn rejects_non_https_unapproved_hosts_and_missing_job_identity() {
        assert!(canonical_url(ApplicationSite::Topcv, "http://topcv.vn/viec-lam/123").is_err());
        assert!(canonical_url(
            ApplicationSite::Topcv,
            "https://topcv.vn.evil.test/viec-lam/123"
        )
        .is_err());
        assert!(canonical_url(
            ApplicationSite::Linkedin,
            "https://linkedin.com/jobs/search"
        )
        .is_err());
        assert!(
            canonical_url(ApplicationSite::Greenhouse, "https://boards.greenhouse.io/").is_err()
        );
    }

    #[test]
    fn topcv_canonical_url_discards_trailing_path_segments() {
        assert_eq!(
            canonical_url(
                ApplicationSite::Topcv,
                "https://www.topcv.vn/viec-lam/12345-ky-su/apply?ref=tracking#form"
            )
            .unwrap(),
            "https://topcv.vn/viec-lam/12345-ky-su"
        );
        assert_eq!(
            canonical_url(
                ApplicationSite::Topcv,
                "https://topcv.vn/viec-lam-12345-ky-su/apply"
            )
            .unwrap(),
            "https://topcv.vn/viec-lam-12345-ky-su"
        );
        assert_eq!(
            canonical_url(
                ApplicationSite::Topcv,
                "https://topcv.vn/viec-lam/12345-ky-su/"
            )
            .unwrap(),
            "https://topcv.vn/viec-lam/12345-ky-su"
        );
    }

    #[test]
    fn direct_email_first_prefers_saved_recipient() {
        assert!(matches!(
            resolve_direct_contact_email(ApplicationMethod::Web, None, Some("hr@company.com")),
            Err(AppError::Conflict(_))
        ));
        assert!(matches!(
            resolve_direct_contact_email(
                ApplicationMethod::Web,
                Some("recruiter@company.com"),
                None
            ),
            Err(AppError::Validation(_))
        ));
        assert_eq!(
            resolve_direct_contact_email(ApplicationMethod::Web, None, None).unwrap(),
            (ApplicationMethod::Web, None)
        );
        assert_eq!(
            resolve_direct_contact_email(ApplicationMethod::Email, None, Some("hr@company.com"))
                .unwrap(),
            (ApplicationMethod::Email, Some("hr@company.com".to_string()))
        );
        assert_eq!(
            resolve_direct_contact_email(
                ApplicationMethod::Email,
                Some(" hr@company.com "),
                Some("hr@company.com")
            )
            .unwrap(),
            (ApplicationMethod::Email, Some("hr@company.com".to_string()))
        );
        assert!(matches!(
            resolve_direct_contact_email(
                ApplicationMethod::Email,
                Some("other@company.com"),
                Some("hr@company.com")
            ),
            Err(AppError::Conflict(_))
        ));
        assert_eq!(
            resolve_direct_contact_email(
                ApplicationMethod::Email,
                Some("candidate@example.com"),
                None
            )
            .unwrap(),
            (
                ApplicationMethod::Email,
                Some("candidate@example.com".to_string())
            )
        );
    }

    #[test]
    fn request_dtos_reject_unknown_fields() {
        assert!(serde_json::from_value::<ApproveApplicationRequestDto>(
            serde_json::json!({"revision":"abc","approved":true})
        )
        .is_err());
        assert!(serde_json::from_value::<BlockApplicationRequestDto>(
            serde_json::json!({"reason_code":"captcha","extra":"ignored"})
        )
        .is_err());
    }
}

#[cfg(test)]
#[path = "application_flow_tests.rs"]
mod application_flow_tests;
