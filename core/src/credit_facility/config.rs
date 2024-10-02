use serde::{Deserialize, Serialize};

use crate::terms::CVLPct;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CreditFacilityConfig {
    #[serde(default = "default_upgrade_buffer_cvl_pct")]
    pub upgrade_buffer_cvl_pct: CVLPct,
}

impl Default for CreditFacilityConfig {
    fn default() -> Self {
        CreditFacilityConfig {
            upgrade_buffer_cvl_pct: default_upgrade_buffer_cvl_pct(),
        }
    }
}

fn default_upgrade_buffer_cvl_pct() -> CVLPct {
    CVLPct::new(5)
}
