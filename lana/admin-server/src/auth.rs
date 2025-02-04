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

pub async fn user_id_from_authentication_id(
    Extension(app): Extension<LanaApp>,
    Json(mut payload): Json<AuthenticationPayload>,
) -> impl IntoResponse {
    let authentication_id = match uuid::Uuid::parse_str(&payload.subject) {
        Ok(authentication_id) => core_user::AuthenticationId::from(authentication_id),
        Err(error) => {
            println!("Error parsing authentication id: {:?}", error);
            return StatusCode::BAD_REQUEST.into_response();
        }
    };

    match app
        .users()
        .find_by_authentication_id(authentication_id)
        .await
    {
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
            println!("User not found: {:?}", authentication_id);
            StatusCode::NOT_FOUND.into_response()
        }
        Err(error) => {
            println!("Error finding user: {:?}", error);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub fn auth_routes() -> Router<JwtDecoderState> {
    Router::new().route(
        "/user/user-id-from-authentication-id",
        post(user_id_from_authentication_id),
    )
}
