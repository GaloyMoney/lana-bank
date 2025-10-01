use serde::{Deserialize, Serialize};

use super::{DeprecatedEncryptionKey, EncryptionConfig, custodian::CustodyProviderConfig};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CustodyConfig {
    #[serde(skip)]
    pub encryption: EncryptionConfig,

    // FIXME: there is no way to pass for now
    #[serde(skip)]
    pub deprecated_encryption_key: Option<DeprecatedEncryptionKey>,

    #[serde(default)]
    pub custody_providers: CustodyProviderConfig,
}
