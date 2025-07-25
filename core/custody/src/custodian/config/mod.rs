mod bitgo;
mod komainu;

use chacha20poly1305::{
    ChaCha20Poly1305,
    aead::{Aead, AeadCore, KeyInit, OsRng},
};
use serde::{Deserialize, Serialize};

pub use bitgo::{BitgoConfig, BitgoDirectoryConfig};
pub use komainu::{KomainuConfig, KomainuDirectoryConfig};

use super::error::CustodianError;

pub type EncryptionKey = chacha20poly1305::Key;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct ConfigCypher(pub(super) Vec<u8>);

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct Nonce(pub(super) Vec<u8>);

pub type EncryptedCustodianConfig = (ConfigCypher, Nonce);

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(into = "RawEncryptionConfig")]
#[serde(try_from = "RawEncryptionConfig")]
pub struct EncryptionConfig {
    pub key: EncryptionKey,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeprecatedEncryptionKey {
    pub nonce: String,
    pub key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

    pub(super) fn decrypt(
        key: &EncryptionKey,
        encrypted_config: &ConfigCypher,
        nonce: &Nonce,
    ) -> Self {
        let cipher = ChaCha20Poly1305::new(key);
        let decrypted_config = cipher
            .decrypt(
                chacha20poly1305::Nonce::from_slice(nonce.0.as_slice()),
                encrypted_config.0.as_slice(),
            )
            .expect("should always decrypt");
        let config: CustodianConfig = serde_json::from_slice(decrypted_config.as_slice())
            .expect("should be able to deserialize config");
        config
    }

    pub(super) fn rotate_encryption_key(
        encryption_key: &EncryptionKey,
        encrypted_config: &EncryptedCustodianConfig,
        deprecated_encryption_key: &DeprecatedEncryptionKey,
    ) -> EncryptedCustodianConfig {
        let cipher = ChaCha20Poly1305::new(encryption_key);
        let nonce_bytes =
            hex::decode(&deprecated_encryption_key.nonce).expect("should be able to decode nonce");
        let nonce = chacha20poly1305::Nonce::from_slice(nonce_bytes.as_slice());
        let deprecated_encrypted_key_bytes =
            hex::decode(&deprecated_encryption_key.key).expect("should be able to decode key");
        let deprecated_key_bytes = cipher
            .decrypt(nonce, deprecated_encrypted_key_bytes.as_slice())
            .expect("should be able to decrypt deprecated key");
        let deprecated_key = EncryptionKey::clone_from_slice(deprecated_key_bytes.as_ref());

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
        let key_bytes = key_vec.as_slice();
        Ok(Self {
            key: EncryptionKey::clone_from_slice(key_bytes),
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
    pub use super::*;

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
