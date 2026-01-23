use reqwest::Url;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct BitgoConfig {
    pub long_lived_token: String,
    pub enterprise_id: String,
    pub passphrase: String,
    pub bitgo_test: bool,
    pub webhook_url: Url,
    pub webhook_secret: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BitgoDirectoryConfig {
    #[serde(default = "default_testing_url")]
    pub testing_url: Url,
    #[serde(default = "default_production_url")]
    pub production_url: Url,
}

impl Default for BitgoDirectoryConfig {
    fn default() -> Self {
        Self {
            testing_url: default_testing_url(),
            production_url: default_production_url(),
        }
    }
}

fn default_testing_url() -> Url {
    "https://app.bitgo-test.com".parse().expect("valid URL")
}

fn default_production_url() -> Url {
    "https://app.bitgo.com/".parse().expect("valid URL")
}
