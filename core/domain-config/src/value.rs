use serde::{Deserialize, Serialize};

use crate::{DomainConfigError, Encrypted, EncryptionKey};

/// Represents a domain config value that can be either plaintext or encrypted.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DomainConfigValue {
    Plain { value: serde_json::Value },
    Encrypted(Encrypted),
}

impl DomainConfigValue {
    /// Returns the plaintext JSON value if Plain, or null for Encrypted.
    pub fn plain_or_null(&self) -> serde_json::Value {
        match self {
            Self::Plain { value } => value.clone(),
            Self::Encrypted(_) => serde_json::Value::Null,
        }
    }

    /// Create a new plaintext value.
    pub(crate) fn plain(value: serde_json::Value) -> Self {
        Self::Plain { value }
    }

    /// Create a new encrypted value from plaintext JSON.
    pub(crate) fn encrypted(key: &EncryptionKey, plaintext: &serde_json::Value) -> Self {
        Self::Encrypted(key.encrypt_json(plaintext))
    }

    /// Returns the plaintext JSON value if this is a Plain variant.
    pub(crate) fn as_plain(&self) -> Option<&serde_json::Value> {
        match self {
            Self::Plain { value } => Some(value),
            Self::Encrypted(_) => None,
        }
    }

    /// Returns true if this is an encrypted value.
    pub fn is_encrypted(&self) -> bool {
        matches!(self, Self::Encrypted(_))
    }

    /// Returns true if this value was encrypted with the given key.
    pub(crate) fn matches_key(&self, key: &EncryptionKey) -> bool {
        match self {
            Self::Encrypted(e) => e.matches_key(key),
            Self::Plain { .. } => false,
        }
    }

    /// Decrypt and return the plaintext JSON value.
    /// Returns an error for Plain variants (use as_plain instead).
    pub(crate) fn decrypt(
        &self,
        key: &EncryptionKey,
    ) -> Result<serde_json::Value, DomainConfigError> {
        match self {
            Self::Plain { .. } => Err(DomainConfigError::InvalidState(
                "Cannot decrypt a plaintext value".to_string(),
            )),
            Self::Encrypted(encrypted) => Ok(key.decrypt_json(encrypted)?),
        }
    }

    pub(crate) fn rotate(
        &self,
        new_key: &EncryptionKey,
        deprecated_key: &EncryptionKey,
    ) -> Result<Encrypted, DomainConfigError> {
        match self {
            Self::Plain { .. } => Err(DomainConfigError::InvalidState(
                "Cannot rotate a plaintext value".to_string(),
            )),
            Self::Encrypted(encrypted) => {
                let bytes = deprecated_key.decrypt(encrypted)?;
                Ok(new_key.encrypt(&bytes))
            }
        }
    }
}
