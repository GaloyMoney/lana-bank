use axum::{
    Extension, Router,
    extract::{Json, Path},
    http::{HeaderMap, Uri},
    routing::post,
};
use jwks_utils::JwtDecoderState;

use lana_app::app::LanaApp;

async fn handle_webhook(
    Extension(app): Extension<LanaApp>,
    Path(provider): Path<String>,
    headers: HeaderMap,
    uri: Uri,
    Json(payload): Json<serde_json::Value>,
) {
    app.custody()
        .handle_webhook(provider, &uri, &headers, payload)
        .await
        .unwrap_or(())
}

pub fn webhook_routes() -> Router<JwtDecoderState> {
    Router::new().route("/{provider}/webhook", post(handle_webhook))
}
