use std::time::Duration;
use aws_config::BehaviorVersion;
use aws_sdk_s3::{
    config::Region,
    presigning::PresigningConfig,
    primitives::ByteStream,
    Client,
};
use crate::shared::{config::AppConfig, error::AppError};

#[derive(Clone)]
pub struct ResumeObjectStore {
    bucket: String,
    internal_client: Client,
    public_client: Client,
}

impl ResumeObjectStore {
    pub async fn from_config(config: &AppConfig) -> Result<Self, AppError> {
        let region = Region::new(config.aws_region.clone());
        let shared_config = aws_config::defaults(BehaviorVersion::latest())
            .region(region.clone())
            .load()
            .await;

        let make_client = |custom_endpoint: Option<&String>| -> Client {
            let mut s3_conf_builder = aws_sdk_s3::config::Builder::from(&shared_config)
                .region(region.clone());

            if let Some(endpoint) = custom_endpoint {
                s3_conf_builder = s3_conf_builder
                    .endpoint_url(endpoint)
                    .force_path_style(true);
            }
            Client::from_conf(s3_conf_builder.build())
        };

        let internal_client = make_client(config.s3_endpoint_internal.as_ref());
        let public_client = match &config.s3_endpoint_public {
            Some(_) => make_client(config.s3_endpoint_public.as_ref()),
            None => internal_client.clone(),
        };

        Ok(Self {
            bucket: config.s3_bucket.clone(),
            internal_client,
            public_client,
        })
    }

            pub async fn presign_put(&self, key: &str, content_type: &str) -> Result<String, AppError> {
        let presign_config = PresigningConfig::expires_in(Duration::from_secs(300))
            .map_err(|e| AppError::Internal(format!("Failed to build presign config: {e}")))?;

        let presigned = self
            .public_client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .content_type(content_type)
            .presigned(presign_config)
            .await
            .map_err(|e| AppError::ExternalService(format!("Failed to presign PUT request: {e}")))?;

        Ok(presigned.uri().to_string())
    }

            pub async fn presign_get(&self, key: &str) -> Result<String, AppError> {
        let presign_config = PresigningConfig::expires_in(Duration::from_secs(300))
            .map_err(|e| AppError::Internal(format!("Failed to build presign config: {e}")))?;

        let presigned = self
            .internal_client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .presigned(presign_config)
            .await
            .map_err(|e| AppError::ExternalService(format!("Failed to presign GET request: {e}")))?;

        Ok(presigned.uri().to_string())
    }

        pub async fn head(&self, key: &str) -> Result<(u64, Option<String>), AppError> {
        let resp = self
            .internal_client
            .head_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| match e.into_service_error() {
                err if err.is_not_found() => AppError::NotFound(format!("Object not found: {key}")),
                err => AppError::ExternalService(format!("Failed to head object {key}: {err}")),
            })?;

        let size = resp.content_length().unwrap_or(0) as u64;
        let content_type = resp.content_type().map(|s| s.to_string());
        Ok((size, content_type))
    }

        pub async fn get_prefix(&self, key: &str, len: u64) -> Result<Vec<u8>, AppError> {
        if len == 0 {
            return Ok(Vec::new());
        }
        let range = format!("bytes=0-{}", len - 1);
        let resp = self
            .internal_client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .range(range)
            .send()
            .await
            .map_err(|e| AppError::ExternalService(format!("Failed to get prefix for {key}: {e}")))?;

        let data = resp
            .body
            .collect()
            .await
            .map_err(|e| AppError::ExternalService(format!("Failed to read stream for {key}: {e}")))?;

        Ok(data.to_vec())
    }

        pub async fn copy(&self, from: &str, to: &str) -> Result<(), AppError> {
        let copy_source = format!("{}/{}", self.bucket, from);
        self.internal_client
            .copy_object()
            .bucket(&self.bucket)
            .copy_source(copy_source)
            .key(to)
            .send()
            .await
            .map_err(|e| AppError::ExternalService(format!("Failed to copy {from} to {to}: {e}")))?;

        Ok(())
    }

        pub async fn delete(&self, key: &str) -> Result<(), AppError> {
        self.internal_client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| AppError::ExternalService(format!("Failed to delete {key}: {e}")))?;

        Ok(())
    }

        pub async fn put_pdf_bytes(&self, key: &str, bytes: Vec<u8>) -> Result<(), AppError> {
        self.internal_client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .content_type("application/pdf")
            .body(ByteStream::from(bytes))
            .send()
            .await
            .map_err(|e| AppError::ExternalService(format!("Failed to put object {key}: {e}")))?;

        Ok(())
    }
}
