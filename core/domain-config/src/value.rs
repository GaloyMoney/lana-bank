use serde::{Deserialize, Serialize};

use crate::{DomainConfigError, Encrypted, EncryptionKey};

/// Represents a domain config value that can be either plaintext or encrypted.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DomainConfigValue {
    Plain { value: serde_json::Value },
    Encrypted(Encrypted),
    Rotated,
}

impl DomainConfigValue {
    /// Returns the plaintext JSON value if Plain, or null for Encrypted.
    pub fn plain_or_null(&self) -> serde_json::Value {
        match self {
            Self::Plain { value } => value.clone(),
            Self::Encrypted(_) => serde_json::Value::Null,
            Self::Rotated => serde_json::Value::Null,
        }
    }

    /// Create a new plaintext value.
    pub(crate) fn plain(value: serde_json::Value) -> Self {
        Self::Plain { value }
    }

    /// Create a new encrypted value from plaintext JSON.
    pub(crate) fn encrypted(key: &EncryptionKey, plaintext: &serde_json::Value) -> Self {
        let bytes = serde_json::to_vec(plaintext).expect("JSON serialization should not fail");
        Self::Encrypted(Encrypted::encrypt(&bytes, key))
    }

    /// Returns the plaintext JSON value if this is a Plain variant.
    pub(crate) fn as_plain(&self) -> Option<&serde_json::Value> {
        match self {
            Self::Plain { value } => Some(value),
            Self::Encrypted(_) | Self::Rotated => None,
        }
    }

    /// Returns true if this is an encrypted value.
    pub fn is_encrypted(&self) -> bool {
        matches!(self, Self::Encrypted(_))
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
            Self::Rotated => Err(DomainConfigError::InvalidState(
                "Cannot decrypt a rotated value".to_string(),
            )),
            Self::Encrypted(encrypted) => {
                let bytes = encrypted.decrypt(key)?;
                Ok(serde_json::from_slice(&bytes)?)
            }
        }
    }

    pub(crate) fn rotate(
        &self,
        new_key: &EncryptionKey,
        deprecated_key: &EncryptionKey,
    ) -> Result<Self, DomainConfigError> {
        let plaintext = self.decrypt(deprecated_key)?;
        Ok(Self::encrypted(new_key, &plaintext))
    }
}
