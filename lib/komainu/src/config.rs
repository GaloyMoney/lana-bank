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
