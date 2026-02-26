use serde::{Deserialize, Serialize};

use crate::EncryptionKey;

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct EncryptionConfig {
    #[serde(skip)]
    pub encryption_key: EncryptionKey,

    #[serde(skip)]
    pub deprecated_encryption_key: Option<EncryptionKey>,
}

impl std::fmt::Debug for EncryptionConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EncryptionConfig {{ key: *******Redacted******* }}")
            .finish()
    }
}
