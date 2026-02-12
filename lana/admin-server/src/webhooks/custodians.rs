use axum::{
    Extension, Router,
    body::Bytes,
    extract::{DefaultBodyLimit, Path},
    http::{HeaderMap, Uri},
    routing::post,
};
use jwks_utils::JwtDecoderState;

use lana_app::app::LanaApp;

#[es_entity::es_event_context]
async fn webhook(
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

pub(crate) fn routes() -> Router<JwtDecoderState> {
    Router::new()
        .route("/webhook/custodian/{provider}", post(webhook))
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024))
}
