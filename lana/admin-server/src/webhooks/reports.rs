use axum::{Extension, Router, http::StatusCode, routing::post};
use jwks_utils::JwtDecoderState;

use lana_app::app::LanaApp;

#[es_entity::es_event_context]
async fn sync_reports(Extension(app): Extension<LanaApp>) -> Result<StatusCode, StatusCode> {
    match app.reports().reports_sync().await {
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
