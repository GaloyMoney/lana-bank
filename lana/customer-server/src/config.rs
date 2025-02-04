use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CustomerServerConfig {
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_jwks_url")]
    pub jwks_url: String,
    #[serde(default = "aud")]
    pub aud: String,
}

impl Default for CustomerServerConfig {
    fn default() -> Self {
        Self {
            port: default_port(),
            jwks_url: default_jwks_url(),
            aud: "https://customer-api/graphql".to_string(),
        }
    }
}

fn default_port() -> u16 {
    5254
}

fn default_jwks_url() -> String {
    "http://localhost:4456/.well-known/jwks.json".to_string()
}

fn aud() -> String {
    "https://admin-api/graphql".to_string()
}
