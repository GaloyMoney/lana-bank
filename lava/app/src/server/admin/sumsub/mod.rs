use axum::{extract::Json, response::IntoResponse, routing::post, Extension, Router};

use crate::{app::LavaApp, server::jwks::JwtDecoderState};

pub async fn sumsub_callback(
    Extension(app): Extension<LavaApp>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    // Log the received HTTP method and JSON payload
    println!("Received kyc callback with payload: {:?}", payload);

    match app.applicants().handle_callback(payload).await {
        Ok(()) => {
            println!("KYC callback handled successfully");
            axum::Json("{}").into_response()
        }
        Err(err) => {
            eprintln!("Failed to handle KYC callback: {:?}", err);
            axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub fn sumsub_routes() -> Router<JwtDecoderState> {
    Router::new().route("/sumsub/callback", post(sumsub_callback))
}
