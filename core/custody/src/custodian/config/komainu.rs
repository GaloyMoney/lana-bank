use serde::{Deserialize, Serialize};

#[allow(unreachable_pub)] // Re-exported via config/mod.rs -> custodian/mod.rs -> lib.rs
pub use komainu::KomainuDirectoryConfig;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KomainuConfig {
    pub api_key: String,
    pub api_secret: String,
    pub testing_instance: bool,
    pub secret_key: String,
    pub webhook_secret: String,
}

impl From<KomainuConfig> for komainu::KomainuConfig {
    fn from(config: KomainuConfig) -> Self {
        Self {
            api_user: config.api_key,
            api_secret: config.api_secret,
            secret_key: komainu::KomainuSecretKey::Plain {
                dem: config.secret_key,
            },
            komainu_test: config.testing_instance,
            webhook_secret: config.webhook_secret.into_bytes(),
        }
    }
}

impl core::fmt::Debug for KomainuConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KomainuConfig")
            .field("api_key", &self.api_key)
            .field("api_secret", &"<redacted>")
            .field("testing_instance", &self.testing_instance)
            .field("secret_key", &"<redacted>")
            .field("webhook_secret", &"<redacted>")
            .finish()
    }
}
