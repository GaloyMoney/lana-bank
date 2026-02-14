use axum::{
    Extension, Router,
    extract::{Path, Query},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use jwks_utils::JwtDecoderState;
use lana_app::app::LanaApp;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct SignedUrlParams {
    expires: u64,
    sig: String,
}

async fn serve_local_file(
    Extension(app): Extension<LanaApp>,
    Path(encoded_path): Path<String>,
    Query(params): Query<SignedUrlParams>,
) -> Response {
    // Decode the base64url-encoded path
    let path = match URL_SAFE_NO_PAD.decode(&encoded_path) {
        Ok(bytes) => match String::from_utf8(bytes) {
            Ok(s) => s,
            Err(_) => return (StatusCode::BAD_REQUEST, "Invalid path encoding").into_response(),
        },
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid base64 encoding").into_response(),
    };

    // Read the file with signature verification
    match app
        .storage()
        .read_local_file(&path, params.expires, &params.sig)
        .await
    {
        Ok(content) => {
            // Determine content type from file extension
            let content_type = path
                .rsplit('.')
                .next()
                .map(|ext| match ext.to_lowercase().as_str() {
                    "pdf" => "application/pdf",
                    "csv" => "text/csv",
                    "txt" => "text/plain",
                    "json" => "application/json",
                    "xml" => "application/xml",
                    _ => "application/octet-stream",
                })
                .unwrap_or("application/octet-stream");

            // Get filename from path
            let filename = path.rsplit('/').next().unwrap_or("download");

            (
                StatusCode::OK,
                [
                    (header::CONTENT_TYPE, content_type),
                    (
                        header::CONTENT_DISPOSITION,
                        &format!("inline; filename=\"{}\"", filename),
                    ),
                ],
                content,
            )
                .into_response()
        }
        Err(e) => {
            tracing::warn!(error = %e, path = %path, "Failed to serve local file");
            use lana_app::storage::client::error::StorageClientError;
            use lana_app::storage::error::StorageError;
            match e {
                StorageError::StorageClientError(client_err) => match client_err {
                    StorageClientError::SignatureExpired => {
                        (StatusCode::FORBIDDEN, "Link has expired").into_response()
                    }
                    StorageClientError::InvalidSignature => {
                        (StatusCode::FORBIDDEN, "Invalid signature").into_response()
                    }
                    StorageClientError::InvalidPath => {
                        (StatusCode::BAD_REQUEST, "Invalid path").into_response()
                    }
                    _ => (StatusCode::NOT_FOUND, "File not found").into_response(),
                },
                StorageError::LocalStorageNotConfigured => {
                    (StatusCode::NOT_FOUND, "Local storage not configured").into_response()
                }
            }
        }
    }
}

pub fn routes() -> Router<JwtDecoderState> {
    Router::new().route("/local-storage/{path}", get(serve_local_file))
}
