use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeycloakConnectionConfig {
    #[serde(default = "default_url")]
    pub url: String,
    pub client_id: String,
    pub realm: String,
    #[serde(skip)]
    pub client_secret: String,
}

fn default_url() -> String {
    "http://localhost:8081".to_string()
}

impl Default for KeycloakConnectionConfig {
    fn default() -> Self {
        Self {
            url: default_url(),
            client_id: "internal-service-account".to_string(),
            client_secret: "".to_string(),
            realm: "internal".to_string(),
        }
    }
}
