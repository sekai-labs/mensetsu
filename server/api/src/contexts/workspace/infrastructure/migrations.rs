use crate::contexts::workspace::infrastructure::{db::Db, schema};
use crate::shared::error::{AppError, Result};
use serde_json::Value;

struct Migration {
    version: &'static str,
    sql: &'static str,
}

const MIGRATIONS: &[Migration] = &[
    Migration {
        version: "001_initial_schema",
        sql: include_str!("../../../../../../infra/migrations/001_initial_schema.surql"),
    },
    Migration {
        version: "002_user_roles_and_llm_settings",
        sql: include_str!(
            "../../../../../../infra/migrations/002_user_roles_and_llm_settings.surql"
        ),
    },
    Migration {
        version: "003_research_and_jd_intelligence",
        sql: include_str!(
            "../../../../../../infra/migrations/003_research_and_jd_intelligence.surql"
        ),
    },
    Migration {
        version: "004_workflow_ledger_and_runs",
        sql: include_str!("../../../../../../infra/migrations/004_workflow_ledger_and_runs.surql"),
    },
    Migration {
        version: "005_converge_with_schema_surql",
        sql: include_str!(
            "../../../../../../infra/migrations/005_converge_with_schema_surql.surql"
        ),
    },
    Migration {
        version: "006_store_workspace_run_inputs",
        sql: include_str!(
            "../../../../../../infra/migrations/006_store_workspace_run_inputs.surql"
        ),
    },
    Migration {
        version: "007_workflow_run_degraded",
        sql: include_str!("../../../../../../infra/migrations/007_workflow_run_degraded.surql"),
    },
    Migration {
        version: "008_llm_provider_routing_cutover",
        sql: include_str!(
            "../../../../../../infra/migrations/008_llm_provider_routing_cutover.surql"
        ),
    },
    Migration {
        version: "009_workflow_run_hybrid_type",
        sql: include_str!("../../../../../../infra/migrations/009_workflow_run_hybrid_type.surql"),
    },
    Migration {
        version: "010_resume_objects",
        sql: include_str!("../../../../../../infra/migrations/010_resume_objects.surql"),
    },
    Migration {
        version: "011_drop_resume_base64",
        sql: include_str!("../../../../../../infra/migrations/011_drop_resume_base64.surql"),
    },
    Migration {
        version: "012_token_usage_event_index",
        sql: include_str!("../../../../../../infra/migrations/012_token_usage_event_index.surql"),
    },
    Migration {
        version: "013_run_idempotency",
        sql: include_str!("../../../../../../infra/migrations/013_run_idempotency.surql"),
    },
    Migration {
        version: "014_cv_library",
        sql: include_str!("../../../../../../infra/migrations/014_cv_library.surql"),
    },
    Migration {
        version: "015_saved_jobs_application_flow",
        sql: include_str!(
            "../../../../../../infra/migrations/015_saved_jobs_application_flow.surql"
        ),
    },
    Migration {
        version: "016_application_dispatch_and_saved_job_index",
        sql: include_str!(
            "../../../../../../infra/migrations/016_application_dispatch_and_saved_job_index.surql"
        ),
    },
    Migration {
        version: "017_oauth_identity_providers",
        sql: include_str!("../../../../../../infra/migrations/017_oauth_identity_providers.surql"),
    },
];

pub async fn apply(client: &Db, ns: &str, db: &str) -> Result<()> {
    let mut info = client
        .query("INFO FOR DB;")
        .await
        .map_err(|e| AppError::Database(format!("Migration preflight failed: {e}")))?;
    let info: Option<Value> = info
        .take(0)
        .map_err(|e| AppError::Database(format!("Migration preflight result failed: {e}")))?;
    if info.is_none() {
        return Err(AppError::Database(
            "Migration preflight returned no database information".into(),
        ));
    }

    client
        .query("DEFINE TABLE IF NOT EXISTS schema_migration SCHEMAFULL; DEFINE FIELD IF NOT EXISTS version ON TABLE schema_migration TYPE string; DEFINE FIELD IF NOT EXISTS applied_at ON TABLE schema_migration TYPE datetime; DEFINE INDEX IF NOT EXISTS idx_schema_migration_version ON TABLE schema_migration COLUMNS version UNIQUE;")
        .await
        .map_err(|e| AppError::Database(format!("Could not initialize migration history: {e}")))?
        .check()
        .map_err(|e| AppError::Database(format!("Could not initialize migration history: {e}")))?;

    let mut result = client
        .query("SELECT VALUE version FROM schema_migration;")
        .await
        .map_err(|e| AppError::Database(format!("Could not read migration history: {e}")))?;
    let applied: Vec<String> = result
        .take(0)
        .map_err(|e| AppError::Database(format!("Could not decode migration history: {e}")))?;

    if applied.is_empty() {
        tracing::info!("Initializing current database schema and migration baseline");
        let legacy_schema = schema::SCHEMA
            .replace(
                "DEFINE NAMESPACE IF NOT EXISTS mensetsu;",
                &format!("DEFINE NAMESPACE IF NOT EXISTS {ns};"),
            )
            .replace("USE NS mensetsu;", &format!("USE NS {ns};"))
            .replace(
                "DEFINE DATABASE IF NOT EXISTS production;",
                &format!("DEFINE DATABASE IF NOT EXISTS {db};"),
            )
            .replace(
                "USE NS mensetsu DB production;",
                &format!("USE NS {ns} DB {db};"),
            );
        schema::apply(client, &legacy_schema, ns, db).await?;
        record_baseline(client).await?;
        return Ok(());
    }

    for migration in MIGRATIONS {
        if applied.iter().any(|version| version == migration.version) {
            continue;
        }
        if migration.version == "014_cv_library" {
            preflight_attached_resume_uploads(client).await?;
        }
        if migration.version == "016_application_dispatch_and_saved_job_index" {
            log_inconsistent_submitted_applications(client).await;
        }
        let sql = migration_sql(migration, ns, db)?;
        let query = format!(
            "BEGIN TRANSACTION;\n{sql}\nCREATE schema_migration:`{}` SET version = '{}', applied_at = time::now();\nCOMMIT TRANSACTION;",
            migration.version, migration.version
        );
        let response = client.query(query).await.map_err(|e| {
            AppError::Database(format!(
                "Migration {} could not be submitted: {e}",
                migration.version
            ))
        })?;
        response.check().map_err(|e| {
            AppError::Database(format!("Migration {} failed: {e}", migration.version))
        })?;
        tracing::info!(version = migration.version, "Applied database migration");
    }
    Ok(())
}
async fn preflight_attached_resume_uploads(client: &Db) -> Result<()> {
    let mut result = client
        .query("SELECT id FROM resume_upload WHERE status = 'attached' AND id NOT IN (SELECT VALUE source_upload_id FROM resume WHERE source_upload_id != NONE);")
        .await
        .map_err(|e| AppError::Database(format!("CV library migration preflight failed: {e}")))?;
    let rows: Vec<Value> = result.take(0).map_err(|e| {
        AppError::Database(format!("CV library migration preflight result failed: {e}"))
    })?;
    if let Some(row) = rows.first() {
        let id = row
            .get("id")
            .map(Value::to_string)
            .unwrap_or_else(|| "<unknown>".to_string());
        return Err(AppError::Database(format!(
            "CV library migration preflight found attached upload without resume.source_upload_id: {id}"
        )));
    }
    Ok(())
}

