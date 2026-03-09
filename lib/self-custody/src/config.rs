use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelfCustodyConfig {
    pub xpub: String,
    pub network: SelfCustodyNetwork,
    pub next_derivation_index: u32,
}

impl core::fmt::Debug for SelfCustodyConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SelfCustodyConfig")
            .field("xpub", &"<redacted>")
            .field("network", &self.network)
            .field("next_derivation_index", &self.next_derivation_index)
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SelfCustodyNetwork {
    Mainnet,
    Testnet,
    Signet,
}

impl std::fmt::Display for SelfCustodyNetwork {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Mainnet => write!(f, "mainnet"),
            Self::Testnet => write!(f, "testnet"),
            Self::Signet => write!(f, "signet"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfCustodyDirectoryConfig {
    #[serde(default = "default_esplora_url")]
    pub esplora_url: String,
}

impl Default for SelfCustodyDirectoryConfig {
    fn default() -> Self {
        Self {
            esplora_url: default_esplora_url(),
        }
    }
}

fn default_esplora_url() -> String {
    "https://blockstream.info/api".to_string()
}
