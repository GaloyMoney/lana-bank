use serde::{Deserialize, Serialize};

pub use bitfinex::BitfinexDirectoryConfig;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BitfinexConfig {
    pub api_key: String,
    pub api_secret: String,
    pub testing_instance: bool,
}

impl From<BitfinexConfig> for bitfinex::BitfinexConfig {
    fn from(config: BitfinexConfig) -> Self {
        Self {
            api_key: config.api_key,
            api_secret: config.api_secret,
            bitfinex_test: config.testing_instance,
        }
    }
}

impl core::fmt::Debug for BitfinexConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BitfinexConfig")
            .field("api_key", &"<redacted>")
            .field("api_secret", &"<redacted>")
            .field("testing_instance", &self.testing_instance)
            .finish()
    }
}
