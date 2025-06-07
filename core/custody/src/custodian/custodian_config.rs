use super::{entity::KomainuConfig, error::CustodianError};

use chacha20poly1305::{
    ChaCha20Poly1305,
    aead::{Aead, AeadCore, KeyInit, OsRng},
};
use serde::{Deserialize, Serialize};

pub type EncryptionKey = chacha20poly1305::Key;
#[derive(Clone)]
pub struct ConfigCypher(pub(super) Vec<u8>);
#[derive(Clone)]
pub struct Nonce(pub(super) Vec<u8>);

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(into = "RawCustodianEncryptionConfig")]
#[serde(try_from = "RawCustodianEncryptionConfig")]
pub struct CustodianEncryptionConfig {
    pub key: EncryptionKey,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CustodianConfig {
    Komainu(KomainuConfig),
}

impl CustodianConfig {
    pub(super) fn encrypt(
        &self,
        key: &EncryptionKey,
    ) -> Result<(ConfigCypher, Nonce), CustodianError> {
        let cipher = ChaCha20Poly1305::new(key);
        let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
        let encrypted_config = cipher
            .encrypt(&nonce, serde_json::to_vec(self)?.as_slice())
            .expect("should always encrypt");

        Ok((ConfigCypher(encrypted_config), Nonce(nonce.to_vec())))
    }

    pub(super) fn decrypt(
        key: &EncryptionKey,
        encrypted_config: &ConfigCypher,
        nonce: &Nonce,
    ) -> Result<Self, CustodianError> {
        let cipher = ChaCha20Poly1305::new(key);
        let decrypted_config = cipher
            .decrypt(
                chacha20poly1305::Nonce::from_slice(nonce.0.as_slice()),
                encrypted_config.0.as_slice(),
            )
            .map_err(CustodianError::CouldNotDecryptCustodianConfig)?;
        let config: CustodianConfig = serde_json::from_slice(decrypted_config.as_slice())?;
        Ok(config)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
struct RawCustodianEncryptionConfig {
    pub key: String,
}
impl From<CustodianEncryptionConfig> for RawCustodianEncryptionConfig {
    fn from(config: CustodianEncryptionConfig) -> Self {
        Self {
            key: hex::encode(config.key),
        }
    }
}

impl TryFrom<RawCustodianEncryptionConfig> for CustodianEncryptionConfig {
    type Error = CustodianError;

    fn try_from(raw: RawCustodianEncryptionConfig) -> Result<Self, Self::Error> {
        let key_vec = hex::decode(raw.key)?;
        let key_bytes = key_vec.as_slice();
        Ok(Self {
            key: EncryptionKey::clone_from_slice(key_bytes),
        })
    }
}

impl std::fmt::Debug for CustodianEncryptionConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CustodianEncryptionConfig {{ key: *******Redacted******* }}"
        )
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
        });
        let key = gen_encryption_key();
        let (encrypted, nonce) = custodian_config.encrypt(&key).expect("Failed to encrypt");
        let decrypted =
            CustodianConfig::decrypt(&key, &encrypted, &nonce).expect("Failed to decrypt");

        assert_eq!(custodian_config, decrypted);
    }

    #[test]
    fn serialize_deserialize() {
        let key = gen_encryption_key();
        let custodian_encryption_config = CustodianEncryptionConfig { key };
        let serialized = serde_json::to_string(&custodian_encryption_config).unwrap();
        let deserialized: CustodianEncryptionConfig = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.key, key);
        assert_eq!(custodian_encryption_config, deserialized)
    }
}
