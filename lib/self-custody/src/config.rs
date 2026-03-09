use std::fmt;

use bitcoin::{
    Network, NetworkKind,
    bip32::{ChildNumber, DerivationPath},
};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SelfCustodyNetwork {
    Testnet3,
    Testnet4,
    Signet,
    Mainnet,
}

impl SelfCustodyNetwork {
    pub(crate) fn bitcoin_network(self) -> Network {
        match self {
            Self::Mainnet => Network::Bitcoin,
            Self::Testnet3 | Self::Testnet4 => Network::Testnet,
            Self::Signet => Network::Signet,
        }
    }

    pub(crate) fn xpub_network_kind(self) -> NetworkKind {
        match self {
            Self::Mainnet => NetworkKind::Main,
            Self::Testnet3 | Self::Testnet4 | Self::Signet => NetworkKind::Test,
        }
    }

    pub(crate) fn bip84_account_path(self) -> DerivationPath {
        let coin_type = match self {
            Self::Mainnet => 0,
            Self::Testnet3 | Self::Testnet4 | Self::Signet => 1,
        };
        DerivationPath::from(vec![
            ChildNumber::from_hardened_idx(84).expect("constant index is valid"),
            ChildNumber::from_hardened_idx(coin_type).expect("constant index is valid"),
            ChildNumber::from_hardened_idx(0).expect("constant index is valid"),
        ])
    }
}

impl fmt::Display for SelfCustodyNetwork {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Testnet3 => f.write_str("testnet3"),
            Self::Testnet4 => f.write_str("testnet4"),
            Self::Signet => f.write_str("signet"),
            Self::Mainnet => f.write_str("mainnet"),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelfCustodyConfig {
    pub account_xpub: String,
    pub network: SelfCustodyNetwork,
}

#[derive(Clone, PartialEq, Eq)]
pub struct SelfCustodyClientConfig {
    pub account_xpub: String,
    pub network: SelfCustodyNetwork,
    pub esplora_url: Url,
}

impl fmt::Debug for SelfCustodyConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SelfCustodyConfig")
            .field("account_xpub", &"<redacted>")
            .field("network", &self.network)
            .finish()
    }
}

impl fmt::Debug for SelfCustodyClientConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SelfCustodyClientConfig")
            .field("account_xpub", &"<redacted>")
            .field("network", &self.network)
            .field("esplora_url", &self.esplora_url)
            .finish()
    }
}
