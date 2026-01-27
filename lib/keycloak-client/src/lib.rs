#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod config;
mod error;

pub use config::KeycloakConnectionConfig;
pub use error::KeycloakClientError;
pub use url::Url;

use keycloak::types::*;
use keycloak::{KeycloakAdmin, KeycloakServiceAccountAdminTokenRetriever};
use reqwest::Client;
use tracing::instrument;
use tracing_macros::record_error_severity;
use uuid::Uuid;

#[derive(Clone)]
pub struct KeycloakClient {
    connection: KeycloakConnectionConfig,
}

impl KeycloakClient {
    pub fn new(connection: KeycloakConnectionConfig) -> Self {
        Self { connection }
    }

    fn get_client(&self) -> KeycloakAdmin<KeycloakServiceAccountAdminTokenRetriever> {
        let http_client = Client::builder()
            .default_headers(tracing_utils::http::inject_trace_reqwest())
            .build()
            .expect("Failed to build HTTP client");

        let service_account_token_retriever =
            KeycloakServiceAccountAdminTokenRetriever::create_with_custom_realm(
                &self.connection.client_id,
                &self.connection.client_secret,
                &self.connection.realm,
                http_client.clone(),
            );

        // Workaround: Strip trailing slash from URL to avoid double slashes in API paths.
        //
        // The `url` crate normalizes "http://host:port" to "http://host:port/" (with trailing slash).
        // The `keycloak` crate builds paths via `format!("{}/admin/...", url)`, creating "http://host:port//admin/...".
        // Keycloak 26.4+ rejects paths with double slashes due to stricter URL normalization enforcement.
        //
        // See: https://github.com/keycloak/keycloak/issues/44269
        // See: https://github.com/keycloak/keycloak/issues/43763
        let url = self.connection.url.as_str().trim_end_matches('/');

        KeycloakAdmin::new(url, service_account_token_retriever, http_client)
    }

    /// Creates a user in Keycloak with the given email and lanaId attribute.
    ///
    /// This operation is idempotent: if a user with the given `lana_id` already exists,
    /// returns the existing user's Keycloak ID instead of creating a duplicate.
    #[record_error_severity]
    #[instrument(name = "keycloak.create_user", skip(self))]
    pub async fn create_user(
        &self,
        email: String,
        lana_id: Uuid,
    ) -> Result<Uuid, KeycloakClientError> {
        // Check if user already exists (idempotency check)
        let existing_users = self
            .query_users_by_attribute("lanaId", &lana_id.to_string())
            .await?;

        if let Some(user) = existing_users.first() {
            let user_id_str = user.id.as_ref().ok_or_else(|| {
                KeycloakClientError::ParseError(
                    "User ID not found in user representation".to_string(),
                )
            })?;
            let uuid = user_id_str.parse::<Uuid>()?;
            tracing::info!(%lana_id, %uuid, "User already exists in Keycloak, skipping creation");
            return Ok(uuid);
        }

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
        let client = self.get_client();
        let response = client
            .realm_users_post(&self.connection.realm, user)
            .await?;

        let user_id_str = response.to_id().ok_or_else(|| {
            KeycloakClientError::ParseError("User ID not found in response".to_string())
        })?;
        let uuid = user_id_str.parse::<Uuid>()?;
        Ok(uuid)
    }

    #[record_error_severity]
    #[instrument(name = "keycloak.update_user_email", skip(self))]
    pub async fn update_user_email(
        &self,
        lana_id: Uuid,
        email: String,
    ) -> Result<(), KeycloakClientError> {
        let user_id = self.get_keycloak_id_by_lana_id(lana_id).await?;
        let user = UserRepresentation {
            email: Some(email),
            email_verified: Some(true),
            ..Default::default()
        };
        let client = self.get_client();
        client
            .realm_users_with_user_id_put(&self.connection.realm, &user_id.to_string(), user)
            .await?;

        Ok(())
    }

    #[record_error_severity]
    #[instrument(name = "keycloak.query_users_by_attribute", skip(self))]
    async fn query_users_by_attribute(
        &self,
        attribute: &str,
        value: &str,
    ) -> Result<Vec<UserRepresentation>, KeycloakClientError> {
        let client = self.get_client();
        let users = client
            .realm_users_get(
                &self.connection.realm,                   // realm
                None, // brief_representation: return minimal fields if Some(true)
                None, // email: filter by email value
                None, // email_verified: filter by email verification status
                None, // enabled: filter by user enabled/disabled state
                None, // exact: applies ONLY to username/firstName/lastName/email; does NOT affect `q`
                None, // first: pagination offset
                None, // first_name: filter by first name
                None, // idp_alias: The alias of an Identity Provider linked to the user
                None, // idp_user_id: The userId at an Identity Provider linked to the user
                None, // last_name: filter by last name
                None, // max: pagination limit
                Some(format!("{}:{}", attribute, value)), // q: attribute query "key:value"
                None, // search: broad text over username/first/last/email
                None, // username: filter by username
            )
            .await?;
        Ok(users)
    }

    #[record_error_severity]
    #[instrument(name = "keycloak.get_keycloak_id_by_lana_id", skip(self))]
    pub async fn get_keycloak_id_by_lana_id(
        &self,
        lana_id: Uuid,
    ) -> Result<Uuid, KeycloakClientError> {
        let users = self
            .query_users_by_attribute("lanaId", &lana_id.to_string())
            .await?;
        match users.len() {
            0 => Err(KeycloakClientError::UserNotFound(format!(
                "No user found with lanaId: {}",
                lana_id
            ))),
            1 => {
                let user = &users[0];
                let user_id_str = user.id.as_ref().ok_or_else(|| {
                    KeycloakClientError::ParseError(
                        "User ID not found in user representation".to_string(),
                    )
                })?;
                let uuid = user_id_str.parse::<Uuid>()?;
                Ok(uuid)
            }
            _ => Err(KeycloakClientError::MultipleUsersFound(format!(
                "Multiple users found with lanaId: {} (found: {})",
                lana_id,
                users.len()
            ))),
        }
    }
}
