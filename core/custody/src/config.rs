use serde::{Deserialize, Serialize};

use super::custodian::CustodyProviderConfig;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct CustodyConfig {
    #[serde(default)]
    pub custody_providers: CustodyProviderConfig,
}
