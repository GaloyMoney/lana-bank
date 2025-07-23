use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositSyncConfig {
    #[serde(default = "default_sumsub_export_enabled")]
    pub sumsub_export_enabled: bool,
}

impl Default for DepositSyncConfig {
    fn default() -> Self {
        Self {
            sumsub_export_enabled: default_sumsub_export_enabled(),
        }
    }
}

fn default_sumsub_export_enabled() -> bool {
    true
}
