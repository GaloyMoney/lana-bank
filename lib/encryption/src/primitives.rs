use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, Serialize, Default, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct KeyId(String);

impl KeyId {
    pub fn new(id: impl Into<String>) -> KeyId {
        Self(id.into())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct EncryptionKey {
    #[serde(skip)]
    pub(crate) key: chacha20poly1305::Key,
    #[serde(skip)]
    pub(crate) id: KeyId,
}

impl EncryptionKey {
    pub fn new(key: [u8; 32]) -> Self {
        let hash = Sha256::digest(key);
        let id = KeyId::new(hex::encode(&hash[..8]));
        Self {
            key: key.into(),
            id,
        }
    }
}

impl Default for EncryptionKey {
    fn default() -> Self {
        Self::new([0u8; 32])
    }
}
