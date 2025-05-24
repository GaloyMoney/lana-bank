mod config;
mod error;

pub use config::KratosAdminConfig;
pub use error::KratosAdminError;

use uuid::Uuid;

use ory_kratos_client::apis::{configuration::Configuration, identity_api};
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

    #[tracing::instrument(name = "kratos_admin.create_user", skip(self))]
    pub async fn create_user<T>(&self, email: String) -> Result<T, KratosAdminError>
    where
        T: From<Uuid>,
    {
        let identity = CreateIdentityBody {
            schema_id: "email".to_string(),
            traits: serde_json::json!({ "email": email }),
            credentials: None,
            metadata_admin: None,
            metadata_public: None,
            recovery_addresses: None,
            state: None,
            verifiable_addresses: None,
            organization_id: None,
        };

        let identity = identity_api::create_identity(&self.config, Some(identity)).await?;
        Ok(identity.id.parse::<Uuid>()?.into())
    }

    #[tracing::instrument(name = "kratos_admin.update_user_email", skip(self))]
    pub async fn update_user_email(
        &self,
        authentication_id: Uuid,
        email: String,
    ) -> Result<(), KratosAdminError> {
        let json_patch = vec![ory_kratos_client::models::JsonPatch {
            op: "replace".to_string(),
            path: "/traits/email".to_string(),
            value: Some(Some(serde_json::Value::String(email))),
            from: None,
        }];

        identity_api::patch_identity(
            &self.config,
            &authentication_id.to_string(),
            Some(json_patch),
        )
        .await?;

        Ok(())
    }
}
