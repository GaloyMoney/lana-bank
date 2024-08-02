use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KratosConfig {
    #[serde(default = "default_url")]
    pub url: String,
}

impl Default for KratosConfig {
    fn default() -> Self {
        Self { url: default_url() }
    }
}

fn default_url() -> String {
    "http://localhost:4434".to_string()
}
