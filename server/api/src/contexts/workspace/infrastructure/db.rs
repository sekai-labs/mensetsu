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

        let use_query = format!("USE NS {} DB {};", config.surreal_ns, config.surreal_db);
        let resp = client
            .query(use_query)
            .await
            .map_err(|e| AppError::Database(format!("USE NS DB query failed: {e}")))?;
        let _ = resp.check();

        
        let schema_sql = include_str!("../../../schema.surql");
        if let Ok(schema_resp) = client.query(schema_sql).await {
            let _ = schema_resp.check();
        }

        Ok(Self {
            client: Arc::new(client),
        })
    }
}
