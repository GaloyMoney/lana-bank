mod bitgo;
mod komainu;

use serde::{Deserialize, Serialize};
use tracing::instrument;
use tracing_macros::record_error_severity;

pub use bitgo::{BitgoConfig, BitgoDirectoryConfig};
use encryption::{Encrypted, EncryptionKey};
pub use komainu::{KomainuConfig, KomainuDirectoryConfig};

use super::{
    client::{CustodianClient, error::CustodianClientError},
    error::CustodianError,
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct CustodyProviderConfig {
    #[serde(default)]
    pub komainu_directory: KomainuDirectoryConfig,
    #[serde(default)]
    pub bitgo_directory: BitgoDirectoryConfig,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CustodianConfig {
    Komainu(KomainuConfig),
    Bitgo(BitgoConfig),

    #[cfg(feature = "mock-custodian")]
    Mock,
}

impl CustodianConfig {
    #[record_error_severity]
    #[instrument(name = "custody.custodian_client", skip(self))]
    pub fn custodian_client(
        self,
        provider_config: &CustodyProviderConfig,
    ) -> Result<Box<dyn CustodianClient>, CustodianClientError> {
        match self {
            CustodianConfig::Komainu(config) => Ok(Box::new(
                ::komainu::KomainuClient::try_new(
                    config.into(),
                    provider_config.komainu_directory.clone(),
                )
                .map_err(CustodianClientError::client)?,
            )),
            CustodianConfig::Bitgo(config) => Ok(Box::new(
                ::bitgo::BitgoClient::try_new(
                    config.into(),
                    provider_config.bitgo_directory.clone(),
                )
                .map_err(CustodianClientError::client)?,
            )),

            #[cfg(feature = "mock-custodian")]
            CustodianConfig::Mock => Ok(Box::new(super::client::mock::CustodianMock)),
        }
    }

    pub(super) fn encrypt(&self, key: &EncryptionKey) -> Encrypted {
        let bytes = serde_json::to_vec(self).expect("should serialize");
        Encrypted::encrypt(&bytes, key)
    }

    pub(super) fn try_decrypt(key: &EncryptionKey, encrypted_config: &Encrypted) -> Option<Self> {
        Self::decrypt(key, encrypted_config).ok()
    }

    pub(super) fn decrypt(
        key: &EncryptionKey,
        encrypted_config: &Encrypted,
    ) -> Result<Self, CustodianError> {
        let bytes = encrypted_config.decrypt(key)?;
        Ok(serde_json::from_slice(&bytes)?)
    }

    pub(super) fn rotate_encryption_key(
        new_key: &EncryptionKey,
        deprecated_key: &EncryptionKey,
        encrypted_config: &Encrypted,
    ) -> Result<Encrypted, CustodianError> {
        let config = Self::decrypt(deprecated_key, encrypted_config)?;
        Ok(config.encrypt(new_key))
    }
}
