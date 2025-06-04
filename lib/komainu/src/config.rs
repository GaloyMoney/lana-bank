use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct KomainuConfig {
    pub api_user: String,
    pub api_secret: String,
    pub secret_key: KomainuSecretKey,
    pub proxy: Option<KomainuProxy>,
    pub komainu_test: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum KomainuSecretKey {
    Encrypted { dem: String, passphrase: String },
    Plain { dem: String },
}

#[derive(Clone, Serialize, Deserialize)]
pub enum KomainuProxy {
    Socks5(String),
}
