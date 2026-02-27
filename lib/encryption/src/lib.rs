#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod config;
mod error;

use chacha20poly1305::{
    ChaCha20Poly1305,
    aead::{Aead, AeadCore, KeyInit, OsRng},
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};

pub use config::EncryptionConfig;
pub use error::EncryptionError;

#[derive(Clone, Debug, Serialize, Default, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct EncryptionKeyId(String);

impl EncryptionKeyId {
    pub fn new(id: impl Into<String>) -> EncryptionKeyId {
        Self(id.into())
    }
}

#[derive(Clone, Debug)]
pub struct EncryptionKey {
    key: chacha20poly1305::Key,
    id: EncryptionKeyId,
}

impl EncryptionKey {
    pub fn new(key: [u8; 32]) -> Self {
        let hash = Sha256::digest(key);
        let id = EncryptionKeyId::new(hex::encode(&hash[..8]));
        Self {
            key: key.into(),
            id,
        }
    }

    pub fn encrypt(&self, plaintext: &[u8]) -> Encrypted {
        let cipher = ChaCha20Poly1305::new(&self.key);
        let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
        let ciphertext = cipher
            .encrypt(&nonce, plaintext)
            .expect("encryption should always succeed");

        Encrypted {
            ciphertext,
            nonce: nonce.to_vec(),
            key_id: self.id.clone(),
        }
    }

    pub fn decrypt(&self, encrypted: &Encrypted) -> Result<Vec<u8>, EncryptionError> {
        let cipher = ChaCha20Poly1305::new(&self.key);
        cipher
            .decrypt(
                encrypted.nonce.as_slice().into(),
                encrypted.ciphertext.as_slice(),
            )
            .map_err(|_| EncryptionError::Decryption)
    }

    pub fn encrypt_json(&self, value: &impl Serialize) -> Encrypted {
        let bytes = serde_json::to_vec(value).expect("JSON serialization should not fail");
        self.encrypt(&bytes)
    }

    pub fn decrypt_json<T: DeserializeOwned>(
        &self,
        encrypted: &Encrypted,
    ) -> Result<T, EncryptionError> {
        let bytes = self.decrypt(encrypted)?;
        Ok(serde_json::from_slice(&bytes)?)
    }
}

impl Default for EncryptionKey {
    fn default() -> Self {
        Self::new([0u8; 32])
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct Encrypted {
    ciphertext: Vec<u8>,
    nonce: Vec<u8>,
    key_id: EncryptionKeyId,
}

impl Encrypted {
    pub fn matches_key(&self, key: &EncryptionKey) -> bool {
        self.key_id == key.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_decrypt_json_roundtrip() {
        let key = EncryptionKey::new([1u8; 32]);
        let original = serde_json::json!({"enabled": true, "limit": 42});

        let encrypted = key.encrypt_json(&original);
        let decrypted: serde_json::Value = key.decrypt_json(&encrypted).unwrap();

        assert_eq!(original, decrypted);
        assert!(encrypted.matches_key(&key));
    }
}
