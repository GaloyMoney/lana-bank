use reqwest::Url;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct KomainuConfig {
    pub api_user: String,
    pub api_secret: String,
    pub secret_key: KomainuSecretKey,
    pub komainu_test: bool,
    pub webhook_secret: Vec<u8>,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum KomainuSecretKey {
    Encrypted { dem: String, passphrase: String },
    Plain { dem: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KomainuDirectoryConfig {
    #[serde(default = "default_production_url")]
    pub production_url: Url,
    #[serde(default = "default_testing_url")]
    pub testing_url: Url,
}

impl Default for KomainuDirectoryConfig {
    fn default() -> Self {
        Self {
            production_url: default_production_url(),
            testing_url: default_testing_url(),
        }
    }
}

fn default_production_url() -> Url {
    "https://api.komainu.io".parse().expect("valid URL")
}

fn default_testing_url() -> Url {
    "https://api-demo.komainu.io".parse().expect("valid URL")
}
