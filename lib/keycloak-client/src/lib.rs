#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod config;
mod error;

pub use config::KeycloakConnectionConfig;
pub use error::KeycloakClientError;

use keycloak::types::*;
use keycloak::{KeycloakAdmin, KeycloakAdminToken};
use reqwest::Client;
use uuid::Uuid;

#[derive(Clone)]
pub struct KeycloakClient {
    connection: KeycloakConnectionConfig,
    http_client: Client,
    realm: String,
}

impl KeycloakClient {
    pub fn new(connection: KeycloakConnectionConfig, realm: String) -> Self {
        Self {
            connection,
            http_client: Client::new(),
            realm,
        }
    }

    async fn get_client(&self) -> Result<KeycloakAdmin, KeycloakClientError> {
        let admin_token = KeycloakAdminToken::acquire(
            &self.connection.url,
            // should not use admin username and password, use client secret instead
            // but it seems KeycloakAdmin plugin doesn't export this at the top level
            // because maybe only at the "sub level" in generated code
            // https://github.com/kilork/keycloak/tree/master/src/rest/generated_rest
            &self.connection.admin_username,
            &self.connection.admin_password,
            &self.http_client,
        )
        .await?;
        Ok(KeycloakAdmin::new(
            &self.connection.url,
            admin_token,
            self.http_client.clone(),
        ))
    }

    pub async fn create_user(&self, email: String) -> Result<Uuid, KeycloakClientError> {
        let user = UserRepresentation {
            email: Some(email),
            enabled: Some(true),
            email_verified: Some(true),
            ..Default::default()
        };
        let client = self.get_client().await?;
        let response = client.realm_users_post(&self.realm, user).await?;
        let user_id_str = response.to_id().ok_or_else(|| {
            KeycloakClientError::ParseError("User ID not found in response".to_string())
        })?;
        let uuid = user_id_str.parse::<Uuid>()?;
        Ok(uuid)
    }

    pub async fn update_user_email(
        &self,
        user_id: Uuid,
        email: String,
    ) -> Result<(), KeycloakClientError> {
        let user = UserRepresentation {
            email: Some(email),
            email_verified: Some(true),
            ..Default::default()
        };
        let client = self.get_client().await?;
        client
            .realm_users_with_user_id_put(&self.realm, &user_id.to_string(), user)
            .await?;
        Ok(())
    }

    pub async fn get_user(&self, user_id: Uuid) -> Result<UserRepresentation, KeycloakClientError> {
        let client = self.get_client().await?;
        let user = client
            .realm_users_with_user_id_get(&self.realm, &user_id.to_string(), None)
            .await?;
        Ok(user)
    }
}
