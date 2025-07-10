use serde::{Deserialize, Serialize};

use super::{DeprecatedEncryptionKey, EncryptionConfig};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CustodyConfig {
    #[serde(default)]
    pub encryption: EncryptionConfig,

    #[serde(default)]
    pub deprecated_encryption_key: Option<DeprecatedEncryptionKey>,
}
