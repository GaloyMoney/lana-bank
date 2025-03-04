use axum::{
    extract::Json, http::StatusCode, response::IntoResponse, routing::post, Extension, Router,
};
use serde::{Deserialize, Serialize};

use jwks_utils::JwtDecoderState;
use lana_app::app::LanaApp;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AuthenticationPayload {
    subject: String,
    extra: serde_json::Value,
}

pub async fn customer_id_from_authentication_id(
    Extension(app): Extension<LanaApp>,
    Json(mut payload): Json<AuthenticationPayload>,
) -> impl IntoResponse {
    let authentication_id = match payload.subject.parse::<core_customer::AuthenticationId>() {
        Ok(id) => id,
        Err(e) => {
            println!("Error parsing authentication id: {:?}", e);
            return StatusCode::BAD_REQUEST.into_response();
        }
    };

    match app
        .customers()
        .find_by_authentication_id(authentication_id)
        .await
    {
        Ok(customer) => {
            if let serde_json::Value::Object(ref mut extra) = payload.extra {
                extra.insert(
                    "subject".to_string(),
                    serde_json::Value::String(customer.id.to_string()),
                );
            } else {
                payload.extra = serde_json::json!({
                    "subject": customer.id.to_string()
                });
            }
            Json(payload).into_response()
        }
        Err(e) if e.was_not_found() => {
            println!("Customer not found: {:?}", authentication_id);
            StatusCode::NOT_FOUND.into_response()
        }
        Err(error) => {
            println!("Error finding customer: {:?}", error);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Verify2faPayload {
    extra: ExtraPayload,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ExtraPayload {
    authenticator_assurance_level: String,
}

pub async fn verify_2fa(Json(payload): Json<Verify2faPayload>) -> impl IntoResponse {
    println!(
        "2FA verification AAL: {}",
        payload.extra.authenticator_assurance_level
    );
    StatusCode::OK.into_response()
}

pub fn auth_routes() -> Router<JwtDecoderState> {
    Router::new()
        .route(
            "/customer/customer-id-from-authentication-id",
            post(customer_id_from_authentication_id),
        )
        .route("/customer/verify-2fa", post(verify_2fa))
}
