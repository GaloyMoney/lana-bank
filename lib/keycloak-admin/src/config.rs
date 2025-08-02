use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeycloakAdminConfig {
    #[serde(default = "default_keycloak_url")]
    pub keycloak_url: String,
    #[serde(default = "default_realm")]
    pub realm: String,
    #[serde(default = "default_client_id")]
    pub client_id: String,
    pub admin_username: String,
    pub admin_password: String,
}

impl Default for KeycloakAdminConfig {
    fn default() -> Self {
        Self {
            keycloak_url: default_keycloak_url(),
            realm: default_realm(),
            client_id: default_client_id(),
            admin_username: "admin".to_string(),
            admin_password: "admin".to_string(),
        }
    }
}

fn default_keycloak_url() -> String {
    "http://localhost:8081".to_string()
}

fn default_realm() -> String {
    "lana-admin".to_string()
}

fn default_client_id() -> String {
    "admin-cli".to_string()
}
