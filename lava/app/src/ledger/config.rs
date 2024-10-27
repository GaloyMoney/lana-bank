use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LedgerConfig {
    #[serde(default = "default_cala_url")]
    pub cala_url: String,
}

impl Default for LedgerConfig {
    fn default() -> Self {
        LedgerConfig {
            cala_url: default_cala_url(),
        }
    }
}

fn default_cala_url() -> String {
    "http://localhost:2252/graphql".to_string()
}
