use axum::{Extension, Router, routing::get};
use axum::response::IntoResponse;
use jwks_utils::JwtDecoderState;
use lana_app::app::LanaApp;

#[es_entity::es_event_context]
async fn callback(Extension(app): Extension<LanaApp>) -> impl IntoResponse {
    match app.reports().handle_new_reports_callback().await {
        Ok(()) => axum::Json("{}").into_response(),
        Err(..) => axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub fn routes() -> Router<JwtDecoderState> {
    Router::new().route("/webhook/new-reports", get(callback))
}
