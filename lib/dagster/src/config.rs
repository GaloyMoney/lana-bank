use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DagsterConfig {
    #[serde(default = "default_uri")]
    pub uri: Url,
}

impl Default for DagsterConfig {
    fn default() -> Self {
        Self { uri: default_uri() }
    }
}

fn default_uri() -> Url {
    Url::parse("http://localhost:3000/graphql").expect("invalid url")
}
