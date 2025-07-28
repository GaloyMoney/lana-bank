use axum::{
    Extension, Router,
    body::Bytes,
    extract::{DefaultBodyLimit, Path},
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
    payload: Bytes,
) {
    app.custody()
        .handle_webhook(provider, uri, headers, payload)
        .await
        .unwrap_or(())
}

pub fn webhook_routes() -> Router<JwtDecoderState> {
    Router::new()
        .route("/webhook/{provider}", post(handle_webhook))
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024))
}
