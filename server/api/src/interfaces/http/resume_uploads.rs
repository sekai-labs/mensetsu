use axum::{
    body::{Body, Bytes},
    extract::{Path, State},
    http::{header, HeaderValue, StatusCode},
    response::Response,
    Json,
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use tokio::io::AsyncReadExt;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    interfaces::http::{extractors::authenticated_user::AuthenticatedUser, workspaces::AppState},
    shared::error::AppError,
};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateResumeUploadRequestDto {
    pub filename: String,
    pub size_bytes: u64,
    pub content_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateResumeUploadResponseDto {
    pub upload_id: String,
    pub upload_url: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CompleteResumeUploadResponseDto {
    pub upload_id: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ResumeUploadItemDto {
    pub upload_id: String,
    pub filename: String,
    pub size_bytes: u64,
    pub created_at: DateTime<Utc>,
}

fn sanitize_filename(basename: &str) -> String {
    let sanitized: String = basename
        .chars()
        .map(|c| {
            if c.is_alphanumeric()
                || matches!(
                    c as u32,
                    0x0300..=0x036F
                        | 0x1AB0..=0x1AFF
                        | 0x1DC0..=0x1DFF
                        | 0x20D0..=0x20FF
                        | 0xFE20..=0xFE2F
                )
                || matches!(c, '.' | '-' | '_')
            {
                c
            } else {
                '_'
            }
        })
        .collect();

    let trimmed = sanitized.trim_matches(|c| c == '.' || c == '_' || c == '-');
    if trimmed.is_empty() {
        "resume.pdf".to_string()
    } else if !trimmed.to_lowercase().ends_with(".pdf") {
        format!("{trimmed}.pdf")
    } else {
        trimmed.to_string()
    }
}

fn content_disposition_filename(filename: &str) -> String {
    let fallback: String = filename
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | '_') {
                character
            } else {
                '_'
            }
        })
        .collect();
    let mut encoded = String::with_capacity(filename.len());
    for byte in filename.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_') {
            encoded.push(char::from(byte));
        } else {
            const HEX: &[u8; 16] = b"0123456789ABCDEF";
            encoded.push('%');
            encoded.push(HEX[(byte >> 4) as usize] as char);
            encoded.push(HEX[(byte & 0x0F) as usize] as char);
        }
    }
    format!("attachment; filename=\"{fallback}\"; filename*=UTF-8''{encoded}")
}

fn validate_and_sanitize_filename(raw: &str) -> Result<String, AppError> {
    let path = std::path::Path::new(raw);
    let basename = path.file_name().and_then(|f| f.to_str()).ok_or_else(|| {
        AppError::Validation("Filename must not be empty or path-only".to_string())
    })?;

    let trimmed_basename = basename.trim();
    if trimmed_basename.is_empty() {
        return Err(AppError::Validation(
            "Filename must not be empty".to_string(),
        ));
    }

    if !trimmed_basename.to_lowercase().ends_with(".pdf") {
        return Err(AppError::Validation(
            "Original filename must have .pdf extension".to_string(),
        ));
    }

    let stem = trimmed_basename[..trimmed_basename.len() - 4]
        .trim_matches(|c| c == '.' || c == '_' || c == '-');
    if stem.is_empty() {
        return Err(AppError::Validation(
            "Filename must contain a valid name before .pdf".to_string(),
        ));
    }

    let sanitized = sanitize_filename(trimmed_basename);
    if !sanitized.to_lowercase().ends_with(".pdf") {
        return Err(AppError::Validation(
            "Sanitized filename must end with .pdf".to_string(),
        ));
    }
    Ok(sanitized)
}

