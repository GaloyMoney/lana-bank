#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod config;
mod error;

pub use config::KeycloakAdminConfig;
pub use error::KeycloakAdminError;

use keycloak::types::*;
use keycloak::{KeycloakAdmin as KeycloakClient, KeycloakAdminToken};
use reqwest::Client;
use uuid::Uuid;

#[derive(Clone)]
pub struct KeycloakAdmin {
    config: KeycloakAdminConfig,
    http_client: Client,
}

impl KeycloakAdmin {
    pub fn init(config: KeycloakAdminConfig) -> Result<Self, KeycloakAdminError> {
        Ok(Self {
            config,
            http_client: Client::new(),
        })
    }

    async fn get_client(&self) -> Result<KeycloakClient, KeycloakAdminError> {
        let admin_token = KeycloakAdminToken::acquire(
            &self.config.keycloak_url,
            &self.config.admin_username,
            &self.config.admin_password,
            &self.http_client,
        )
        .await
        .map_err(KeycloakAdminError::KeycloakError)?;
        Ok(KeycloakClient::new(
            &self.config.keycloak_url,
            admin_token,
            self.http_client.clone(),
        ))
    }

    pub async fn create_user(&self, email: String) -> Result<Uuid, KeycloakAdminError> {
        let user = UserRepresentation {
            email: Some(email),
            enabled: Some(true),
            email_verified: Some(true),
            ..Default::default()
        };
        let client = self.get_client().await?;
        let response = client
            .realm_users_post(&self.config.realm, user)
            .await
            .map_err(KeycloakAdminError::KeycloakError)?;
        let user_id_str = response.to_id().ok_or_else(|| {
            KeycloakAdminError::ParseError("User ID not found in response".to_string())
        })?;
        let uuid = user_id_str.parse::<Uuid>()?;
        Ok(uuid)
    }

    pub async fn update_user_email(
        &self,
        user_id: Uuid,
        email: String,
    ) -> Result<(), KeycloakAdminError> {
        let user = UserRepresentation {
            email: Some(email),
            email_verified: Some(true),
            ..Default::default()
        };
        let client = self.get_client().await?;
        client
            .realm_users_with_user_id_put(&self.config.realm, &user_id.to_string(), user)
            .await
            .map_err(KeycloakAdminError::KeycloakError)?;
        Ok(())
    }
}
