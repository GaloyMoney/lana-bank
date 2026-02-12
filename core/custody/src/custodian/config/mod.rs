mod bitgo;
mod komainu;

use chacha20poly1305::{
    ChaCha20Poly1305,
    aead::{Aead, AeadCore, KeyInit, OsRng},
};
use serde::{Deserialize, Serialize};
use tracing::instrument;
use tracing_macros::record_error_severity;

#[allow(unreachable_pub)] // Re-exported via custodian/mod.rs -> lib.rs
pub use bitgo::{BitgoConfig, BitgoDirectoryConfig};
#[allow(unreachable_pub)] // Re-exported via custodian/mod.rs -> lib.rs
pub use komainu::{KomainuConfig, KomainuDirectoryConfig};

use super::{
    client::{CustodianClient, error::CustodianClientError},
    error::CustodianError,
};

pub(crate) type EncryptionKey = chacha20poly1305::Key;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct ConfigCypher(pub(super) Vec<u8>);

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct Nonce(pub(super) Vec<u8>);

pub(crate) type EncryptedCustodianConfig = (ConfigCypher, Nonce);

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(into = "RawEncryptionConfig")]
#[serde(try_from = "RawEncryptionConfig")]
pub struct EncryptionConfig {
    pub key: EncryptionKey,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeprecatedEncryptionKey {
    pub nonce: String,
    pub key: String,
}

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

    pub(super) fn encrypt(&self, key: &EncryptionKey) -> EncryptedCustodianConfig {
        let cipher = ChaCha20Poly1305::new(key);
        let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
        let encrypted_config = cipher
            .encrypt(
                &nonce,
                serde_json::to_vec(self)
                    .expect("should always convert self to json")
                    .as_slice(),
            )
            .expect("should always encrypt");

        (ConfigCypher(encrypted_config), Nonce(nonce.to_vec()))
    }

    pub(super) fn try_decrypt(
        key: &EncryptionKey,
        encrypted_config: &ConfigCypher,
        nonce: &Nonce,
    ) -> Option<Self> {
        let cipher = ChaCha20Poly1305::new(key);
        let decrypted_config = cipher
            .decrypt(nonce.0.as_slice().into(), encrypted_config.0.as_slice())
            .ok()?;
        serde_json::from_slice(decrypted_config.as_slice()).ok()
    }

    pub(super) fn decrypt(
        key: &EncryptionKey,
        encrypted_config: &ConfigCypher,
        nonce: &Nonce,
    ) -> Self {
        Self::try_decrypt(key, encrypted_config, nonce)
            .expect("should always decrypt and deserialize config")
    }

    pub(super) fn rotate_encryption_key(
        encryption_key: &EncryptionKey,
        encrypted_config: &EncryptedCustodianConfig,
        deprecated_encryption_key: &DeprecatedEncryptionKey,
    ) -> EncryptedCustodianConfig {
        let cipher = ChaCha20Poly1305::new(encryption_key);
        let nonce_bytes =
            hex::decode(&deprecated_encryption_key.nonce).expect("should be able to decode nonce");
        let nonce = nonce_bytes.as_slice().into();
        let deprecated_encrypted_key_bytes =
            hex::decode(&deprecated_encryption_key.key).expect("should be able to decode key");
        let deprecated_key_bytes = cipher
            .decrypt(nonce, deprecated_encrypted_key_bytes.as_slice())
            .expect("should be able to decrypt deprecated key");
        let key_array: [u8; 32] = deprecated_key_bytes
            .as_slice()
            .try_into()
            .expect("key is 32 bytes");
        let deprecated_key: EncryptionKey = key_array.into();

        let new_config = Self::decrypt(&deprecated_key, &encrypted_config.0, &encrypted_config.1);

        new_config.encrypt(encryption_key)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
struct RawEncryptionConfig {
    pub key: String,
}
impl From<EncryptionConfig> for RawEncryptionConfig {
    fn from(config: EncryptionConfig) -> Self {
        Self {
            key: hex::encode(config.key),
        }
    }
}

impl TryFrom<RawEncryptionConfig> for EncryptionConfig {
    type Error = CustodianError;

    fn try_from(raw: RawEncryptionConfig) -> Result<Self, Self::Error> {
        let key_vec = hex::decode(raw.key)?;
        let key_array: [u8; 32] = key_vec
            .as_slice()
            .try_into()
            .map_err(|_| CustodianError::InvalidEncryptionKey)?;
        Ok(Self {
            key: key_array.into(),
        })
    }
}

impl std::fmt::Debug for EncryptionConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EncryptionConfig {{ key: *******Redacted******* }}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gen_encryption_key() -> EncryptionKey {
        ChaCha20Poly1305::generate_key(&mut OsRng)
    }

    #[test]
    fn encrypt_decrypt() {
        let custodian_config = CustodianConfig::Komainu(KomainuConfig {
            api_key: "api_key".to_string(),
            secret_key: "secret_key".to_string(),
            api_secret: "api_secret".to_string(),
            testing_instance: false,
            webhook_secret: "webhook".to_string(),
        });
        let key = gen_encryption_key();
        let (encrypted, nonce) = custodian_config.encrypt(&key);
        let decrypted = CustodianConfig::decrypt(&key, &encrypted, &nonce);
        assert_eq!(custodian_config, decrypted);
    }

    #[test]
    fn serialize_deserialize() {
        let key = gen_encryption_key();
        let encryption_config = EncryptionConfig { key };
        let serialized = serde_json::to_string(&encryption_config).unwrap();
        let deserialized: EncryptionConfig = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.key, key);
        assert_eq!(encryption_config, deserialized)
    }
}
