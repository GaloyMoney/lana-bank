use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Serialize, Deserialize)]
pub struct BfxAuthConfig {
    pub api_key: String,
    pub api_secret: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BfxDirectoryConfig {
    #[serde(default = "default_api_url")]
    pub api_url: Url,
}

impl Default for BfxDirectoryConfig {
    fn default() -> Self {
        Self {
            api_url: default_api_url(),
        }
    }
}

fn default_api_url() -> Url {
    "https://api.bitfinex.com/v2".parse().expect("valid URL")
}
