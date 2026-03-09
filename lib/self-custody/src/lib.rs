#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod config;
mod error;

pub use config::*;
pub use error::*;

use bitcoin::{
    CompressedPublicKey, NetworkKind,
    bip32::{ChildNumber, Xpub},
};

pub struct SelfCustodyClient {
    xpub: Xpub,
    network: SelfCustodyNetwork,
    esplora_url: String,
    http_client: reqwest::Client,
}

impl SelfCustodyClient {
    pub fn try_new(
        config: &SelfCustodyConfig,
        directory_config: &SelfCustodyDirectoryConfig,
    ) -> Result<Self, SelfCustodyError> {
        let xpub: Xpub = config
            .xpub
            .parse()
            .map_err(|e| SelfCustodyError::InvalidXpub(format!("{e}")))?;

        let expected_network = match config.network {
            SelfCustodyNetwork::Mainnet => NetworkKind::Main,
            SelfCustodyNetwork::Testnet | SelfCustodyNetwork::Signet => NetworkKind::Test,
        };

        if xpub.network != expected_network {
            return Err(SelfCustodyError::NetworkMismatch {
                expected: config.network.to_string(),
                actual: format!("{:?}", xpub.network),
            });
        }

        let esplora_url = directory_config
            .esplora_url
            .trim_end_matches('/')
            .to_string();

        Ok(Self {
            xpub,
            network: config.network,
            esplora_url,
            http_client: reqwest::Client::new(),
        })
    }

    pub fn derive_address(&self, index: u32) -> Result<String, SelfCustodyError> {
        // BIP84: derive external chain (0) then index
        let child_key = self
            .xpub
            .ckd_pub(
                &bitcoin::secp256k1::Secp256k1::new(),
                ChildNumber::Normal { index: 0 },
            )
            .and_then(|k| {
                k.ckd_pub(
                    &bitcoin::secp256k1::Secp256k1::new(),
                    ChildNumber::Normal { index },
                )
            })
            .map_err(|e| SelfCustodyError::DerivationError(format!("{e}")))?;

        let pubkey = CompressedPublicKey(child_key.public_key);

        let network = match self.network {
            SelfCustodyNetwork::Mainnet => bitcoin::Network::Bitcoin,
            SelfCustodyNetwork::Testnet => bitcoin::Network::Testnet,
            SelfCustodyNetwork::Signet => bitcoin::Network::Signet,
        };

        let address = bitcoin::Address::p2wpkh(&pubkey, network);
        Ok(address.to_string())
    }

    pub async fn get_address_balance(&self, address: &str) -> Result<u64, SelfCustodyError> {
        let url = format!("{}/address/{}", self.esplora_url, address);

        let resp = self
            .http_client
            .get(&url)
            .send()
            .await?
            .error_for_status()?;

        let body: serde_json::Value = resp.json().await?;

        let chain_stats = body
            .get("chain_stats")
            .ok_or_else(|| SelfCustodyError::EsploraResponse("missing chain_stats".to_string()))?;

        let funded: u64 = chain_stats
            .get("funded_txo_sum")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| {
                SelfCustodyError::EsploraResponse("missing funded_txo_sum".to_string())
            })?;

        let spent: u64 = chain_stats
            .get("spent_txo_sum")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| {
                SelfCustodyError::EsploraResponse("missing spent_txo_sum".to_string())
            })?;

        Ok(funded.saturating_sub(spent))
    }

    pub fn is_testnet(&self) -> bool {
        matches!(
            self.network,
            SelfCustodyNetwork::Testnet | SelfCustodyNetwork::Signet
        )
    }
}
