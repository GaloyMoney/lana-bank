use domain_config::{DomainConfigError, define_exposed_config};

define_exposed_config! {
    pub struct Timezone(String);
    spec {
        key: "timezone";
        default: || Some("UTC".to_string());
        validate: |value: &String| {
            value.parse::<chrono_tz::Tz>().map_err(|_| {
                DomainConfigError::InvalidState(format!("Invalid timezone: {value}"))
            })?;
            Ok(())
        };
    }
}

define_exposed_config! {
    pub struct ClosingTime(String);
    spec {
        key: "closing-time";
        default: || Some("00:00:00".to_string());
        validate: |value: &String| {
            value.parse::<chrono::NaiveTime>().map_err(|_| {
                DomainConfigError::InvalidState(format!("Invalid closing time: {value}"))
            })?;
            Ok(())
        };
    }
}
