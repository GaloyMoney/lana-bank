use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(deny_unknown_fields)]
pub struct EmailConfig {
    #[serde(skip)]
    pub username: String,
    #[serde(skip)]
    pub password: String,
    #[serde(default)]
    pub from_email: String,
    #[serde(default)]
    pub from_name: String,
    #[serde(default)]
    pub relay: String,
    #[serde(default)]
    pub port: u16,
    #[serde(default)]
    pub insecure: bool,
    #[serde(default)]
    pub admin_panel_url: String,
}

impl EmailConfig {
    pub fn to_smtp_config(&self) -> smtp_client::SmtpConfig {
        smtp_client::SmtpConfig {
            username: self.username.clone(),
            password: self.password.clone(),
            from_email: self.from_email.clone(),
            from_name: self.from_name.clone(),
            relay: self.relay.clone(),
            port: self.port,
            insecure: self.insecure,
        }
    }
}
