use axum::{
    extract::Json,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Extension, Router,
};
use serde::{Deserialize, Serialize};

use crate::{app::LavaApp, server::jwks::JwtDecoderState};

#[derive(Deserialize, std::fmt::Debug, Serialize)]
pub struct CustomerCallbackPayload {
    email: String,
    flow_id: String,
    flow_type: String,
    identity_id: String,
    schema_id: String,
    transient_payload: serde_json::Value,
}

pub async fn customer_callback(
    Extension(app): Extension<LavaApp>,
    Json(payload): Json<CustomerCallbackPayload>,
) -> impl IntoResponse {
    // Log the received HTTP method and JSON payload
    println!("Received auth callback with payload: {:?}", payload);

    let email = payload.email;
    let id = match payload.identity_id.parse() {
        Ok(id) => id,
        Err(error) => {
            println!("Error parsing identity_id: {:?}", error);
            return (StatusCode::BAD_REQUEST, "Invalid identity_id format").into_response();
        }
    };

    match app
        .customers()
        .create_customer_through_kratos(id, email)
        .await
    {
        Ok(user) => axum::Json(serde_json::json!( {
            "identity": { "id": user.id }
        }))
        .into_response(),
        Err(error) => {
            println!("Error creating user: {:?}", error);
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
        }
    }
}

#[derive(Deserialize, std::fmt::Debug, Serialize)]
pub struct UserCallbackPayload {
    email: String,
    transient_payload: serde_json::Value,
}

pub async fn user_callback(
    Extension(app): Extension<LavaApp>,
    Json(payload): Json<UserCallbackPayload>,
) -> Result<Response, StatusCode> {
    // Log the received HTTP method and JSON payload
    println!("Received user callback with payload: {:?}", payload);

    let email = payload.email;

    match app.users().find_by_email(&email).await {
        Ok(Some(_user)) => Ok(StatusCode::OK.into_response()),
        Ok(None) => {
            println!("User not found: {:?}", email);
            Ok(StatusCode::NOT_FOUND.into_response())
        }
        Err(error) => {
            println!("Error finding user: {:?}", error);
            Ok(StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct HydratorPayload {
    subject: String,
    extra: serde_json::Value,
    header: serde_json::Value,
    match_context: MatchContext,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MatchContext {
    regexp_capture_groups: serde_json::Value,
    url: serde_json::Value,
}

pub async fn user_id_from_email(
    Extension(app): Extension<LavaApp>,
    Json(mut payload): Json<HydratorPayload>,
) -> impl IntoResponse {
    let email = &payload.subject;
    match app.users().find_by_email(email).await {
        Ok(Some(user)) => {
            if let serde_json::Value::Object(ref mut extra) = payload.extra {
                extra.insert(
                    "subject".to_string(),
                    serde_json::Value::String(user.id.to_string()),
                );
            } else {
                payload.extra = serde_json::json!({
                    "subject": user.id.to_string()
                });
            }
            Json(payload).into_response()
        }
        Ok(None) => {
            println!("User not found: {:?}", email);
            StatusCode::NOT_FOUND.into_response()
        }
        Err(error) => {
            println!("Error finding user: {:?}", error);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub fn auth_routes() -> Router<JwtDecoderState> {
    Router::new()
        .route("/customer/callback", post(customer_callback))
        .route("/user/callback", post(user_callback))
        .route("/user/id-from-email", post(user_id_from_email))
}
