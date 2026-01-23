use lettre::address::Address;
use serde::{Deserialize, Serialize};
use url::Url;

use domain_config::{DomainConfigError, define_exposed_config};

#[derive(Clone, Debug, Deserialize, Serialize)]
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
    #[serde(default = "default_admin_panel_url")]
    pub admin_panel_url: Url,
}

fn default_admin_panel_url() -> Url {
    Url::parse("http://localhost:3000").expect("valid default URL")
}

impl Default for EmailInfraConfig {
    fn default() -> Self {
        Self {
            username: String::new(),
            password: String::new(),
            relay: String::new(),
            port: 0,
            insecure: false,
            admin_panel_url: default_admin_panel_url(),
        }
    }
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

define_exposed_config! {
    pub struct NotificationFromEmail(String);
    spec {
        key: "notification-email-from-email";
        validate: |value: &String| {
            if value.trim().is_empty() {
                return Err(DomainConfigError::InvalidState(
                    "from_email is required".to_string(),
                ));
            }

            value.parse::<Address>().map_err(|e| {
                DomainConfigError::InvalidState(format!("from_email is invalid: {e}"))
            })?;

            Ok(())
        };
    }
}

define_exposed_config! {
    pub struct NotificationFromName(String);
    spec {
        key: "notification-email-from-name";
        validate: |value: &String| {
            if value.trim().is_empty() {
                return Err(DomainConfigError::InvalidState(
                    "from_name is required".to_string(),
                ));
            }

            Ok(())
        };
    }
}

#[cfg(test)]
mod tests {
    use domain_config::ConfigSpec;

    use super::*;

    #[test]
    fn validate_from_email_accepts_valid_address() {
        let config = "notifications@example.com".to_string();
        assert!(<NotificationFromEmail as ConfigSpec>::validate(&config).is_ok());
    }

    #[test]
    fn validate_from_email_rejects_empty_address() {
        let config = "   ".to_string();
        let result = <NotificationFromEmail as ConfigSpec>::validate(&config);

        assert!(matches!(
            result,
            Err(DomainConfigError::InvalidState(msg)) if msg == "from_email is required"
        ));
    }

    #[test]
    fn validate_from_email_rejects_invalid_address() {
        let config = "invalid-email".to_string();
        let result = <NotificationFromEmail as ConfigSpec>::validate(&config);

        assert!(matches!(
            result,
            Err(DomainConfigError::InvalidState(msg))
                if msg.starts_with("from_email is invalid")
        ));
    }

    #[test]
    fn validate_from_name_rejects_empty_from_name() {
        let config = "   ".to_string();
        let result = <NotificationFromName as ConfigSpec>::validate(&config);

        assert!(matches!(
            result,
            Err(DomainConfigError::InvalidState(msg)) if msg == "from_name is required"
        ));
    }
}
