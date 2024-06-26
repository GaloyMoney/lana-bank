use axum::{extract::Json, response::IntoResponse, routing::post, Extension, Router};

use crate::app::LavaApp;

pub async fn sumsub_callback(
    Extension(app): Extension<LavaApp>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    // Log the received HTTP method and JSON payload
    println!("Received kyc callback with payload: {:?}", payload);

    match app.applicants().handle_callback(payload).await {
        Ok(()) => {
            println!("KYC callback handled successfully");
            return axum::Json("{}").into_response();
        }
        Err(err) => {
            eprintln!("Failed to handle KYC callback: {:?}", err);
            return axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    }
}

pub fn sumsub_routes() -> Router {
    Router::new().route("/sumsub/callback", post(sumsub_callback))
}
