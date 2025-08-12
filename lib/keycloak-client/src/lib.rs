#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod config;
mod error;

pub use config::KeycloakConnectionConfig;
pub use error::KeycloakClientError;

use keycloak::types::*;
use keycloak::{KeycloakAdmin, KeycloakServiceAccountAdminTokenRetriever};
use reqwest::Client;
use uuid::Uuid;

#[derive(Clone)]
pub struct KeycloakClient {
    connection: KeycloakConnectionConfig,
    http_client: Client,
}

impl KeycloakClient {
    pub fn new(connection: KeycloakConnectionConfig) -> Self {
        Self {
            connection,
            http_client: Client::new(),
        }
    }

    async fn get_client(
        &self,
    ) -> Result<KeycloakAdmin<KeycloakServiceAccountAdminTokenRetriever>, KeycloakClientError> {
        let service_account_token_retriever =
            KeycloakServiceAccountAdminTokenRetriever::create_with_custom_realm(
                &self.connection.client_id,
                &self.connection.client_secret,
                &self.connection.realm,
                self.http_client.clone(),
            );

        Ok(KeycloakAdmin::new(
            &self.connection.url,
            service_account_token_retriever,
            self.http_client.clone(),
        ))
    }

    pub async fn create_user(
        &self,
        email: String,
        lana_id: Uuid,
    ) -> Result<Uuid, KeycloakClientError> {
        use std::collections::HashMap;

        let mut attributes: HashMap<String, Vec<String>> = HashMap::new();
        attributes.insert("lanaId".to_string(), vec![lana_id.to_string()]);

        let user = UserRepresentation {
            email: Some(email),
            enabled: Some(true),
            email_verified: Some(true),
            attributes: Some(attributes),
            ..Default::default()
        };
        let client = self.get_client().await?;
        let response = client
            .realm_users_post(&self.connection.realm, user)
            .await?;
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
            .realm_users_with_user_id_put(&self.connection.realm, &user_id.to_string(), user)
            .await?;
        Ok(())
    }

    pub async fn get_user(&self, user_id: Uuid) -> Result<UserRepresentation, KeycloakClientError> {
        let client = self.get_client().await?;
        let user = client
            .realm_users_with_user_id_get(&self.connection.realm, &user_id.to_string(), None)
            .await?;
        Ok(user)
    }
}
