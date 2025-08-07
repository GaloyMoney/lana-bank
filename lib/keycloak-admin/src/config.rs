use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeycloakConnectionConfig {
    #[serde(default = "default_url", alias = "keycloak_url")]
    pub url: String,
    #[serde(default = "default_client_id")]
    pub client_id: String,
    #[serde(default)]
    pub admin_username: String,
    #[serde(default)]
    pub admin_password: String,
}

fn default_url() -> String {
    "http://localhost:8081".to_string()
}

fn default_client_id() -> String {
    "admin-cli".to_string()
}

impl Default for KeycloakConnectionConfig {
    fn default() -> Self {
        Self {
            url: default_url(),
            client_id: default_client_id(),
            admin_username: "admin".to_string(),
            admin_password: "admin".to_string(),
        }
    }
}
