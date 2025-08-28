use axum::{Extension, Router, routing::post};
use axum::{extract::Json, response::IntoResponse};
use jwks_utils::JwtDecoderState;
use lana_app::app::LanaApp;

#[es_entity::es_event_context]
async fn callback(
    Extension(app): Extension<LanaApp>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    match app.applicants().handle_callback(payload).await {
        Ok(()) => axum::Json("{}").into_response(),
        Err(..) => axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub fn routes() -> Router<JwtDecoderState> {
    Router::new().route("/webhook/sumsub", post(callback))
}