#[utoipa::path(
    post,
    path = "/api/v1/resume-uploads",
    operation_id = "create_resume_upload",
    tag = "resume_uploads",
    request_body = CreateResumeUploadRequestDto,
    responses(
        (status = 201, description = "Upload initiated", body = CreateResumeUploadResponseDto),
        (status = 400, description = "Invalid request payload"),
        (status = 401, description = "Unauthorized"),
        (status = 502, description = "Storage failure")
    ),
    security(("bearerAuth" = []))
)]
pub async fn create_resume_upload(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Json(payload): Json<CreateResumeUploadRequestDto>,
) -> Result<(StatusCode, Json<CreateResumeUploadResponseDto>), AppError> {
    if payload.size_bytes == 0 || payload.size_bytes > 5 * 1024 * 1024 {
        return Err(AppError::Validation(
            "File size must be between 1 byte and 5 MiB (5242880 bytes)".to_string(),
        ));
    }

    if payload.content_type.to_lowercase() != "application/pdf" {
        return Err(AppError::Validation(
            "Only application/pdf content type is supported".to_string(),
        ));
    }

    let sanitized_filename = validate_and_sanitize_filename(&payload.filename)?;

    let user_id = auth_user.user_id.0.replace("user:", "").replace('`', "");
    let upload_id = Uuid::new_v4().to_string();
    let staging_key = format!("resumes/staging/{upload_id}.pdf");

    let upload_url = state
        .storage
        .presign_put(&staging_key, "application/pdf")
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to generate presigned PUT url");
            AppError::ExternalService("Failed to generate presigned upload URL".to_string())
        })?;

    let now = Utc::now();
    let expires_at = now + Duration::minutes(5);

    let db = &state.db.client;
    let query = "
        CREATE type::record('resume_upload', $upload_id) SET
            user_id = type::record('user', $user_id),
            staging_key = $staging_key,
            object_key = $staging_key,
            filename = $filename,
            size_bytes = $size_bytes,
            content_type = 'application/pdf',
            status = 'pending',
            created_at = time::now(),
            updated_at = time::now();
    ";

    db.query(query)
        .bind(("upload_id", upload_id.clone()))
        .bind(("user_id", user_id))
        .bind(("staging_key", staging_key))
        .bind(("filename", sanitized_filename))
        .bind(("size_bytes", payload.size_bytes as i64))
        .await
        .and_then(|res| res.check())
        .map_err(|e| AppError::Database(format!("Failed to record resume upload: {e}")))?;

    Ok((
        StatusCode::CREATED,
        Json(CreateResumeUploadResponseDto {
            upload_id,
            upload_url,
            expires_at,
        }),
    ))
}

