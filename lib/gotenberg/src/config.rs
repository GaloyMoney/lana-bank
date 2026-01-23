use reqwest::Url;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GotenbergConfig {
    #[serde(default = "default_gotenberg_url")]
    pub url: Url,
}

fn default_gotenberg_url() -> Url {
    "http://localhost:3030".parse().expect("valid URL")
}

impl Default for GotenbergConfig {
    fn default() -> Self {
        Self {
            url: default_gotenberg_url(),
        }
    }
}
