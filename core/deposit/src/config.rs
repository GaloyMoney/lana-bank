#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct DepositConfig {
    #[serde(default = "default_require_verified_customer_for_account")]
    pub require_verified_customer_for_account: bool,
}

impl Default for DepositConfig {
    fn default() -> Self {
        Self {
            require_verified_customer_for_account: default_require_verified_customer_for_account(),
        }
    }
}

fn default_require_verified_customer_for_account() -> bool {
    true
}
