use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositActivityConfig {
    #[serde(default = "default_inactive_threshold_days")]
    inactive_threshold_days: u32,
    #[serde(default = "default_escheatment_threshold_days")]
    escheatment_threshold_days: u32,
}

impl Default for DepositActivityConfig {
    fn default() -> Self {
        Self {
            inactive_threshold_days: default_inactive_threshold_days(),
            escheatment_threshold_days: default_escheatment_threshold_days(),
        }
    }
}

impl DepositActivityConfig {
    pub fn get_inactive_threshold_date(&self, now: DateTime<Utc>) -> DateTime<Utc> {
        now - Duration::days(self.inactive_threshold_days.into())
    }

    pub fn get_escheatment_threshold_date(&self, now: DateTime<Utc>) -> DateTime<Utc> {
        now - Duration::days(self.escheatment_threshold_days.into())
    }
}

fn default_inactive_threshold_days() -> u32 {
    365
}

fn default_escheatment_threshold_days() -> u32 {
    3650
}
