use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KeycloakConnectionConfig {
    #[serde(default = "default_url")]
    pub url: Url,
    pub client_id: String,
    pub realm: String,
    #[serde(skip)]
    pub client_secret: String,
}

fn default_url() -> Url {
    Url::parse("http://localhost:8081").expect("valid default URL")
}

impl Default for KeycloakConnectionConfig {
    fn default() -> Self {
        Self {
            url: default_url(),
            client_id: "internal-service-account".to_string(),
            client_secret: "secret".to_string(),
            realm: "internal".to_string(),
        }
    }
}
