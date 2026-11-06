use std::sync::Arc;

use mensetsu_api::{
    contexts::workspace::infrastructure::{db::DatabasePool, storage::ResumeObjectStore},
    interfaces::grpc::{
        proto::resume::{
            resume_storage_service_server::ResumeStorageService,
            GetResumeDownloadUrlRequest,
        },
        resume_storage::ResumeStorageGrpcService,
    },
    shared::config::AppConfig,
};
use tonic::{Code, Request};

#[tokio::test]
async fn download_url_requires_service_credentials_and_owned_attached_resume() {
    let mut config = AppConfig::from_env();
    config.internal_service_token = "test-storage-service-credential".to_string();
    let db = DatabasePool::connect(&config).await.expect("SurrealDB must be reachable");
    let store = Arc::new(ResumeObjectStore::from_config(&config).await.expect("S3 signer must initialize"));
    let service = ResumeStorageGrpcService::new(db, store, &config.internal_service_token);
    let lookup = GetResumeDownloadUrlRequest {
        workspace_id: uuid::Uuid::new_v4().to_string(),
        user_id: uuid::Uuid::new_v4().to_string(),
        resume_upload_id: uuid::Uuid::new_v4().to_string(),
    };

    let unauthenticated = service.get_resume_download_url(Request::new(lookup.clone())).await
        .expect_err("A worker with no service token must not receive a signed URL");
    assert_eq!(unauthenticated.code(), Code::Unauthenticated);

    let mut wrong_token = Request::new(lookup.clone());
    wrong_token.metadata_mut().insert("authorization", "Bearer invalid".parse().unwrap());
    let rejected = service.get_resume_download_url(wrong_token).await
        .expect_err("An incorrect service token must not receive a signed URL");
    assert_eq!(rejected.code(), Code::Unauthenticated);

    let mut owned_lookup = Request::new(lookup.clone());
    owned_lookup.metadata_mut().insert(
        "authorization", "Bearer test-storage-service-credential".parse().unwrap(),
    );
    let missing = service.get_resume_download_url(owned_lookup).await
        .expect_err("No owned attached PDF exists for unrelated record IDs");
    assert_eq!(missing.code(), Code::NotFound);
    let mut prefixed_lookup = Request::new(GetResumeDownloadUrlRequest {
        workspace_id: format!("workspace:{}", lookup.workspace_id),
        user_id: "user:admin_mensetsu".to_string(),
        resume_upload_id: format!("resume_upload:{}", lookup.resume_upload_id),
    });
    prefixed_lookup.metadata_mut().insert(
        "authorization", "Bearer test-storage-service-credential".parse().unwrap(),
    );
    let missing_prefixed = service.get_resume_download_url(prefixed_lookup).await
        .expect_err("Table-qualified Surreal record IDs should be normalized before lookup");
    assert_eq!(missing_prefixed.code(), Code::NotFound);
}
