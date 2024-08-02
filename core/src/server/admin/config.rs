use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AdminServerConfig {
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_jwks_url")]
    pub jwks_url: String,
    #[serde(default = "aud")]
    pub aud: String,
    #[serde(default = "default_kratos_admin_url")]
    pub kratos_admin_url: String,
}

impl Default for AdminServerConfig {
    fn default() -> Self {
        Self {
            port: default_port(),
            jwks_url: default_jwks_url(),
            aud: "https://admin-api/graphql".to_string(),
            kratos_admin_url: default_kratos_admin_url(),
        }
    }
}

fn default_port() -> u16 {
    5253
}

fn default_jwks_url() -> String {
    "http://localhost:4456/.well-known/jwks.json".to_string()
}

fn aud() -> String {
    "https://admin-api/graphql".to_string()
}

fn default_kratos_admin_url() -> String {
    "http://localhost:4433".to_string()
}
