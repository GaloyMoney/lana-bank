use lettre::address::Address;
use serde::{Deserialize, Serialize};

use domain_config::{ConfigSpec, DomainConfigError, DomainConfigKey, Simple, Visibility};

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(deny_unknown_fields)]
pub struct EmailInfraConfig {
    #[serde(skip)]
    pub username: String,
    #[serde(skip)]
    pub password: String,
    #[serde(default)]
    pub relay: String,
    #[serde(default)]
    pub port: u16,
    #[serde(default)]
    pub insecure: bool,
    #[serde(default)]
    pub admin_panel_url: String,
}

impl EmailInfraConfig {
    pub fn to_smtp_config(&self) -> smtp_client::SmtpConfig {
        smtp_client::SmtpConfig {
            username: self.username.clone(),
            password: self.password.clone(),
            relay: self.relay.clone(),
            port: self.port,
            insecure: self.insecure,
        }
    }
}

pub struct NotificationFromEmailConfigSpec;
impl ConfigSpec for NotificationFromEmailConfigSpec {
    const KEY: DomainConfigKey = DomainConfigKey::new("notification-email-from-email");
    const VISIBILITY: Visibility = Visibility::Exposed;
    type Kind = Simple<String>;

    fn validate(value: &String) -> Result<(), DomainConfigError> {
        if value.trim().is_empty() {
            return Err(DomainConfigError::InvalidState(
                "from_email is required".to_string(),
            ));
        }

        value
            .parse::<Address>()
            .map_err(|e| DomainConfigError::InvalidState(format!("from_email is invalid: {e}")))?;

        Ok(())
    }
}

pub struct NotificationFromNameConfigSpec;
impl ConfigSpec for NotificationFromNameConfigSpec {
    const KEY: DomainConfigKey = DomainConfigKey::new("notification-email-from-name");
    const VISIBILITY: Visibility = Visibility::Exposed;
    type Kind = Simple<String>;

    fn validate(value: &String) -> Result<(), DomainConfigError> {
        if value.trim().is_empty() {
            return Err(DomainConfigError::InvalidState(
                "from_name is required".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_from_email_accepts_valid_address() {
        let config = "notifications@example.com".to_string();
        assert!(<NotificationFromEmailConfigSpec as ConfigSpec>::validate(&config).is_ok());
    }

    #[test]
    fn validate_from_email_rejects_empty_address() {
        let config = "   ".to_string();
        let result = <NotificationFromEmailConfigSpec as ConfigSpec>::validate(&config);

        assert!(matches!(
            result,
            Err(DomainConfigError::InvalidState(msg)) if msg == "from_email is required"
        ));
    }

    #[test]
    fn validate_from_email_rejects_invalid_address() {
        let config = "invalid-email".to_string();
        let result = <NotificationFromEmailConfigSpec as ConfigSpec>::validate(&config);

        assert!(matches!(
            result,
            Err(DomainConfigError::InvalidState(msg))
                if msg.starts_with("from_email is invalid")
        ));
    }

    #[test]
    fn validate_from_name_rejects_empty_from_name() {
        let config = "   ".to_string();
        let result = <NotificationFromNameConfigSpec as ConfigSpec>::validate(&config);

        assert!(matches!(
            result,
            Err(DomainConfigError::InvalidState(msg)) if msg == "from_name is required"
        ));
    }
}
