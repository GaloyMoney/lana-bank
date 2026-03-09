use reqwest::Url;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct BitfinexConfig {
    pub api_key: String,
    pub api_secret: String,
    pub bitfinex_test: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BitfinexDirectoryConfig {
    #[serde(default = "default_production_url")]
    pub production_url: Url,
    #[serde(default = "default_testing_url")]
    pub testing_url: Url,
}

impl Default for BitfinexDirectoryConfig {
    fn default() -> Self {
        Self {
            production_url: default_production_url(),
            testing_url: default_testing_url(),
        }
    }
}

fn default_production_url() -> Url {
    "https://api.bitfinex.com".parse().expect("valid URL")
}

fn default_testing_url() -> Url {
    "https://api.bitfinex.com".parse().expect("valid URL")
}
