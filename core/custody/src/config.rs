use serde::{Deserialize, Serialize};

use encryption::{EncryptionConfig, EncryptionKey};

use super::custodian::CustodyProviderConfig;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct CustodyConfig {
    #[serde(skip)]
    pub encryption: EncryptionConfig,

    // FIXME: there is no way to pass for now
    #[serde(skip)]
    pub deprecated_encryption_key: Option<EncryptionKey>,

    #[serde(default)]
    pub custody_providers: CustodyProviderConfig,
}
