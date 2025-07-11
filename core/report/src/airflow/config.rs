use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AirflowConfig {
    #[serde(default = "default_uri")]
    pub uri: String,
}

impl Default for AirflowConfig {
    fn default() -> Self {
        Self { uri: default_uri() }
    }
}

fn default_uri() -> String {
    "http://localhost:8080".to_string()
}
