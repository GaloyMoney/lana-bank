use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

pub use self_custody::{SelfCustodyClientConfig, SelfCustodyConfig, SelfCustodyNetwork};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct SelfCustodyDirectoryConfig {
    #[serde(default)]
    pub mainnet_url: Option<Url>,
    #[serde(default)]
    pub testnet3_url: Option<Url>,
    #[serde(default)]
    pub testnet4_url: Option<Url>,
    #[serde(default)]
    pub signet_url: Option<Url>,
}

#[derive(Debug, Error)]
pub enum SelfCustodyDirectoryConfigError {
    #[error("SelfCustodyDirectoryConfigError - MissingEsploraUrlForNetwork: {network}")]
    MissingEsploraUrlForNetwork { network: SelfCustodyNetwork },
}

impl SelfCustodyDirectoryConfig {
    pub fn client_config(
        &self,
        config: SelfCustodyConfig,
    ) -> Result<SelfCustodyClientConfig, SelfCustodyDirectoryConfigError> {
        let esplora_url = self.esplora_url(config.network).cloned().ok_or(
            SelfCustodyDirectoryConfigError::MissingEsploraUrlForNetwork {
                network: config.network,
            },
        )?;

        Ok(SelfCustodyClientConfig {
            account_xpub: config.account_xpub,
            network: config.network,
            esplora_url,
        })
    }

    fn esplora_url(&self, network: SelfCustodyNetwork) -> Option<&Url> {
        match network {
            SelfCustodyNetwork::Mainnet => self.mainnet_url.as_ref(),
            SelfCustodyNetwork::Testnet3 => self.testnet3_url.as_ref(),
            SelfCustodyNetwork::Testnet4 => self.testnet4_url.as_ref(),
            SelfCustodyNetwork::Signet => self.signet_url.as_ref(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_network_specific_esplora_url() {
        let config = SelfCustodyDirectoryConfig {
            signet_url: Some("https://signet.example.com".parse().expect("valid url")),
            ..Default::default()
        };

        let client_config = config
            .client_config(SelfCustodyConfig {
                account_xpub: "xpub".to_string(),
                network: SelfCustodyNetwork::Signet,
            })
            .expect("url resolves");

        assert_eq!(
            client_config.esplora_url.as_str(),
            "https://signet.example.com/"
        );
    }

    #[test]
    fn missing_network_url_returns_error() {
        let error = SelfCustodyDirectoryConfig::default()
            .client_config(SelfCustodyConfig {
                account_xpub: "xpub".to_string(),
                network: SelfCustodyNetwork::Testnet4,
            })
            .expect_err("missing url should fail");

        match error {
            SelfCustodyDirectoryConfigError::MissingEsploraUrlForNetwork { network } => {
                assert_eq!(network, SelfCustodyNetwork::Testnet4);
            }
        }
    }
}
