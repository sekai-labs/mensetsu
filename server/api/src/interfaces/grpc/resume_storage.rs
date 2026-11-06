use std::sync::Arc;

use subtle::ConstantTimeEq;
use tonic::{Request, Response, Status};
use uuid::Uuid;

use super::client::proto::resume::{
    resume_storage_service_server::ResumeStorageService, GetResumeDownloadUrlRequest,
    GetResumeDownloadUrlResponse,
};
use crate::contexts::workspace::infrastructure::{
    db::DatabasePool, storage::ResumeObjectStore,
};

#[derive(Clone)]
pub struct ResumeStorageGrpcService {
    db: DatabasePool,
    storage: Arc<ResumeObjectStore>,
    authorization: Arc<str>,
}

impl ResumeStorageGrpcService {
    pub fn new(db: DatabasePool, storage: Arc<ResumeObjectStore>, service_token: &str) -> Self {
        Self {
            db,
            storage,
            authorization: format!("Bearer {service_token}").into(),
        }
    }
}

/// Accepts a UUID or a Surreal record ID qualified by its expected table.
fn unqualified_record_id<'a>(raw: &'a str, table: &str) -> Result<&'a str, Status> {
    let raw = raw.trim().trim_matches('`');
    let id = match raw.split_once(':') {
        Some((prefix, id)) if prefix == table => id.trim_matches('`'),
        Some(_) => return Err(Status::invalid_argument("Invalid record ID")),
        None => raw,
    };
    if id.is_empty() || id.contains(':') {
        return Err(Status::invalid_argument("Invalid record ID"));
    }
    Ok(id)
}

fn parse_record_uuid(raw: &str, table: &str) -> Result<String, Status> {
    let id = unqualified_record_id(raw, table)?;
    Uuid::parse_str(id)
        .map(|id| id.to_string())
        .map_err(|_| Status::invalid_argument("Invalid record ID"))
}

/// User IDs are UUIDs for ordinary accounts but may be stable named IDs.
fn parse_user_record_id(raw: &str) -> Result<String, Status> {
    let id = unqualified_record_id(raw, "user")?;
    if !id.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-') {
        return Err(Status::invalid_argument("Invalid record ID"));
    }
    Ok(id.to_owned())
}

#[cfg(test)]
mod tests {
    use super::{parse_record_uuid, parse_user_record_id};

    const ID: &str = "550e8400-e29b-41d4-a716-446655440000";

    #[test]
    fn accepts_bare_and_table_qualified_record_ids() {
        assert_eq!(parse_record_uuid(ID, "workspace").unwrap(), ID);
        assert_eq!(parse_record_uuid(&format!("workspace:{ID}"), "workspace").unwrap(), ID);
        assert_eq!(parse_record_uuid(&format!("resume_upload:`{ID}`"), "resume_upload").unwrap(), ID);
    }

    #[test]
    fn accepts_uuid_and_named_user_record_ids() {
        assert_eq!(parse_user_record_id(ID).unwrap(), ID);
        assert_eq!(parse_user_record_id("admin_mensetsu").unwrap(), "admin_mensetsu");
        assert_eq!(parse_user_record_id("user:`admin_mensetsu`").unwrap(), "admin_mensetsu");
    }

    #[test]
    fn rejects_wrong_table_and_malformed_record_ids() {
        assert!(parse_record_uuid(&format!("user:{ID}"), "workspace").is_err());
        assert!(parse_record_uuid("workspace:not-a-uuid", "workspace").is_err());
        assert!(parse_user_record_id("workspace:admin_mensetsu").is_err());
        assert!(parse_user_record_id("user:bad/id").is_err());
    }
}

#[tonic::async_trait]
impl ResumeStorageService for ResumeStorageGrpcService {
    async fn get_resume_download_url(
        &self,
        request: Request<GetResumeDownloadUrlRequest>,
    ) -> Result<Response<GetResumeDownloadUrlResponse>, Status> {
        let presented = request
            .metadata()
            .get("authorization")
            .and_then(|header| header.to_str().ok())
            .unwrap_or("");
        if !bool::from(presented.as_bytes().ct_eq(self.authorization.as_bytes())) {
            return Err(Status::unauthenticated("Invalid service credentials"));
        }

        let input = request.into_inner();
        let workspace_id = parse_record_uuid(&input.workspace_id, "workspace")?;
        let user_id = parse_user_record_id(&input.user_id)?;
        let upload_id = parse_record_uuid(&input.resume_upload_id, "resume_upload")?;

        let mut owned = self.db.client
            .query("SELECT id FROM type::record('workspace', $workspace_id) WHERE user_id = type::record('user', $user_id);")
            .bind(("workspace_id", workspace_id.clone()))
            .bind(("user_id", user_id.clone()))
            .await
            .map_err(|_| Status::internal("Workspace lookup failed"))?;
        let owned: Vec<serde_json::Value> = owned.take(0)
            .map_err(|_| Status::internal("Workspace lookup failed"))?;
        if owned.is_empty() {
            return Err(Status::not_found("Attached resume not found"));
        }

        let mut upload = self.db.client
            .query("SELECT filename, size_bytes, object_key FROM type::record('resume_upload', $upload_id) WHERE user_id = type::record('user', $user_id) AND workspace_id = type::record('workspace', $workspace_id) AND status = 'attached';")
            .bind(("upload_id", upload_id.clone()))
            .bind(("user_id", user_id))
            .bind(("workspace_id", workspace_id.clone()))
            .await
            .map_err(|_| Status::internal("Upload lookup failed"))?;
        let rows: Vec<serde_json::Value> = upload.take(0)
            .map_err(|_| Status::internal("Upload lookup failed"))?;
        let row = rows.first().ok_or_else(|| Status::not_found("Attached resume not found"))?;
        let key = row["object_key"].as_str()
            .filter(|key| key.starts_with("resumes/final/"))
            .ok_or_else(|| Status::failed_precondition("Resume object unavailable"))?;
        let filename = row["filename"].as_str()
            .ok_or_else(|| Status::failed_precondition("Resume filename unavailable"))?;
        let size_bytes = row["size_bytes"].as_u64()
            .filter(|size| (1..=5 * 1024 * 1024).contains(size))
            .ok_or_else(|| Status::failed_precondition("Invalid resume size"))?;

        let resume_id = format!("res_{workspace_id}");
        let mut resume = self.db.client
            .query("SELECT id FROM type::record('resume', $resume_id) WHERE workspace_id = type::record('workspace', $workspace_id) AND source_upload_id = type::record('resume_upload', $upload_id) AND source_object_key = $key;")
            .bind(("resume_id", resume_id))
            .bind(("workspace_id", workspace_id))
            .bind(("upload_id", upload_id))
            .bind(("key", key.to_owned()))
            .await
            .map_err(|_| Status::internal("Resume lookup failed"))?;
        let linked: Vec<serde_json::Value> = resume.take(0)
            .map_err(|_| Status::internal("Resume lookup failed"))?;
        if linked.is_empty() {
            return Err(Status::not_found("Attached resume not found"));
        }

        let url = self.storage.presign_get(key).await
            .map_err(|error| {
                tracing::error!(error = %error, "Unable to sign private resume download");
                Status::unavailable("Resume download unavailable")
            })?;
        Ok(Response::new(GetResumeDownloadUrlResponse {
            url,
            filename: filename.to_owned(),
            size_bytes,
        }))
    }
}
