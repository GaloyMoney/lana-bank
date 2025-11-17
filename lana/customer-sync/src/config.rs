use keycloak_client::KeycloakConnectionConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CustomerSyncConfig {
    #[serde(default = "default_customer_status_sync_active")]
    pub customer_status_sync_active: bool,
    #[serde(default = "default_keycloak")]
    pub keycloak: KeycloakConnectionConfig,
}

impl Default for CustomerSyncConfig {
    fn default() -> Self {
        Self {
            customer_status_sync_active: default_customer_status_sync_active(),
            keycloak: default_keycloak(),
        }
    }
}

fn default_keycloak() -> KeycloakConnectionConfig {
    KeycloakConnectionConfig {
        url: "http://localhost:8081".to_string(),
        client_id: "customer-service-account".to_string(),
        client_secret: "secret".to_string(),
        realm: "customer".to_string(),
    }
}

fn default_customer_status_sync_active() -> bool {
    true
}