#[derive(Deserialize)]
struct StoredUploadRecord {
    user_id: serde_json::Value,
    staging_key: Option<String>,
    size_bytes: u64,
    status: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/resume-uploads/{upload_id}/complete",
    operation_id = "complete_resume_upload",
    tag = "resume_uploads",
    params(
        ("upload_id" = String, Path, description = "Upload ID")
    ),
    responses(
        (status = 200, description = "Upload completed", body = CompleteResumeUploadResponseDto),
        (status = 400, description = "Invalid object or validation failed"),
        (status = 403, description = "Forbidden: not the owner"),
        (status = 404, description = "Upload record not found"),
        (status = 409, description = "Upload conflict or missing staging object"),
        (status = 502, description = "Storage failure")
    ),
    security(("bearerAuth" = []))
)]
pub async fn complete_resume_upload(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(raw_upload_id): Path<String>,
) -> Result<Json<CompleteResumeUploadResponseDto>, AppError> {
    let upload_id = raw_upload_id.replace("resume_upload:", "").replace('`', "");
    if Uuid::parse_str(&upload_id).is_err() {
        return Err(AppError::NotFound(format!("Upload {upload_id} not found")));
    }

    let caller_user_id = auth_user.user_id.0.replace("user:", "").replace('`', "");
    let db = &state.db.client;

    let query = "SELECT user_id, staging_key, size_bytes, status FROM type::record('resume_upload', $upload_id);";
    let mut res = db
        .query(query)
        .bind(("upload_id", upload_id.clone()))
        .await
        .map_err(|e| AppError::Database(format!("Failed to query upload record: {e}")))?;

    let rows: Vec<serde_json::Value> = res
        .take(0)
        .map_err(|e| AppError::Database(format!("Failed to take upload record: {e}")))?;

    let row = rows
        .into_iter()
        .next()
        .ok_or_else(|| AppError::NotFound(format!("Upload {upload_id} not found")))?;

    let record: StoredUploadRecord = serde_json::from_value(row)
        .map_err(|e| AppError::Database(format!("Failed to parse upload record: {e}")))?;

    let record_user_id = match record.user_id {
        serde_json::Value::String(s) => s.replace("user:", "").replace('`', ""),
        serde_json::Value::Object(map) => map
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .replace("user:", "")
            .replace('`', ""),
        _ => String::new(),
    };

    if record_user_id != caller_user_id {
        return Err(AppError::Forbidden(
            "You do not own this upload".to_string(),
        ));
    }

    if record.status == "completed" {
        return Ok(Json(CompleteResumeUploadResponseDto {
            upload_id,
            status: "completed".to_string(),
        }));
    }

    if record.status != "pending" {
        return Err(AppError::Conflict(format!(
            "Upload is in state '{}', cannot be completed",
            record.status
        )));
    }
    let staging_key = record
        .staging_key
        .filter(|k| !k.trim().is_empty())
        .ok_or_else(|| {
            AppError::Conflict("Staging object key not recorded for upload".to_string())
        })?;

    let (staging_size, staging_content_type) =
        state
            .storage
            .head(&staging_key)
            .await
            .map_err(|e| match e {
                AppError::NotFound(_) => {
                    AppError::Conflict("Staging object not found in storage".to_string())
                }
                AppError::ExternalService(msg) => AppError::ExternalService(msg),
                other => AppError::ExternalService(other.to_string()),
            })?;

    if staging_size != record.size_bytes {
        return Err(AppError::Validation(format!(
            "Staged object size ({staging_size} bytes) does not match declared size ({} bytes)",
            record.size_bytes
        )));
    }

    if !(1..=5 * 1024 * 1024).contains(&staging_size) {
        return Err(AppError::Validation(format!(
            "Staged object size must be between 1 and 5242880 bytes, got {staging_size}"
        )));
    }

    let ct = staging_content_type.ok_or_else(|| {
        AppError::Validation("Staged object is missing Content-Type header".to_string())
    })?;
    if ct.to_lowercase() != "application/pdf" {
        return Err(AppError::Validation(format!(
            "Staged object Content-Type must be application/pdf, got {ct}"
        )));
    }

    let final_uuid = Uuid::new_v4().to_string();
    let final_key = format!("resumes/final/{final_uuid}.pdf");

    state
        .storage
        .copy(&staging_key, &final_key)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to copy staging object to final key");
            AppError::ExternalService("Failed to copy staged object to final location".to_string())
        })?;

    let head_and_prefix_result = async {
        let (final_size, final_ct) = state.storage.head(&final_key).await?;
        if final_size != record.size_bytes {
            return Err(AppError::Validation(format!(
                "Final object size mismatch: expected {}, got {final_size}",
                record.size_bytes
            )));
        }
        let ct = final_ct.ok_or_else(|| {
            AppError::Validation("Final object is missing Content-Type header".to_string())
        })?;
        if ct.to_lowercase() != "application/pdf" {
            return Err(AppError::Validation(format!(
                "Final object Content-Type must be application/pdf, got {ct}"
            )));
        }
        let prefix = state.storage.get_prefix(&final_key, 4).await?;
        if prefix.len() < 4 || &prefix[..4] != b"%PDF" {
            return Err(AppError::Validation(
                "Uploaded file is not a valid PDF (magic bytes %PDF missing)".to_string(),
            ));
        }
        Ok(())
    }
    .await;

    if let Err(err) = head_and_prefix_result {
        let _ = state.storage.delete(&final_key).await;
        return Err(match err {
            AppError::Validation(msg) => AppError::Validation(msg),
            AppError::NotFound(msg) => AppError::Conflict(msg),
            AppError::ExternalService(msg) => AppError::ExternalService(msg),
            other => AppError::ExternalService(other.to_string()),
        });
    }

    let cas_query = "
        UPDATE type::record('resume_upload', $upload_id) SET
            object_key = $final_key,
            status = 'completed',
            updated_at = time::now()
        WHERE status = 'pending';
    ";

    let mut update_res = match db
        .query(cas_query)
        .bind(("upload_id", upload_id.clone()))
        .bind(("final_key", final_key.clone()))
        .await
    {
        Ok(res) => res,
        Err(e) => {
            let _ = state.storage.delete(&final_key).await;
            return Err(AppError::Database(format!(
                "Failed to execute CAS update on upload: {e}"
            )));
        }
    };

    let updated_rows: Vec<serde_json::Value> = match update_res.take(0) {
        Ok(rows) => rows,
        Err(e) => {
            let _ = state.storage.delete(&final_key).await;
            return Err(AppError::Database(format!(
                "Failed to take CAS update result: {e}"
            )));
        }
    };

    if updated_rows.is_empty() {
        let _ = state.storage.delete(&final_key).await;

        let mut check_res = db
            .query("SELECT status FROM type::record('resume_upload', $upload_id);")
            .bind(("upload_id", upload_id.clone()))
            .await
            .map_err(|e| {
                AppError::Database(format!("Failed to query upload status on race: {e}"))
            })?;
        let check_rows: Vec<serde_json::Value> = check_res.take(0).map_err(|e| {
            AppError::Database(format!("Failed to read upload status on race: {e}"))
        })?;
        if let Some(r) = check_rows.into_iter().next() {
            if let Some(s) = r.get("status").and_then(|v| v.as_str()) {
                if s == "completed" {
                    return Ok(Json(CompleteResumeUploadResponseDto {
                        upload_id,
                        status: "completed".to_string(),
                    }));
                }
            }
        }

        return Err(AppError::Conflict("Upload completion conflict".to_string()));
    }

    Ok(Json(CompleteResumeUploadResponseDto {
        upload_id,
        status: "completed".to_string(),
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/resume-uploads",
    operation_id = "list_resume_uploads",
    tag = "resume_uploads",
    responses(
        (status = 200, description = "List completed CV uploads", body = [ResumeUploadItemDto]),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearerAuth" = []))
)]
pub async fn list_resume_uploads(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<Json<Vec<ResumeUploadItemDto>>, AppError> {
    let user_id = auth_user.user_id.0.replace("user:", "").replace('`', "");
    let mut result = state.db.client
        .query("SELECT id, filename, size_bytes, created_at FROM resume_upload WHERE user_id = type::record('user', $user_id) AND status = 'completed' ORDER BY created_at DESC;")
        .bind(("user_id", user_id))
        .await
        .map_err(|e| AppError::Database(format!("Failed to list resume uploads: {e}")))?;
    let rows: Vec<serde_json::Value> = result
        .take(0)
        .map_err(|e| AppError::Database(format!("Failed to read resume uploads: {e}")))?;
    rows.into_iter()
        .map(|row| {
            let record_id = row
                .get("id")
                .ok_or_else(|| AppError::Database("Resume upload is missing its ID".into()))?;
            let raw_id = match record_id {
                serde_json::Value::String(value) => value.clone(),
                serde_json::Value::Object(value) => value
                    .get("id")
                    .and_then(|id| id.as_str())
                    .map(str::to_owned)
                    .ok_or_else(|| {
                        AppError::Database("Resume upload ID has an unexpected shape".into())
                    })?,
                _ => {
                    return Err(AppError::Database(
                        "Resume upload ID has an unexpected shape".into(),
                    ))
                }
            };
            let upload_id = raw_id
                .strip_prefix("resume_upload:")
                .unwrap_or(&raw_id)
                .trim_matches('`')
                .to_owned();
            serde_json::from_value(serde_json::json!({
                "upload_id": upload_id,
                "filename": row.get("filename").cloned().unwrap_or_default(),
                "size_bytes": row.get("size_bytes").cloned().unwrap_or_default(),
                "created_at": row.get("created_at").cloned().unwrap_or_default()
            }))
            .map_err(|e| AppError::Database(format!("Invalid resume upload row: {e}")))
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Json)
}

#[utoipa::path(
    get,
    path = "/api/v1/resume-uploads/{upload_id}/download",
    operation_id = "download_resume_upload",
    tag = "resume_uploads",
    params(("upload_id" = String, Path)),
    responses(
        (status = 200, description = "Download PDF", content_type = "application/pdf"),
        (status = 404, description = "Upload not found"),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearerAuth" = []))
)]
pub async fn download_resume_upload(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(raw_upload_id): Path<String>,
) -> Result<Response<Body>, AppError> {
    let upload_id = raw_upload_id.replace("resume_upload:", "").replace('`', "");
    if Uuid::parse_str(&upload_id).is_err() {
        return Err(AppError::NotFound(format!("Upload {upload_id} not found")));
    }
    let user_id = auth_user.user_id.0.replace("user:", "").replace('`', "");
    let mut result = state.db.client
        .query("SELECT filename, object_key FROM type::record('resume_upload', $upload_id) WHERE user_id = type::record('user', $user_id) AND status = 'completed';")
        .bind(("upload_id", upload_id.clone()))
        .bind(("user_id", user_id))
        .await
        .map_err(|e| AppError::Database(format!("Failed to load resume upload: {e}")))?;
    let rows: Vec<serde_json::Value> = result
        .take(0)
        .map_err(|e| AppError::Database(format!("Failed to read resume upload: {e}")))?;
    let row = rows
        .first()
        .ok_or_else(|| AppError::NotFound(format!("Upload {upload_id} not found")))?;
    let key = row
        .get("object_key")
        .and_then(|v| v.as_str())
        .filter(|key| key.starts_with("resumes/final/"))
        .ok_or_else(|| AppError::Conflict("Resume object unavailable".into()))?;
    let filename = row
        .get("filename")
        .and_then(|v| v.as_str())
        .unwrap_or("resume.pdf");
    let safe_filename = sanitize_filename(filename);
    let stream = state.storage.get_pdf_stream(key).await?;
    let mut reader = stream.into_async_read();
    let body_stream = async_stream::stream! {
        let mut chunk = vec![0u8; 64 * 1024];
        loop {
            match reader.read(&mut chunk).await {
                Ok(0) => break,
                Ok(read) => yield Ok::<Bytes, std::io::Error>(Bytes::copy_from_slice(&chunk[..read])),
                Err(error) => {
                    yield Err(error);
                    break;
                }
            }
        }
    };
    let disposition = HeaderValue::from_str(&content_disposition_filename(&safe_filename))
        .map_err(|e| AppError::Internal(format!("Invalid resume filename header: {e}")))?;
    Response::builder()
        .header(header::CONTENT_TYPE, "application/pdf")
        .header(header::CONTENT_DISPOSITION, disposition)
        .header(header::CACHE_CONTROL, "no-store")
        .body(Body::from_stream(body_stream))
        .map_err(|e| AppError::Internal(format!("Failed to build resume response: {e}")))
}

#[utoipa::path(
    delete,
    path = "/api/v1/resume-uploads/{upload_id}",
    operation_id = "delete_resume_upload",
    tag = "resume_uploads",
    params(("upload_id" = String, Path)),
    responses(
        (status = 204, description = "CV deleted"),
        (status = 404, description = "Upload not found"),
        (status = 409, description = "CV is referenced by a workspace or application"),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearerAuth" = []))
)]
pub async fn delete_resume_upload(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(raw_upload_id): Path<String>,
) -> Result<StatusCode, AppError> {
    let upload_id = raw_upload_id.replace("resume_upload:", "").replace('`', "");
    if Uuid::parse_str(&upload_id).is_err() {
        return Err(AppError::NotFound(format!("Upload {upload_id} not found")));
    }
    let user_id = auth_user.user_id.0.replace("user:", "").replace('`', "");
    let result = state.db.client.query(
        "BEGIN TRANSACTION;\
         LET $upload = type::record('resume_upload', $upload_id);\
         LET $owned = SELECT VALUE id FROM resume_upload:$upload_id WHERE user_id = type::record('user', $user_id) AND status IN ['completed', 'deleting'];\
         IF array::len($owned) = 0 { THROW 'resume_upload_not_found'; };\
         LET $resume_refs = SELECT VALUE id FROM resume WHERE source_upload_id = $upload;\
         LET $application_refs = SELECT VALUE id FROM job_application WHERE resume_upload_id = $upload;\
         IF array::len($resume_refs) + array::len($application_refs) > 0 { THROW 'resume_upload_referenced'; };\
         UPDATE resume_upload:$upload_id SET status = 'deleting';\
         COMMIT TRANSACTION;\
         RETURN 'deleting';"
    ).bind(("upload_id", upload_id.clone()))
        .bind(("user_id", user_id.clone()))
        .await
        .map_err(|e| {
            let message = e.to_string();
            if message.contains("resume_upload_referenced") {
                AppError::Conflict("Resume upload is referenced by a resume or job application".into())
            } else if message.contains("resume_upload_not_found") {
                AppError::NotFound(format!("Upload {upload_id} not found"))
            } else {
                AppError::Database(format!("Failed to begin resume deletion: {e}"))
            }
        })?;
    result
        .check()
        .map_err(|e| AppError::Database(format!("Failed to guard resume deletion: {e}")))?;
    let mut lookup = state.db.client
        .query("SELECT object_key FROM type::record('resume_upload', $upload_id) WHERE user_id = type::record('user', $user_id) AND status = 'deleting';")
        .bind(("upload_id", upload_id.clone()))
        .bind(("user_id", user_id))
        .await
        .map_err(|e| AppError::Database(format!("Failed to load deleting resume: {e}")))?;
    let rows: Vec<serde_json::Value> = lookup
        .take(0)
        .map_err(|e| AppError::Database(format!("Failed to read deleting resume: {e}")))?;
    let key = rows
        .first()
        .and_then(|row| row.get("object_key"))
        .and_then(|v| v.as_str())
        .filter(|key| key.starts_with("resumes/final/"))
        .ok_or_else(|| AppError::Conflict("Resume object unavailable".into()))?;
    if let Err(error) = state.storage.delete(key).await {
        let _ = state.db.client.query("UPDATE type::record('resume_upload', $upload_id) SET status = 'completed' WHERE status = 'deleting';")
            .bind(("upload_id", upload_id))
            .await;
        return Err(error);
    }
    state
        .db
        .client
        .query("DELETE type::record('resume_upload', $upload_id) WHERE status = 'deleting';")
        .bind(("upload_id", upload_id))
        .await
        .and_then(|result| result.check())
        .map_err(|e| AppError::Database(format!("Failed to remove resume upload metadata: {e}")))?;
    Ok(StatusCode::NO_CONTENT)
}
#[cfg(test)]
mod filename_sanitization_tests {
    use super::validate_and_sanitize_filename;

    #[test]
    fn preserves_safe_unicode_resume_names() {
        assert_eq!(
            validate_and_sanitize_filename("履歴書.pdf").unwrap(),
            "履歴書.pdf"
        );
        assert_eq!(
            validate_and_sanitize_filename("résumé.pdf").unwrap(),
            "résumé.pdf"
        );
        assert_eq!(
            validate_and_sanitize_filename("sơ yếu lý lịch.pdf").unwrap(),
            "sơ_yếu_lý_lịch.pdf"
        );
    }

    #[test]
    fn preserves_decomposed_combining_marks() {
        assert_eq!(
            validate_and_sanitize_filename("e\u{301}.pdf").unwrap(),
            "e\u{301}.pdf"
        );
    }
}
#[cfg(test)]
mod filename_content_disposition_tests {
    use super::content_disposition_filename;

    #[test]
    fn emits_ascii_fallback_and_utf8_extended_filename() {
        assert_eq!(
            content_disposition_filename("résumé.pdf"),
            "attachment; filename=\"r_sum_.pdf\"; filename*=UTF-8''r%C3%A9sum%C3%A9.pdf"
        );
    }
}
