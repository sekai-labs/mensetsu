use std::sync::Arc;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;

use crate::shared::config::AppConfig;
use crate::shared::error::{AppError, Result};

pub type Db = Surreal<Client>;

#[derive(Clone)]
pub struct DatabasePool {
    pub client: Arc<Db>,
}

impl DatabasePool {
    pub async fn connect(config: &AppConfig) -> Result<Self> {
        let trimmed_url = config.surreal_url.trim_start_matches("ws://").trim_start_matches("http://");
        let client = Surreal::new::<Ws>(trimmed_url)
            .await
            .map_err(|e| AppError::Database(format!("Failed to connect to SurrealDB at {}: {e}", config.surreal_url)))?;

        client
            .signin(Root {
                username: config.surreal_user.clone(),
                password: config.surreal_pass.clone(),
            })
            .await
            .map_err(|e| AppError::Database(format!("SurrealDB auth failed: {e}")))?;
        client
            .use_ns(&config.surreal_ns)
            .use_db(&config.surreal_db)
            .await
            .map_err(|e| AppError::Database(format!("SurrealDB namespace select failed: {e}")))?;
        // Preflight check: refuse startup before schema::apply if legacy source_pdf_base64 exists and has non-empty values.
        Self::preflight_legacy_resume_base64(&client).await?;


        super::schema::apply(&client, super::schema::SCHEMA, &config.surreal_ns, &config.surreal_db).await?;

        Ok(Self {
            client: Arc::new(client),
        })
    }
    async fn preflight_legacy_resume_base64(client: &Db) -> Result<()> {
        // Query whether resume table exists and if any row has non-empty source_pdf_base64.
        // Check table presence first to distinguish absent table from database failures.
        let mut info_resp = client
            .query("INFO FOR DB;")
            .await
            .map_err(|e| AppError::Database(format!("Preflight schema check failed: {e}")))?;

        let db_info: Option<serde_json::Value> = info_resp
            .take(0)
            .map_err(|e| AppError::Database(format!("Preflight failed to read DB info: {e}")))?;
        let db_info = db_info.ok_or_else(|| AppError::Database("Preflight returned no database information".into()))?;
        let tables = db_info.get("tables").and_then(serde_json::Value::as_object)
            .ok_or_else(|| AppError::Database("Preflight returned no database table information".into()))?;
        let table_exists = tables.contains_key("resume");

        if !table_exists {
            // Fresh database without resume table, safe to proceed
            return Ok(());
        }

        let query = "SELECT id, string::len(source_pdf_base64) AS len FROM resume WHERE source_pdf_base64 != NONE AND source_pdf_base64 != NULL;";
        let mut resp = client
            .query(query)
            .await
            .map_err(|e| AppError::Database(format!("Preflight query for legacy resume base64 failed: {e}")))?;

        let rows: Vec<serde_json::Value> = resp
            .take(0)
            .map_err(|e| AppError::Database(format!("Preflight legacy resume row deserialization failed: {e}")))?;

        let mut unmigrated = Vec::new();
        for row in rows {
            let id = row.get("id").and_then(serde_json::Value::as_str)
                .ok_or_else(|| AppError::Database("Preflight resume row has no record ID".into()))?;
            match row.get("len").and_then(serde_json::Value::as_u64) {
                Some(0) => {}
                Some(_) => unmigrated.push(id.to_owned()),
                None => return Err(AppError::Database(format!(
                    "Preflight could not determine legacy PDF length for {id}"
                ))),
            }
        }
        if !unmigrated.is_empty() {
            let count = unmigrated.len();
            let sample = unmigrated.into_iter().take(10).collect::<Vec<_>>().join(", ");
            return Err(AppError::Database(format!(
                "Refusing startup: found {count} resume record(s) with unmigrated legacy source_pdf_base64 data (sample IDs: [{sample}]). Run `server/api/src/bin/migrate-resume-objects.rs` before starting the server."
            )));
        }

        Ok(())
    }
}
