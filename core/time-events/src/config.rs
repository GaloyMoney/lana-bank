use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TimeEventsConfig {
    #[serde(default)]
    pub daily: DailyConfig,
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
