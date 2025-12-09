use chrono::NaiveTime;
use chrono_tz::Tz;
use serde::Deserialize;
use crate::error::TimeEventsError;

#[derive(Debug, Clone, Deserialize)]
#[serde(from = "TimeEventsConfigRaw", into = "TimeEventsConfigRaw")]
pub struct TimeEventsConfig {
    pub daily: DailyConfig,
}

#[derive(Debug, Clone)]
pub struct DailyConfig {
    pub closing_time: NaiveTime,
    pub timezone: Tz,
}

// Raw struct for disk serialization
#[derive(Debug, Clone, Deserialize, Default)]
struct TimeEventsConfigRaw {
    #[serde(default)]
    daily: DailyConfigRaw,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct DailyConfigRaw {
    #[serde(default = "default_closing_time")]
    closing_time: String,
    #[serde(default = "default_timezone")]
    timezone: String,
}

fn default_closing_time() -> String {
    "23:59:00".to_string()
}

fn default_timezone() -> String {
    "UTC".to_string()
}

impl Default for TimeEventsConfig {
    fn default() -> Self {
        Self::from(TimeEventsConfigRaw::default())
    }
}

impl TryFrom<TimeEventsConfigRaw> for TimeEventsConfig {
    type Error = TimeEventsError;

    fn try_from(raw: TimeEventsConfigRaw) -> Result<Self, Self::Error> {
        Ok(Self {
            daily: DailyConfig::try_from(raw.daily)?,
        })
    }
}

impl TryFrom<DailyConfigRaw> for DailyConfig {
    type Error = TimeEventsError;

    fn try_from(raw: DailyConfigRaw) -> Result<Self, Self::Error> {
        let timezone = raw
            .timezone
            .parse()
            .unwrap_or_else(|_| panic!("Invalid timezone in config: {}", raw.timezone));

        let closing_time = NaiveTime::parse_from_str(&raw.closing_time, "%H:%M:%S")
            .unwrap_or_else(|_| panic!("Invalid time format in config: {}", raw.closing_time));

        Ok(Self {
            closing_time,
            timezone,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn other_timezone() {
        let config = DailyConfigRaw {
            closing_time: "23:59:00".to_owned(),
            timezone: "EST".to_owned(),
        };

        let _: DailyConfig = config.into();
    }

    #[test]
    fn will_panick() {
        let config = DailyConfigRaw {
            closing_time: "23:59:00".to_owned(),
            timezone: "EST2".to_owned(),
        };

        let _: DailyConfig = config.into();
    }
}
