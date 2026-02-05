use chacha20poly1305::{
    ChaCha20Poly1305,
    aead::{Aead, AeadCore, KeyInit, OsRng},
};
use serde::{Deserialize, Serialize};

use crate::DomainConfigError;

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

#[cfg(test)]
mod tests {
    use super::*;

    fn gen_encryption_key() -> EncryptionKey {
        ChaCha20Poly1305::generate_key(&mut OsRng)
    }

    #[test]
    fn encrypt_decrypt_bytes() {
        let key = gen_encryption_key();
        let original = b"hello world";

        let encrypted = EncryptedValue::encrypt(&key, original);
        let decrypted = encrypted.decrypt(&key).unwrap();

        assert_eq!(original.as_slice(), decrypted.as_slice());
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
