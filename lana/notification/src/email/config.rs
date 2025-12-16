use lettre::address::Address;
use serde::{Deserialize, Serialize};

use domain_config::{DomainConfigError, DomainConfigKey, DomainConfigValue};

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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct NotificationEmailConfig {
    pub from_email: String,
    pub from_name: String,
}

impl DomainConfigValue for NotificationEmailConfig {
    const KEY: DomainConfigKey = DomainConfigKey::new("notification-email");

    fn validate(&self) -> Result<(), DomainConfigError> {
        // DomainConfigError is smelly here
        // we're looking at if an email exist and is well formatted
        // this has nothing to do with DomainConfig really
        // it's proper business logic to the notification email

        if self.from_email.trim().is_empty() {
            return Err(DomainConfigError::InvalidState(
                "from_email is required".to_string(),
            ));
        }

        if self.from_name.trim().is_empty() {
            // from name can be as long as one want and create
            // some burden to postgres
            //
            // the default implementation is overriden here
            // those function are not additional, it's this one OR the default. 
            return Err(DomainConfigError::InvalidState(
                "from_name is required".to_string(),
            ));
        }

        self.from_email
            .parse::<Address>()
            .map_err(|e| DomainConfigError::InvalidState(format!("from_email is invalid: {e}")))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_email_config_accepts_valid_address() {
        let config = NotificationEmailConfig {
            from_email: "notifications@example.com".to_string(),
            from_name: "Notifier".to_string(),
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn validate_email_config_rejects_empty_address() {
        let config = NotificationEmailConfig {
            from_email: "   ".to_string(),
            from_name: "Notifier".to_string(),
        };

        let result = config.validate();

        assert!(matches!(
            result,
            Err(DomainConfigError::InvalidState(msg)) if msg == "from_email is required"
        ));
    }

    #[test]
    fn validate_email_config_rejects_empty_from_name() {
        let config = NotificationEmailConfig {
            from_email: "notifications@example.com".to_string(),
            from_name: "   ".to_string(),
        };

        let result = config.validate();

        assert!(matches!(
            result,
            Err(DomainConfigError::InvalidState(msg)) if msg == "from_name is required"
        ));
    }

    #[test]
    fn validate_email_config_rejects_invalid_address() {
        let config = NotificationEmailConfig {
            from_email: "invalid-email".to_string(),
            from_name: "Notifier".to_string(),
        };

        let result = config.validate();

        assert!(matches!(
            result,
            Err(DomainConfigError::InvalidState(msg))
                if msg.starts_with("from_email is invalid")
        ));
    }
}
