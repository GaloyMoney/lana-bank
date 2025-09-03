use chrono::{DateTime, Duration, NaiveTime, Timelike, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCustomerActivityConfig {
    #[serde(default = "default_activity_update_enabled")]
    pub activity_update_enabled: bool,
    #[serde(default = "default_inactive_threshold_days")]
    inactive_threshold_days: u32,
    #[serde(default = "default_escheatment_threshold_days")]
    escheatment_threshold_days: u32,
    #[serde(default)]
    activity_update_utc_time: ActivityCheckJobRunTime,
}

impl Default for UpdateCustomerActivityConfig {
    fn default() -> Self {
        Self {
            activity_update_enabled: default_activity_update_enabled(),
            inactive_threshold_days: default_inactive_threshold_days(),
            escheatment_threshold_days: default_escheatment_threshold_days(),
            activity_update_utc_time: Default::default(),
        }
    }
}

impl UpdateCustomerActivityConfig {
    pub fn activity_check_run_time(&self) -> &ActivityCheckJobRunTime {
        &self.activity_update_utc_time
    }

    pub fn get_inactive_threshold_date(&self, now: DateTime<Utc>) -> DateTime<Utc> {
        now - Duration::days(self.inactive_threshold_days.into())
    }

    pub fn get_escheatment_threshold_date(&self, now: DateTime<Utc>) -> DateTime<Utc> {
        now - Duration::days(self.escheatment_threshold_days.into())
    }
}

#[derive(Default, Debug, Clone)]
pub struct ActivityCheckJobRunTime {
    hours_past_midnight: u32,
    minutes_past_hour: u32,
}

impl ActivityCheckJobRunTime {
    pub fn next_after(&self, after: DateTime<Utc>) -> DateTime<Utc> {
        let tomorrow = after + Duration::days(1);

        let midnight = tomorrow
            .date_naive()
            .and_hms_opt(self.hours_past_midnight, self.minutes_past_hour, 0)
            .expect("Cannot update time");

        midnight
            .and_local_timezone(Utc)
            .single()
            .expect("Cannot update time")
    }
}

impl<'de> Deserialize<'de> for ActivityCheckJobRunTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let time = NaiveTime::parse_from_str(&s, "%H:%M")
            .map_err(|e| serde::de::Error::custom(format!("Invalid time format '{}': {}", s, e)))?;

        Ok(ActivityCheckJobRunTime {
            hours_past_midnight: time.hour(),
            minutes_past_hour: time.minute(),
        })
    }
}

impl Serialize for ActivityCheckJobRunTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let time_str = format!(
            "{:02}:{:02}",
            self.hours_past_midnight, self.minutes_past_hour
        );
        serializer.serialize_str(&time_str)
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
