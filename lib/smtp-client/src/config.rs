use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(deny_unknown_fields)]
pub struct SmtpConfig {
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
}
