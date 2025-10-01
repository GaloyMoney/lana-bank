#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct CreditConfig {
    #[serde(default = "default_customer_active_check_enabled")]
    pub customer_active_check_enabled: bool,
}

impl Default for CreditConfig {
    fn default() -> Self {
        CreditConfig {
            customer_active_check_enabled: default_customer_active_check_enabled(),
        }
    }
}

fn default_customer_active_check_enabled() -> bool {
    true
}
