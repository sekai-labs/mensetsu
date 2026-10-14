use std::convert::Infallible;
use std::time::Duration;
use axum::{
    extract::{Path, State},
    response::sse::{Event, KeepAlive, Sse},
};
use tokio::time::sleep;
use tokio_stream::Stream;

use super::auth_check::verify_workspace_owner;
use super::AppState;
use crate::interfaces::http::extractors::authenticated_user::AuthenticatedUser;
use crate::shared::error::AppError;

pub async fn workspace_events(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(id): Path<String>,
) -> std::result::Result<Sse<impl Stream<Item = std::result::Result<Event, Infallible>>>, AppError> {
    let clean_id = id.replace("workspace:", "").replace('`', "");
    verify_workspace_owner(&state.db.client, &clean_id, &auth_user.user_id.0).await?;

    let db = state.db.client.clone();
    let stream = async_stream::stream! {
        let mut last_phase = String::new();
        let mut last_progress = 0u8;

        loop {
            let result = db
                .query("SELECT * FROM type::record('workspace', $id);")
                .bind(("id", clean_id.clone()))
                .await;

            if let Ok(mut res) = result {
                let item: Option<serde_json::Value> = res.take(0).unwrap_or(None);
                if let Some(w) = item {
                    let current_phase = w["current_phase"].as_str().unwrap_or("preparing_input").to_string();
                    let progress_percent = w["progress_percent"].as_u64().unwrap_or(0) as u8;

                    if current_phase != last_phase || progress_percent != last_progress {
                        last_phase = current_phase.clone();
                        last_progress = progress_percent;

                        let payload = serde_json::json!({
                            "workspace_id": clean_id,
                            "phase": current_phase,
                            "progress_percent": progress_percent,
                            "message": format!("Progress update: {current_phase} ({progress_percent}%)")
                        });

                        yield Ok(Event::default().event("progress").data(payload.to_string()));

                        if current_phase == "ready" || current_phase == "failed" {
                            break;
                        }
                    }
                }
            }

            sleep(Duration::from_millis(500)).await;
        }
    };

    Ok(Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(15))))
}
