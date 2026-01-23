use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CustomerServerConfig {
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_jwks_url")]
    pub jwks_url: Url,
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

fn default_jwks_url() -> Url {
    Url::parse("http://localhost:4456/.well-known/jwks.json").expect("valid default URL")
}

fn aud() -> String {
    "https://customer-api/graphql".to_string()
}
