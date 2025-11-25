use chrono::NaiveTime;
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(from = "TimeEventsConfigRaw", into = "TimeEventsConfigRaw")]
pub struct TimeEventsConfig {
    pub daily: DailyConfig,
}

#[derive(Debug, Clone)]
pub struct DailyConfig {
    pub closing_time: NaiveTime,
    pub timezone: Tz,
}

// Internal struct for disk serialization
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct TimeEventsConfigRaw {
    #[serde(default)]
    daily: DailyConfigRaw,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DailyConfigRaw {
    closing_time: String,
    timezone: String,
}

impl Default for TimeEventsConfig {
    fn default() -> Self {
        Self::from(TimeEventsConfigRaw::default())
    }
}

impl Default for DailyConfigRaw {
    fn default() -> Self {
        Self {
            closing_time: "23:59:00".to_string(),
            timezone: "UTC".to_string(),
        }
    }
}

impl From<TimeEventsConfigRaw> for TimeEventsConfig {
    fn from(raw: TimeEventsConfigRaw) -> Self {
        Self {
            daily: DailyConfig::from(raw.daily),
        }
    }
}

impl From<DailyConfigRaw> for DailyConfig {
    fn from(raw: DailyConfigRaw) -> Self {
        let timezone = raw
            .timezone
            .parse()
            .unwrap_or_else(|_| panic!("Invalid timezone in config: {}", raw.timezone));

        let closing_time = NaiveTime::parse_from_str(&raw.closing_time, "%H:%M:%S")
            .unwrap_or_else(|_| panic!("Invalid time format in config: {}", raw.closing_time));

        Self {
            closing_time,
            timezone,
        }
    }
}

impl From<TimeEventsConfig> for TimeEventsConfigRaw {
    fn from(config: TimeEventsConfig) -> Self {
        Self {
            daily: config.daily.into(),
        }
    }
}

impl From<DailyConfig> for DailyConfigRaw {
    fn from(config: DailyConfig) -> Self {
        Self {
            closing_time: config.closing_time.format("%H:%M:%S").to_string(),
            timezone: config.timezone.to_string(),
        }
    }
}
