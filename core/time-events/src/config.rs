use chrono::NaiveTime;
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};

use domain_config::{DomainConfigError, DomainConfigKey, DomainConfigValue};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct TimezoneConfig {
    pub timezone: Tz,
}

impl DomainConfigValue for TimezoneConfig {
    const KEY: DomainConfigKey = DomainConfigKey::new("timezone");

    fn validate(&self) -> Result<(), DomainConfigError> {
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ClosingTimeConfig {
    pub closing_time: NaiveTime,
}

impl DomainConfigValue for ClosingTimeConfig {
    const KEY: DomainConfigKey = DomainConfigKey::new("closing-time");

    fn validate(&self) -> Result<(), DomainConfigError> {
        Ok(())
    }
}
