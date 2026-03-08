use axum::{Extension, Router, body::Bytes, http::StatusCode, routing::post};
use jwks_utils::JwtDecoderState;
use serde::Deserialize;

use lana_app::app::LanaApp;

#[derive(Deserialize)]
struct SyncReportsWebhookPayload {
    dagster_run_id: String,
}

#[es_entity::es_event_context]
async fn sync_reports(
    Extension(app): Extension<LanaApp>,
    body: Bytes,
) -> Result<StatusCode, StatusCode> {
    let dagster_run_id = if body.is_empty() {
        None
    } else {
        let payload: SyncReportsWebhookPayload =
            serde_json::from_slice(&body).map_err(|_| StatusCode::BAD_REQUEST)?;
        Some(payload.dagster_run_id)
    };

    match app.reports().reports_sync(dagster_run_id).await {
        Ok(_job_id) => Ok(StatusCode::OK),
        Err(e) => {
            tracing::error!(error = %e, "Failed to sync reports");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub fn routes() -> Router<JwtDecoderState> {
    Router::new().route("/webhook/reports/sync", post(sync_reports))
}
