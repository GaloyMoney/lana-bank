mod config;
mod error;

pub use config::KratosAdminConfig;
pub use error::KratosAdminError;

use reqwest::Client;
use serde_json::{json, Value};
use uuid::Uuid;

#[derive(Clone)]
pub struct KratosAdmin {
    pub client: Client,
    pub base_url: String,
}

impl KratosAdmin {
    pub fn init(config: KratosAdminConfig) -> Self {
        Self {
            client: Client::new(),
            base_url: config.kratos_admin_url,
        }
    }

    #[tracing::instrument(name = "kratos_admin.create_user", skip(self))]
    pub async fn create_user<T>(&self, email: String) -> Result<T, KratosAdminError>
    where
        T: From<Uuid>,
    {
        let identity_body = json!({
            "schema_id": "email",
            "traits": {
                "email": email
            }
        });

        let response = self
            .client
            .post(&format!("{}/admin/identities", self.base_url))
            .json(&identity_body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(KratosAdminError::HttpError(format!(
                "Failed to create identity: {} - {}",
                response.status(),
                response.text().await.unwrap_or_default()
            )));
        }

        let identity: Value = response.json().await?;
        let id_str = identity["id"]
            .as_str()
            .ok_or_else(|| KratosAdminError::ParseError("Missing id field".to_string()))?;

        let uuid = id_str.parse::<Uuid>()?;
        Ok(uuid.into())
    }

    #[tracing::instrument(name = "kratos_admin.update_user_email", skip(self))]
    pub async fn update_user_email(
        &self,
        authentication_id: Uuid,
        email: String,
    ) -> Result<(), KratosAdminError> {
        let patch_body = json!([{
            "op": "replace",
            "path": "/traits/email",
            "value": email
        }]);

        let response = self
            .client
            .patch(&format!(
                "{}/admin/identities/{}",
                self.base_url, authentication_id
            ))
            .json(&patch_body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(KratosAdminError::HttpError(format!(
                "Failed to update identity: {} - {}",
                response.status(),
                response.text().await.unwrap_or_default()
            )));
        }

        Ok(())
    }
}
