use chacha20poly1305::{
    ChaCha20Poly1305,
    aead::{Aead, AeadCore, KeyInit, OsRng},
};
use serde::{Deserialize, Serialize};

use crate::{DomainConfig, DomainConfigError, EncryptionConfig};

pub type EncryptionKey = chacha20poly1305::Key;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct EncryptedValue {
    ciphertext: Vec<u8>,
    nonce: Vec<u8>,
}

impl EncryptedValue {
    pub fn encrypt(key: &EncryptionKey, plaintext: &[u8]) -> Self {
        let cipher = ChaCha20Poly1305::new(key);
        let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
        let ciphertext = cipher
            .encrypt(&nonce, plaintext)
            .expect("encryption should always succeed");

        Self {
            ciphertext,
            nonce: nonce.to_vec(),
        }
    }

    pub fn decrypt(&self, key: &EncryptionKey) -> Result<Vec<u8>, DomainConfigError> {
        let cipher = ChaCha20Poly1305::new(key);
        cipher
            .decrypt(self.nonce.as_slice().into(), self.ciphertext.as_slice())
            .map_err(|_| DomainConfigError::Decryption)
    }
}

pub(crate) enum StorageEncryption {
    None,
    Encrypted(EncryptionKey),
}

impl StorageEncryption {
    pub fn from_config(entity: &DomainConfig, config: &EncryptionConfig) -> Self {
        if entity.encrypted {
            Self::Encrypted(config.key)
        } else {
            Self::None
        }
    }

    pub fn encrypt_for_storage(
        &self,
        value: serde_json::Value,
    ) -> Result<serde_json::Value, DomainConfigError> {
        match self {
            Self::Encrypted(key) => {
                let bytes = serde_json::to_vec(&value)?;
                let encrypted = EncryptedValue::encrypt(key, &bytes);
                Ok(serde_json::to_value(encrypted)?)
            }
            Self::None => Ok(value),
        }
    }

    pub fn decrypt_from_storage(
        &self,
        value: &serde_json::Value,
    ) -> Result<serde_json::Value, DomainConfigError> {
        match self {
            Self::Encrypted(key) if !value.is_null() => {
                let encrypted: EncryptedValue = serde_json::from_value(value.clone())?;
                let bytes = encrypted.decrypt(key)?;
                Ok(serde_json::from_slice(&bytes)?)
            }
            _ => Ok(value.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gen_encryption_key() -> EncryptionKey {
        ChaCha20Poly1305::generate_key(&mut OsRng)
    }

    #[test]
    fn encrypt_decrypt_json_roundtrip() {
        let key = gen_encryption_key();
        let original = serde_json::json!({"enabled": true, "limit": 42});
        let bytes = serde_json::to_vec(&original).unwrap();

        let encrypted = EncryptedValue::encrypt(&key, &bytes);
        let decrypted_bytes = encrypted.decrypt(&key).unwrap();
        let decrypted: serde_json::Value = serde_json::from_slice(&decrypted_bytes).unwrap();

        assert_eq!(original, decrypted);
    }
}