async fn log_inconsistent_submitted_applications(client: &Db) {
    let query = "SELECT id, workspace_id, user_id FROM job_application WHERE status = 'submitted' AND (workspace_id = NONE OR workspace_id NOT IN (SELECT VALUE id FROM workspace));\nSELECT id, workspace_id, user_id FROM job_application WHERE status = 'submitted' AND workspace_id != NONE AND workspace_id.user_id != user_id;";
    let mut response = match client.query(query).await {
        Ok(response) => response,
        Err(error) => {
            tracing::warn!(%error, "Could not inspect submitted applications for inconsistent workspace references");
            return;
        }
    };
    for statement in 0..2 {
        let rows: Vec<Value> = response.take(statement).unwrap_or_default();
        for row in rows {
            let application_id = row
                .get("id")
                .map(Value::to_string)
                .unwrap_or_else(|| "<unknown>".to_string());
            let workspace_id = row
                .get("workspace_id")
                .map(Value::to_string)
                .unwrap_or_else(|| "NONE".to_string());
            let user_id = row
                .get("user_id")
                .map(Value::to_string)
                .unwrap_or_else(|| "<unknown>".to_string());
            tracing::warn!(
                %application_id,
                %workspace_id,
                %user_id,
                "Submitted application has no owned workspace; leaving it for manual repair"
            );
        }
    }
}

pub async fn prepare_namespace_database(client: &Db, ns: &str, db: &str) -> Result<()> {
    validate_identifier(ns)?;
    validate_identifier(db)?;
    client
        .query(format!(
            "DEFINE NAMESPACE IF NOT EXISTS {ns}; USE NS {ns}; DEFINE DATABASE IF NOT EXISTS {db};"
        ))
        .await
        .map_err(|e| {
            AppError::Database(format!(
                "Could not prepare SurrealDB namespace/database: {e}"
            ))
        })?
        .check()
        .map_err(|e| {
            AppError::Database(format!(
                "Could not prepare SurrealDB namespace/database: {e}"
            ))
        })?;
    Ok(())
}

fn migration_sql(migration: &Migration, ns: &str, db: &str) -> Result<String> {
    validate_identifier(ns)?;
    validate_identifier(db)?;
    let mut sql = migration.sql.to_owned();
    sql = sql.replace("USE NS mensetsu DB production;", "");
    sql = sql.replace("USE NS mensetsu;", "");
    sql = sql.replace("DEFINE NAMESPACE IF NOT EXISTS mensetsu;", "");
    sql = sql.replace("DEFINE DATABASE IF NOT EXISTS production;", "");
    if sql.contains("USE ") {
        return Err(AppError::Database(format!(
            "Migration {} contains an unsupported database target",
            migration.version
        )));
    }
    Ok(sql)
}

fn validate_identifier(value: &str) -> Result<()> {
    if value.is_empty()
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    {
        return Err(AppError::Database(format!(
            "Invalid SurrealDB namespace/database identifier: {value}"
        )));
    }
    Ok(())
}

async fn record_baseline(client: &Db) -> Result<()> {
    let records = MIGRATIONS
        .iter()
        .map(|migration| {
            format!(
                "CREATE schema_migration:`{}` SET version = '{}', applied_at = time::now();",
                migration.version, migration.version
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    client
        .query(format!(
            "BEGIN TRANSACTION;\n{records}\nCOMMIT TRANSACTION;"
        ))
        .await
        .map_err(|e| AppError::Database(format!("Could not record existing schema baseline: {e}")))?
        .check()
        .map_err(|e| {
            AppError::Database(format!("Could not record existing schema baseline: {e}"))
        })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_identifier;

    #[test]
    fn database_identifiers_fail_closed() {
        assert!(validate_identifier("valid_123").is_ok());
        assert!(validate_identifier("invalid; DROP DATABASE production").is_err());
    }
}
