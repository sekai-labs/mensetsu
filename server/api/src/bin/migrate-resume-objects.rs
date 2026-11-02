use std::sync::Arc;
use base64::prelude::*;
use surrealdb::engine::remote::ws::Ws;
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use uuid::Uuid;

use mensetsu_api::{
    contexts::workspace::infrastructure::storage::ResumeObjectStore,
    shared::config::AppConfig,
};

#[derive(serde::Deserialize, Debug)]
struct LegacyResumeRecord {
    id: String,
    workspace_id: String,
    source_pdf_base64: Option<String>,
    source_filename: Option<String>,
    source_upload_id: Option<String>,
    source_object_key: Option<String>,
}

#[derive(serde::Deserialize, Debug)]
struct WorkspaceOwner {
    user_id: String,
}
fn record_key<'a>(record: &'a str, table: &str) -> Result<&'a str, Box<dyn std::error::Error>> {
    let key = record.strip_prefix(table).and_then(|value| value.strip_prefix(':'))
        .ok_or_else(|| format!("Unexpected record ID {record}; expected table {table}"))?;
    Ok(key.strip_prefix('`').and_then(|value| value.strip_suffix('`')).unwrap_or(key))
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let config = AppConfig::from_env();
    println!("Connecting to SurrealDB at {}", config.surreal_url);

    let trimmed_url = config
        .surreal_url
        .trim_start_matches("ws://")
        .trim_start_matches("http://");

    let client = Surreal::new::<Ws>(trimmed_url).await?;
    client
        .signin(Root {
            username: config.surreal_user.clone(),
            password: config.surreal_pass.clone(),
        })
        .await?;

    client
        .use_ns(&config.surreal_ns)
        .use_db(&config.surreal_db)
        .await?;

    let storage = Arc::new(ResumeObjectStore::from_config(&config).await?);

    println!("Querying legacy resume records with non-empty source_pdf_base64...");

    let query = "SELECT id, workspace_id, source_pdf_base64, source_filename, source_upload_id, source_object_key FROM resume WHERE source_pdf_base64 != NONE AND source_pdf_base64 != NULL;";
    let mut resp = client.query(query).await?;
    let rows: Vec<serde_json::Value> = resp.take(0)?;
    let records: Vec<LegacyResumeRecord> = rows.into_iter()
        .map(serde_json::from_value).collect::<Result<_, _>>()?;

    println!("Found {} candidate record(s) to inspect.", records.len());

    let mut total_migrated = 0;
    let mut total_skipped = 0;
    let mut failed_ids = Vec::new();

    for record in records {
        let resume_id_str = record.id.to_string();

        let base64_str = match &record.source_pdf_base64 {
            Some(s) if !s.is_empty() => s.trim(),
            _ => {
                total_skipped += 1;
                continue;
            }
        };

        // If already has source_upload_id and source_object_key, verify object existence and upload record before clearing
        if let (Some(upload_id), Some(object_key)) = (&record.source_upload_id, &record.source_object_key) {
            if base64_str.len() > 7_500_000 {
                eprintln!("Record {} has oversized legacy base64; preserving it.", resume_id_str);
                failed_ids.push(resume_id_str);
                continue;
            }
            let original = match BASE64_STANDARD.decode(base64_str.as_bytes()) {
                Ok(bytes) if (4..=5 * 1024 * 1024).contains(&bytes.len()) && bytes.starts_with(b"%PDF") => bytes,
                _ => {
                    eprintln!("Record {} has invalid legacy PDF; preserving it.", resume_id_str);
                    failed_ids.push(resume_id_str);
                    continue;
                }
            };
            println!("Record {} has upload/object references ({}, {}); verifying before clearing...", resume_id_str, upload_id, object_key);

            // 1. Verify object exists in S3 via HEAD
            let head_result = storage.head(object_key).await;
            let verified_size = match head_result {
                Ok((size, _)) if size == original.len() as u64 => size,
                Ok((size, _)) => {
                    eprintln!("Record {} linked object {} has invalid size ({} bytes). Preserving base64.", resume_id_str, object_key, size);
                    failed_ids.push(resume_id_str);
                    continue;
                }
                Err(e) => {
                    eprintln!("Record {} linked object {} HEAD failed: {}. Preserving base64.", resume_id_str, object_key, e);
                    failed_ids.push(resume_id_str);
                    continue;
                }
            };

            // 2. Verify resume_upload record exists, matches owner/workspace, and size
            let mut up_check_resp = client
                .query("SELECT user_id, workspace_id, object_key, size_bytes, status FROM type::record('resume_upload', $id);")
                .bind(("id", record_key(upload_id, "resume_upload")?))
                .await?;

            #[derive(serde::Deserialize)]
            struct UploadCheck {
                user_id: String,
                workspace_id: Option<String>,
                object_key: String,
                size_bytes: u64,
                status: String,
            }

            let rows: Vec<serde_json::Value> = up_check_resp.take(0)?;
            let uploads: Vec<UploadCheck> = rows.into_iter()
                .map(serde_json::from_value).collect::<Result<_, _>>()?;
            let upload = match uploads.first() {
                Some(u) => u,
                None => {
                    eprintln!("Record {} linked upload {} does not exist in DB. Preserving base64.", resume_id_str, upload_id);
                    failed_ids.push(resume_id_str);
                    continue;
                }
            };

            let mut owner_response = client
                .query("SELECT user_id FROM type::record('workspace', $id);")
                .bind(("id", record_key(&record.workspace_id, "workspace")?))
                .await?;
            let rows: Vec<serde_json::Value> = owner_response.take(0)?;
            let owners: Vec<WorkspaceOwner> = rows.into_iter()
                .map(serde_json::from_value).collect::<Result<_, _>>()?;
            let correct_owner = owners.first().is_some_and(|owner| owner.user_id == upload.user_id);
            if !correct_owner
                || upload.workspace_id.as_ref() != Some(&record.workspace_id)
                || upload.object_key != *object_key
                || !object_key.starts_with("resumes/final/")
            {
                eprintln!("Record {} upload owner, workspace, or object key does not match; preserving base64.", resume_id_str);
                failed_ids.push(resume_id_str);
                continue;
            }
            if upload.size_bytes != verified_size || upload.status != "attached" {
                eprintln!("Record {} linked upload {} has mismatching size or status (status={}, size={}, s3_size={}). Preserving base64.",
                    resume_id_str, upload_id, upload.status, upload.size_bytes, verified_size);
                failed_ids.push(resume_id_str);
                continue;
            }

            // Verified! Safe to clear legacy base64
            let update_q = "UPDATE type::record('resume', $id) SET source_pdf_base64 = NONE;";
            let mut up_resp = client
                .query(update_q)
                .bind(("id", record_key(&record.id, "resume")?))
                .await?;
            let _check: Option<serde_json::Value> = up_resp.take(0)?;
            println!("Successfully verified and cleared base64 for already-migrated record {}", resume_id_str);
            total_migrated += 1;
            continue;
        }

        // Guard base64 encoded length before decoding to prevent unbounded allocation.
        // 5 MiB binary = ~7,000,000 base64 chars (5 * 1024 * 1024 * 4 / 3 = 6,990,507 chars).
        const MAX_BASE64_CHARS: usize = 7_500_000;
        if base64_str.len() > MAX_BASE64_CHARS {
            eprintln!(
                "Base64 length ({} chars) for resume record {} exceeds maximum allowed ({} chars). Preserving.",
                base64_str.len(),
                resume_id_str,
                MAX_BASE64_CHARS
            );
            failed_ids.push(resume_id_str);
            continue;
        }


        // 1. Decode base64
        let bytes = match BASE64_STANDARD.decode(base64_str.as_bytes()) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("Failed to decode base64 for resume record {}: {}", resume_id_str, e);
                failed_ids.push(resume_id_str);
                continue;
            }
        };

        // 2. Validate size: <= 5 MiB (5,242,880 bytes) and > 0 bytes
        const MAX_SIZE: usize = 5 * 1024 * 1024;
        if bytes.is_empty() || bytes.len() > MAX_SIZE {
            eprintln!(
                "Invalid size ({} bytes) for resume record {}: must be between 1 and {} bytes",
                bytes.len(),
                resume_id_str,
                MAX_SIZE
            );
            failed_ids.push(resume_id_str);
            continue;
        }

        // 3. Validate %PDF magic (%PDF = 0x25, 0x50, 0x44, 0x46)
        if bytes.len() < 4 || &bytes[0..4] != b"%PDF" {
            eprintln!("Invalid magic bytes (not a PDF) for resume record {}", resume_id_str);
            failed_ids.push(resume_id_str);
            continue;
        }

        let pdf_len = bytes.len() as u64;

        // 4. Fetch workspace owner user_id
        let mut ws_resp = client
            .query("SELECT user_id FROM type::record('workspace', $id);")
            .bind(("id", record_key(&record.workspace_id, "workspace")?))
            .await?;
        let rows: Vec<serde_json::Value> = ws_resp.take(0)?;
        let owners: Vec<WorkspaceOwner> = rows.into_iter()
            .map(serde_json::from_value).collect::<Result<_, _>>()?;
        let owner = match owners.first() {
            Some(o) => o,
            None => {
                eprintln!(
                    "Workspace {} not found for resume record {}",
                    record.workspace_id, resume_id_str
                );
                failed_ids.push(resume_id_str);
                continue;
            }
        };

        // 5. Generate final S3 key
        let upload_uuid = Uuid::new_v4();
        let final_key = format!("resumes/final/{}.pdf", upload_uuid);

        // 6. Write to private S3
        if let Err(e) = storage.put_pdf_bytes(&final_key, bytes).await {
            eprintln!("Failed to upload PDF to S3 for resume record {}: {}", resume_id_str, e);
            failed_ids.push(resume_id_str);
            continue;
        }

        // 7. HEAD-verify size
        let head_result = storage.head(&final_key).await;
        match head_result {
            Ok((head_size, _)) if head_size == pdf_len => {
                // Verified
            }
            Ok((head_size, _)) => {
                eprintln!(
                    "HEAD size mismatch for resume record {}: expected {} bytes, got {}",
                    resume_id_str, pdf_len, head_size
                );
                failed_ids.push(resume_id_str);
                continue;
            }
            Err(e) => {
                eprintln!("HEAD verification failed for resume record {}: {}", resume_id_str, e);
                failed_ids.push(resume_id_str);
                continue;
            }
        }

        // 8. Create resume_upload record with status 'attached'
        let filename = record
            .source_filename
            .as_deref()
            .unwrap_or("migrated_resume.pdf");

        let upload_id = upload_uuid.to_string();
        let create_upload_q = "CREATE type::record('resume_upload', $upload_id) SET
            user_id = type::record('user', $user_id),
            workspace_id = type::record('workspace', $workspace_id),
            object_key = $object_key,
            filename = $filename,
            size_bytes = $size_bytes,
            content_type = 'application/pdf',
            status = 'attached',
            created_at = time::now(),
            updated_at = time::now();";

        let mut create_resp = client
            .query(create_upload_q)
            .bind(("upload_id", upload_id.clone()))
            .bind(("user_id", record_key(&owner.user_id, "user")?))
            .bind(("workspace_id", record_key(&record.workspace_id, "workspace")?))
            .bind(("object_key", final_key.clone()))
            .bind(("filename", filename.to_string()))
            .bind(("size_bytes", pdf_len))
            .await?;
        let _check: Option<serde_json::Value> = create_resp.take(0)?;

        // 9. Update resume record with source_upload_id and source_object_key, and clear source_pdf_base64
        let update_resume_q = "UPDATE type::record('resume', $id) SET
            source_upload_id = type::record('resume_upload', $upload_id),
            source_object_key = $object_key,
            source_pdf_base64 = NONE;";

        let mut update_resp = client
            .query(update_resume_q)
            .bind(("id", record_key(&record.id, "resume")?))
            .bind(("upload_id", upload_id))
            .bind(("object_key", final_key))
            .await?;
        let _check: Option<serde_json::Value> = update_resp.take(0)?;

        println!("Successfully migrated resume record {}", resume_id_str);
        total_migrated += 1;
    }

    println!("\n--- Migration Summary ---");
    println!("Total migrated: {}", total_migrated);
    println!("Total skipped:  {}", total_skipped);
    println!("Total failed:   {}", failed_ids.len());

    if !failed_ids.is_empty() {
        eprintln!("\nFailed record IDs:");
        for id in &failed_ids {
            eprintln!("  - {}", id);
        }
        return Err(format!("Migration finished with {} failure(s). Review failed IDs above.", failed_ids.len()).into());
    }

    println!("All legacy resume records migrated successfully.");
    Ok(())
}
