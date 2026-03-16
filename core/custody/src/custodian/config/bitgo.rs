use serde::{Deserialize, Serialize};
use url::Url;

pub use bitgo::BitgoDirectoryConfig;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BitgoConfig {
    pub long_lived_token: String,
    pub enterprise_id: String,
    pub passphrase: String,
    pub testing_instance: bool,
    pub webhook_url: Url,
    pub webhook_secret: String,
}

impl From<BitgoConfig> for bitgo::BitgoConfig {
    fn from(config: BitgoConfig) -> Self {
        Self {
            long_lived_token: config.long_lived_token,
            enterprise_id: config.enterprise_id,
            passphrase: config.passphrase,
            bitgo_test: config.testing_instance,
            webhook_url: config.webhook_url,
            webhook_secret: config.webhook_secret.into_bytes(),
        }
    }
}

impl core::fmt::Debug for BitgoConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BitgoConfig")
            .field("long_lived_token", &"<redacted>")
            .field("enterprise_id", &self.enterprise_id)
            .field("passphrase", &"<redacted>")
            .field("testing_instance", &self.testing_instance)
            .field("webhook_secret", &"<redacted>")
            .finish()
    }
}
