use serde::{Deserialize, Serialize};

use super::CustodianEncryptionConfig;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CustodyConfig {
    #[serde(default)]
    pub custodian_encryption: CustodianEncryptionConfig,
}
