use serde::{Deserialize, Serialize};

use super::{CustodianEncryptionConfig, DeprecatedEncryptionKey};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CustodyConfig {
    #[serde(default)]
    pub custodian_encryption: CustodianEncryptionConfig,

    #[serde(default)]
    pub deprecated_encryption_key: Option<DeprecatedEncryptionKey>,
}
