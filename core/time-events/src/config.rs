use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeEventsConfig {
    pub daily: DailyConfig,
}

impl Default for TimeEventsConfig {
    fn default() -> Self {
        Self {
            daily: DailyConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyConfig {
    pub closing_time: String,
    pub timezone: String,
}

impl Default for DailyConfig {
    fn default() -> Self {
        Self {
            closing_time: "23:59:00".to_string(),
            timezone: "UTC".to_string(),
        }
    }
}
