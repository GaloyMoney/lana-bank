use serde::{Deserialize, Serialize};

use crate::EncryptionKey;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct DomainEncryptionConfig {
    #[serde(skip)]
    pub key: EncryptionKey,
}
