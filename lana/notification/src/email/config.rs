use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
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
