use chrono::NaiveTime;
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};

use domain_config::{
    Complex, ConfigSpec, ConfigType, DomainConfigError, DomainConfigKey, ExposedConfig, Visibility,
    inventory,
};

// TODO: Need to rethink use of domain configs for this, current implementation can be changed
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

inventory::submit! {
    domain_config::registry::ConfigSpecEntry {
        key: "timezone",
        visibility: Visibility::Exposed,
        config_type: ConfigType::Complex,
        validate_json: <TimezoneConfig as ConfigSpec>::validate_json,
    }
}

inventory::submit! {
    domain_config::registry::ConfigSpecEntry {
        key: "closing-time",
        visibility: Visibility::Exposed,
        config_type: ConfigType::Complex,
        validate_json: <ClosingTimeConfig as ConfigSpec>::validate_json,
    }
}
