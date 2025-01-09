use axum::{
    extract::Json,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Extension, Router,
};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use jwks_utils::JwtDecoderState;
use lana_app::app::LanaApp;

#[derive(Deserialize, std::fmt::Debug, Serialize)]
pub struct UserCallbackPayload {
    email: String,
    transient_payload: serde_json::Value,
}

#[instrument(name = "admin.auth.user_callback", skip(app))]
pub async fn user_callback(
    Extension(app): Extension<LanaApp>,
    Json(payload): Json<UserCallbackPayload>,
) -> Result<Response, StatusCode> {
    let email = payload.email;

    match app.users().find_by_email(None, &email).await {
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
    Extension(app): Extension<LanaApp>,
    Json(mut payload): Json<HydratorPayload>,
) -> impl IntoResponse {
    let email = &payload.subject;
    match app.users().find_by_email(None, email).await {
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
        .route("/user/callback", post(user_callback))
        .route("/user/id-from-email", post(user_id_from_email))
}
