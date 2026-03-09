use serde::{Deserialize, Serialize};

pub use bfx_client::BfxDirectoryConfig as BitfinexDirectoryConfig;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BitfinexConfig {
    pub api_key: String,
    pub api_secret: String,
    pub wallet: String,
}

impl From<BitfinexConfig> for bfx_client::BfxAuthConfig {
    fn from(config: BitfinexConfig) -> Self {
        Self {
            api_key: config.api_key,
            api_secret: config.api_secret,
        }
    }
}

impl core::fmt::Debug for BitfinexConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BitfinexConfig")
            .field("api_key", &self.api_key)
            .field("api_secret", &"<redacted>")
            .field("wallet", &self.wallet)
            .finish()
    }
}
