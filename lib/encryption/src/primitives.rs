use serde::{Deserialize, Serialize};

pub type EncryptionKey = chacha20poly1305::Key;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct KeyId(String);

impl KeyId {
    pub fn new(id: impl Into<String>) -> KeyId {
        Self(id.into())
    }
}

impl Default for KeyId {
    fn default() -> Self {
        Self(String::new())
    }
}
