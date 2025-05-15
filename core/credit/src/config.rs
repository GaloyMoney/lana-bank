use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::primitives::CVLPct;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CreditConfig {
    #[serde(default = "default_upgrade_buffer_cvl_pct")]
    pub upgrade_buffer_cvl_pct: CVLPct,
    #[serde(default = "default_customer_active_check_enabled")]
    pub customer_active_check_enabled: bool,
    #[serde(default = "default_chart_of_accounts_config_path")]
    pub chart_of_accounts_config_path: Option<PathBuf>,
}

impl Default for CreditConfig {
    fn default() -> Self {
        CreditConfig {
            upgrade_buffer_cvl_pct: default_upgrade_buffer_cvl_pct(),
            customer_active_check_enabled: default_customer_active_check_enabled(),
            chart_of_accounts_config_path: default_chart_of_accounts_config_path(),
        }
    }
}

fn default_upgrade_buffer_cvl_pct() -> CVLPct {
    CVLPct::new(5)
}

fn default_customer_active_check_enabled() -> bool {
    true
}

fn default_chart_of_accounts_config_path() -> Option<PathBuf> {
    None
}
