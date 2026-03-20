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

/// Result of creating an agent client in Keycloak.
pub struct AgentClientResult {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Clone)]
pub struct KeycloakClient {
    connection: KeycloakConnectionConfig,
}

impl KeycloakClient {
    pub fn new(connection: KeycloakConnectionConfig) -> Self {
        Self { connection }
    }

    fn get_client(&self) -> KeycloakAdmin<KeycloakServiceAccountAdminTokenRetriever> {
        let _ = rustls::crypto::ring::default_provider().install_default();
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
    #[instrument(name = "keycloak.disable_user", skip(self))]
    pub async fn disable_user(&self, lana_id: Uuid) -> Result<(), KeycloakClientError> {
        let user_id = self.get_keycloak_id_by_lana_id(lana_id).await?;
        let user = UserRepresentation {
            enabled: Some(false),
            ..Default::default()
        };
        let client = self.get_client();
        client
            .realm_users_with_user_id_put(&self.connection.realm, &user_id.to_string(), user)
            .await?;
        Ok(())
    }

    #[record_error_severity]
    #[instrument(name = "keycloak.enable_user", skip(self))]
    pub async fn enable_user(&self, lana_id: Uuid) -> Result<(), KeycloakClientError> {
        let user_id = self.get_keycloak_id_by_lana_id(lana_id).await?;
        let user = UserRepresentation {
            enabled: Some(true),
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

    /// Creates a confidential OAuth2 client for an agent with client_credentials grant.
    ///
    /// The client is configured with:
    /// - Service account enabled (client_credentials flow)
    /// - A custom `lanaId` attribute on the service account user
    /// - A protocol mapper that includes `lanaId` in JWT claims
    #[allow(deprecated)]
    #[record_error_severity]
    #[instrument(name = "keycloak.create_agent_client", skip(self))]
    pub async fn create_agent_client(
        &self,
        agent_name: String,
        lana_id: Uuid,
    ) -> Result<AgentClientResult, KeycloakClientError> {
        use std::collections::HashMap;

        let client_id = format!("agent-{}", lana_id);
        let client = self.get_client();

        // Create a confidential client with service account enabled
        let client_rep = keycloak::types::ClientRepresentation {
            client_id: Some(client_id.clone()),
            name: Some(format!("Agent: {}", agent_name)),
            enabled: Some(true),
            public_client: Some(false),
            service_accounts_enabled: Some(true),
            standard_flow_enabled: Some(false),
            direct_access_grants_enabled: Some(false),
            client_authenticator_type: Some("client-secret".to_string()),
            protocol: Some("openid-connect".to_string()),
            protocol_mappers: Some(vec![
                // Add lanaId as a JWT claim via user attribute mapper
                keycloak::types::ProtocolMapperRepresentation {
                    name: Some("lanaId".to_string()),
                    protocol: Some("openid-connect".to_string()),
                    protocol_mapper: Some("oidc-usermodel-attribute-mapper".to_string()),
                    consent_required: None,
                    consent_text: None,
                    config: Some({
                        let mut config = HashMap::new();
                        config.insert("user.attribute".to_string(), "lanaId".to_string());
                        config.insert("claim.name".to_string(), "lanaId".to_string());
                        config.insert("jsonType.label".to_string(), "String".to_string());
                        config.insert("id.token.claim".to_string(), "true".to_string());
                        config.insert("access.token.claim".to_string(), "true".to_string());
                        config.insert("userinfo.token.claim".to_string(), "true".to_string());
                        config
                    }),
                    id: None,
                },
                // Add lanaSubjectType claim so the server can distinguish agents from users
                keycloak::types::ProtocolMapperRepresentation {
                    name: Some("lanaSubjectType".to_string()),
                    protocol: Some("openid-connect".to_string()),
                    protocol_mapper: Some("oidc-hardcoded-claim-mapper".to_string()),
                    consent_required: None,
                    consent_text: None,
                    config: Some({
                        let mut config = HashMap::new();
                        config.insert("claim.name".to_string(), "lanaSubjectType".to_string());
                        config.insert("claim.value".to_string(), "agent".to_string());
                        config.insert("jsonType.label".to_string(), "String".to_string());
                        config.insert("id.token.claim".to_string(), "true".to_string());
                        config.insert("access.token.claim".to_string(), "true".to_string());
                        config.insert("userinfo.token.claim".to_string(), "true".to_string());
                        config
                    }),
                    id: None,
                },
            ]),
            ..Default::default()
        };

        client
            .realm_clients_post(&self.connection.realm, client_rep)
            .await?;

        // Find the created client to get its internal ID
        let clients = client
            .realm_clients_get(
                &self.connection.realm,
                Some(client_id.clone()),
                None,
                None,
                None,
                None,
                None,
            )
            .await?;

        let kc_client = clients.first().ok_or_else(|| {
            KeycloakClientError::ParseError("Created client not found".to_string())
        })?;

        let internal_id = kc_client
            .id
            .as_ref()
            .ok_or_else(|| KeycloakClientError::ParseError("Client ID not found".to_string()))?;

        // Get the client secret
        let credential = client
            .realm_clients_with_client_uuid_client_secret_get(&self.connection.realm, internal_id)
            .await?;

        let client_secret = credential.value.ok_or_else(|| {
            KeycloakClientError::ParseError("Client secret not found".to_string())
        })?;

        // Set lanaId attribute on the service account user
        let service_account_user = client
            .realm_clients_with_client_uuid_service_account_user_get(
                &self.connection.realm,
                internal_id,
            )
            .await?;

        let sa_user_id = service_account_user.id.as_ref().ok_or_else(|| {
            KeycloakClientError::ParseError("Service account user ID not found".to_string())
        })?;

        let mut attributes: HashMap<String, Vec<String>> =
            service_account_user.attributes.clone().unwrap_or_default();
        attributes.insert("lanaId".to_string(), vec![lana_id.to_string()]);
        attributes.insert("lanaSubjectType".to_string(), vec!["agent".to_string()]);

        let updated_user = keycloak::types::UserRepresentation {
            attributes: Some(attributes),
            ..Default::default()
        };

        client
            .realm_users_with_user_id_put(&self.connection.realm, sa_user_id, updated_user)
            .await?;

        tracing::info!(%lana_id, %client_id, "Agent client created in Keycloak");

        Ok(AgentClientResult {
            client_id,
            client_secret,
        })
    }

    /// Disables an agent's Keycloak client (used when deactivating an agent).
    #[record_error_severity]
    #[instrument(name = "keycloak.disable_agent_client", skip(self))]
    pub async fn disable_agent_client(&self, client_id: &str) -> Result<(), KeycloakClientError> {
        let client = self.get_client();

        let clients = client
            .realm_clients_get(
                &self.connection.realm,
                Some(client_id.to_string()),
                None,
                None,
                None,
                None,
                None,
            )
            .await?;

        let kc_client = clients.first().ok_or_else(|| {
            KeycloakClientError::ParseError(format!("Client {} not found", client_id))
        })?;

        let internal_id = kc_client
            .id
            .as_ref()
            .ok_or_else(|| KeycloakClientError::ParseError("Client ID not found".to_string()))?;

        let updated_client = keycloak::types::ClientRepresentation {
            enabled: Some(false),
            ..Default::default()
        };

        client
            .realm_clients_with_client_uuid_put(&self.connection.realm, internal_id, updated_client)
            .await?;

        tracing::info!(%client_id, "Agent client disabled in Keycloak");

        Ok(())
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
