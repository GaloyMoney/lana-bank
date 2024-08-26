use serde::{Deserialize, Serialize};

use super::CVLPct;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LoanConfig {
    #[serde(default = "default_collateral_upgrade_buffer")]
    pub collateral_upgrade_buffer: CVLPct,
}

impl Default for LoanConfig {
    fn default() -> Self {
        LoanConfig {
            collateral_upgrade_buffer: default_collateral_upgrade_buffer(),
        }
    }
}

fn default_collateral_upgrade_buffer() -> CVLPct {
    CVLPct::new(5)
}
