use chrono::NaiveTime;
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};

use domain_config::{Complex, ConfigSpec, DomainConfigError, DomainConfigKey, Visibility};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimeEventsConfig {
    pub timezone: Tz,
    pub closing_time: NaiveTime,
}

pub struct TimeEventsConfigSpec;

impl ConfigSpec for TimeEventsConfigSpec {
    const KEY: DomainConfigKey = DomainConfigKey::new("time-events");
    const VISIBILITY: Visibility = Visibility::Internal;
    type Kind = Complex<TimeEventsConfig>;

    fn default_value() -> Option<TimeEventsConfig> {
        Some(TimeEventsConfig {
            timezone: Tz::default(),
            closing_time: NaiveTime::default(),
        })
    }

    fn validate(_value: &TimeEventsConfig) -> Result<(), DomainConfigError> {
        Ok(())
    }
}
