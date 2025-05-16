use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DepositConfig {
    #[serde(default = "default_chart_of_accounts_config_path")]
    pub chart_of_accounts_config_path: Option<PathBuf>,
}

impl Default for DepositConfig {
    fn default() -> Self {
        DepositConfig {
            chart_of_accounts_config_path: default_chart_of_accounts_config_path(),
        }
    }
}

fn default_chart_of_accounts_config_path() -> Option<PathBuf> {
    None
}
