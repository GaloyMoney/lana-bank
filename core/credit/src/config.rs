#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[serde_with::serde_as]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct CreditConfig {
    #[serde(default = "default_customer_active_check_enabled")]
    pub customer_active_check_enabled: bool,
    #[serde(default = "default_collateralization_from_price_job_interval_secs")]
    #[serde_as(as = "serde_with::DurationSeconds<u64>")]
    #[cfg_attr(feature = "json-schema", schemars(with = "u64"))]
    pub collateralization_from_price_job_interval: std::time::Duration,
    #[serde(default = "default_pending_collateralization_from_price_job_interval_secs")]
    #[serde_as(as = "serde_with::DurationSeconds<u64>")]
    #[cfg_attr(feature = "json-schema", schemars(with = "u64"))]
    pub pending_collateralization_from_price_job_interval: std::time::Duration,
}

impl Default for CreditConfig {
    fn default() -> Self {
        CreditConfig {
            customer_active_check_enabled: default_customer_active_check_enabled(),
            collateralization_from_price_job_interval:
                default_collateralization_from_price_job_interval_secs(),
            pending_collateralization_from_price_job_interval:
                default_pending_collateralization_from_price_job_interval_secs(),
        }
    }
}

fn default_customer_active_check_enabled() -> bool {
    true
}

fn default_collateralization_from_price_job_interval_secs() -> std::time::Duration {
    std::time::Duration::from_secs(30)
}

fn default_pending_collateralization_from_price_job_interval_secs() -> std::time::Duration {
    std::time::Duration::from_secs(30)
}
