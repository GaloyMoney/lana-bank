use core_customer::AuthenticationId;

mod config;
mod error;

pub use config::KratosAdminConfig;
pub use error::KratosAdminError;

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

    pub async fn create_user(&self, email: String) -> Result<AuthenticationId, KratosAdminError> {
        let identity = CreateIdentityBody {
            schema_id: "email".to_string(),
            traits: serde_json::json!({ "email": email }),
            credentials: None,
            metadata_admin: None,
            metadata_public: None,
            recovery_addresses: None,
            state: None,
            verifiable_addresses: None,
        };

        let identity = create_identity(&self.config, Some(&identity)).await?;
        let kratos_id = uuid::Uuid::parse_str(&identity.id)?;

        let authentication_id = AuthenticationId::from(kratos_id);
        Ok(authentication_id)
    }
}
