use keycloak_client::{KeycloakConnectionConfig, Url};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CustomerSyncConfig {
    #[serde(default = "default_keycloak")]
    pub keycloak: KeycloakConnectionConfig,
}

impl Default for CustomerSyncConfig {
    fn default() -> Self {
        Self {
            keycloak: default_keycloak(),
        }
    }
}

fn default_keycloak() -> KeycloakConnectionConfig {
    KeycloakConnectionConfig {
        url: Url::parse("http://localhost:8081").expect("valid default URL"),
        client_id: "customer-service-account".to_string(),
        client_secret: "secret".to_string(),
        realm: "customer".to_string(),
    }
}
