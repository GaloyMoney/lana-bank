#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod config;
mod error;
mod primitives;

use chacha20poly1305::{
    ChaCha20Poly1305,
    aead::{Aead, AeadCore, KeyInit, OsRng},
};
use serde::{Deserialize, Serialize};

pub use config::EncryptionConfig;
pub use error::EncryptionError;
pub use primitives::{EncryptionKey, KeyId};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct Encrypted {
    ciphertext: Vec<u8>,
    nonce: Vec<u8>,
    key_id: KeyId,
}

impl Encrypted {
    pub fn decrypt(&self, key: &EncryptionKey) -> Result<Vec<u8>, EncryptionError> {
        let cipher = ChaCha20Poly1305::new(&key.key);
        cipher
            .decrypt(self.nonce.as_slice().into(), self.ciphertext.as_slice())
            .map_err(|_| EncryptionError::Decryption)
    }

    pub fn encrypt(plaintext: &[u8], key: &EncryptionKey) -> Self {
        let cipher = ChaCha20Poly1305::new(&key.key);
        let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
        let ciphertext = cipher
            .encrypt(&nonce, plaintext)
            .expect("encryption should always succeed");

        Self {
            ciphertext,
            nonce: nonce.to_vec(),
            key_id: key.id.clone(),
        }
    }

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
        let bytes = serde_json::to_vec(&original).unwrap();

        let encrypted = Encrypted::encrypt(&bytes, &key);
        let decrypted_bytes = encrypted.decrypt(&key).unwrap();
        let decrypted: serde_json::Value = serde_json::from_slice(&decrypted_bytes).unwrap();

        assert_eq!(original, decrypted);
        assert!(encrypted.matches_key(&key));
    }
}
