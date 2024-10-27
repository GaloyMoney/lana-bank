use serde::{Deserialize, Serialize};

use super::CVLPct;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LoanConfig {
    #[serde(default = "default_upgrade_buffer_cvl_pct")]
    pub upgrade_buffer_cvl_pct: CVLPct,
}

impl Default for LoanConfig {
    fn default() -> Self {
        LoanConfig {
            upgrade_buffer_cvl_pct: default_upgrade_buffer_cvl_pct(),
        }
    }
}

fn default_upgrade_buffer_cvl_pct() -> CVLPct {
    CVLPct::new(5)
}
