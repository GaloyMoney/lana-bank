use chrono::{DateTime, Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

// Use January 1st, 2000 as the minimum date
pub const EARLIEST_SEARCH_START: DateTime<Utc> = {
    let date = NaiveDate::from_ymd_opt(2000, 1, 1)
        .expect("valid date")
        .and_hms_opt(0, 0, 0)
        .expect("valid time");
    DateTime::from_naive_utc_and_offset(date, Utc)
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerConfig {
    #[serde(default = "default_inactive_threshold_days")]
    inactive_threshold_days: u32,
    #[serde(default = "default_escheatment_threshold_days")]
    escheatment_threshold_days: u32,
}

impl Default for CustomerConfig {
    fn default() -> Self {
        Self {
            inactive_threshold_days: default_inactive_threshold_days(),
            escheatment_threshold_days: default_escheatment_threshold_days(),
        }
    }
}

impl CustomerConfig {
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
