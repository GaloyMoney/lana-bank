mod config;
mod error;

pub use config::KratosAdminConfig;
pub use error::KratosAdminError;

use core_user::UserId;
use ory_kratos_client::apis::{configuration::Configuration, identity_api::create_identity};
use ory_kratos_client::models::create_identity_body::CreateIdentityBody;

#[derive(Clone)]
pub struct KratosAdmin {
    pub config: Configuration,
}

impl KratosAdmin {
    pub fn init(config: KratosAdminConfig) -> Self {
        Self {
            config: Configuration {
                base_path: config.kratos_admin_url.clone(),
                ..Default::default()
            },
        }
    }

    pub async fn create_user(
        &self,
        user_id: UserId,
        email: String,
    ) -> Result<(), KratosAdminError> {
        let identity = CreateIdentityBody {
            schema_id: "email".to_string(),
            traits: serde_json::json!({
                "email": email,
                "user_id": user_id.to_string(),
            }),
            credentials: None,
            metadata_admin: None,
            metadata_public: None,
            recovery_addresses: None,
            state: None,
            verifiable_addresses: None,
        };

        create_identity(&self.config, Some(&identity)).await?;

        Ok(())
    }
}
