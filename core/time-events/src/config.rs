use chrono::NaiveTime;
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};

use domain_config::{
    Complex, ConfigSpec, DomainConfigError, DomainConfigKey, ExposedConfig, Visibility,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimezoneConfig {
    pub value: Tz,
}

impl ConfigSpec for TimezoneConfig {
    const KEY: DomainConfigKey = DomainConfigKey::new("timezone");
    const VISIBILITY: Visibility = Visibility::Exposed;
    type Kind = Complex<TimezoneConfig>;

    fn default_value() -> Option<TimezoneConfig> {
        Some(TimezoneConfig { value: Tz::UTC })
    }

    fn validate(_value: &TimezoneConfig) -> Result<(), DomainConfigError> {
        Ok(())
    }
}

impl ExposedConfig for TimezoneConfig {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClosingTimeConfig {
    pub value: NaiveTime,
}

impl ConfigSpec for ClosingTimeConfig {
    const KEY: DomainConfigKey = DomainConfigKey::new("closing-time");
    const VISIBILITY: Visibility = Visibility::Exposed;
    type Kind = Complex<ClosingTimeConfig>;

    fn default_value() -> Option<ClosingTimeConfig> {
        Some(ClosingTimeConfig {
            value: NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
        })
    }

    fn validate(_value: &ClosingTimeConfig) -> Result<(), DomainConfigError> {
        Ok(())
    }
}

impl ExposedConfig for ClosingTimeConfig {}
