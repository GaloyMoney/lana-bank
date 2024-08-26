use serde::{Deserialize, Serialize};

use crate::primitives::SECS_IN_1_MIN;

use super::{CVLPct, StalePriceInterval};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LoanConfig {
    #[serde(default = "default_stale_price_interval")]
    pub stale_price_interval: StalePriceInterval,
    #[serde(default = "default_collateral_upgrade_buffer")]
    pub collateral_upgrade_buffer: CVLPct,
}

impl Default for LoanConfig {
    fn default() -> Self {
        LoanConfig {
            stale_price_interval: default_stale_price_interval(),
            collateral_upgrade_buffer: default_collateral_upgrade_buffer(),
        }
    }
}

fn default_stale_price_interval() -> StalePriceInterval {
    StalePriceInterval::new(std::time::Duration::from_secs(20 * SECS_IN_1_MIN))
}

fn default_collateral_upgrade_buffer() -> CVLPct {
    CVLPct::new(5)
}
