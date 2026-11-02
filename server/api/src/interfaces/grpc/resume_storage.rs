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
        for id in [&input.workspace_id, &input.user_id, &input.resume_upload_id] {
            Uuid::parse_str(id).map_err(|_| Status::invalid_argument("Invalid record ID"))?;
        }

        let mut owned = self.db.client
            .query("SELECT id FROM type::record('workspace', $workspace_id) WHERE user_id = type::record('user', $user_id);")
            .bind(("workspace_id", input.workspace_id.clone()))
            .bind(("user_id", input.user_id.clone()))
            .await
            .map_err(|_| Status::internal("Workspace lookup failed"))?;
        let owned: Vec<serde_json::Value> = owned.take(0)
            .map_err(|_| Status::internal("Workspace lookup failed"))?;
        if owned.is_empty() {
            return Err(Status::not_found("Attached resume not found"));
        }

        let mut upload = self.db.client
            .query("SELECT filename, size_bytes, object_key FROM type::record('resume_upload', $upload_id) WHERE user_id = type::record('user', $user_id) AND workspace_id = type::record('workspace', $workspace_id) AND status = 'attached';")
            .bind(("upload_id", input.resume_upload_id.clone()))
            .bind(("user_id", input.user_id))
            .bind(("workspace_id", input.workspace_id.clone()))
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

        let resume_id = format!("res_{}", input.workspace_id);
        let mut resume = self.db.client
            .query("SELECT id FROM type::record('resume', $resume_id) WHERE workspace_id = type::record('workspace', $workspace_id) AND source_upload_id = type::record('resume_upload', $upload_id) AND source_object_key = $key;")
            .bind(("resume_id", resume_id))
            .bind(("workspace_id", input.workspace_id))
            .bind(("upload_id", input.resume_upload_id))
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
