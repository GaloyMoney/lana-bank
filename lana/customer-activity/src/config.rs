use chrono::{DateTime, Duration, NaiveTime, Timelike, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCustomerActivityConfig {
    #[serde(default = "default_activity_update_enabled")]
    pub activity_update_enabled: bool,
    #[serde(default = "default_inactive_threshold_days")]
    pub inactive_threshold_days: u32,
    #[serde(default = "default_escheatment_threshold_days")]
    pub escheatment_threshold_days: u32,
    #[serde(default = "default_activity_update_utc_time")]
    pub activity_update_utc_time: String,
}

impl Default for UpdateCustomerActivityConfig {
    fn default() -> Self {
        Self {
            activity_update_enabled: default_activity_update_enabled(),
            inactive_threshold_days: default_inactive_threshold_days(),
            escheatment_threshold_days: default_escheatment_threshold_days(),
            activity_update_utc_time: default_activity_update_utc_time(),
        }
    }
}

impl UpdateCustomerActivityConfig {
    // Parse the activity update time string into hour and minute
    // Expected format: "HH:MM"
    // This time is interpreted as UTC time, regardless of whether the system
    // is running with sim-time or real-time. The job will run at this UTC time
    // every day.
    pub fn parse_activity_update_time(&self) -> Result<(u32, u32), Box<dyn std::error::Error>> {
        let time = NaiveTime::parse_from_str(&self.activity_update_utc_time, "%H:%M")?;
        Ok((time.hour(), time.minute()))
    }

    pub fn get_inactive_threshold_date(&self, now: DateTime<Utc>) -> DateTime<Utc> {
        now - Duration::days(self.inactive_threshold_days.into())
    }

    pub fn get_escheatment_threshold_date(&self, now: DateTime<Utc>) -> DateTime<Utc> {
        now - Duration::days(self.escheatment_threshold_days.into())
    }
}

fn default_activity_update_enabled() -> bool {
    true
}

fn default_inactive_threshold_days() -> u32 {
    365
}

fn default_escheatment_threshold_days() -> u32 {
    3650
}

fn default_activity_update_utc_time() -> String {
    "00:00".to_string()
}
