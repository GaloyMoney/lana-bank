use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PublicServerConfig {
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_jwks_url")]
    pub jwks_url: String,
    #[serde(default = "aud")]
    pub aud: String,
}

impl Default for PublicServerConfig {
    fn default() -> Self {
        Self {
            port: default_port(),
            jwks_url: default_jwks_url(),
            aud: "https://public-api/graphql".to_string(),
        }
    }
}

fn default_port() -> u16 {
    5252
}

fn default_jwks_url() -> String {
    "http://localhost:4456/.well-known/jwks.json".to_string()
}

fn aud() -> String {
    "https://public-api/graphql".to_string()
}
