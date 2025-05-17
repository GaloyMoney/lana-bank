use serde::{Deserialize, Serialize};

use crate::primitives::CVLPct;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CreditConfig {
    #[serde(default = "default_upgrade_buffer_cvl_pct")]
    pub upgrade_buffer_cvl_pct: CVLPct,
    #[serde(default = "default_customer_active_check_enabled")]
    pub customer_active_check_enabled: bool,
}

impl Default for CreditConfig {
    fn default() -> Self {
        CreditConfig {
            upgrade_buffer_cvl_pct: default_upgrade_buffer_cvl_pct(),
            customer_active_check_enabled: default_customer_active_check_enabled(),
        }
    }
}

fn default_upgrade_buffer_cvl_pct() -> CVLPct {
    CVLPct::new(5)
}

fn default_customer_active_check_enabled() -> bool {
    true
}
