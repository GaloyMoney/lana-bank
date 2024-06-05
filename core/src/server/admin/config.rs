use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AdminServerConfig {
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_endpoint")]
    pub endpoint: String,
}

impl Default for AdminServerConfig {
    fn default() -> Self {
        Self {
            port: default_port(),
            endpoint: default_endpoint(),
        }
    }
}

fn default_port() -> u16 {
    5253
}

fn default_endpoint() -> String {
    "http://localhost".to_string()
}
