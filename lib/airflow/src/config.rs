use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AirflowConfig {
    #[serde(default = "default_uri")]
    pub uri: Url,
}

impl Default for AirflowConfig {
    fn default() -> Self {
        Self { uri: default_uri() }
    }
}

fn default_uri() -> Url {
    Url::parse("http://localhost:8080").expect("invalid url")
}
