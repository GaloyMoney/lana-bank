use serde::{Deserialize, Serialize};

use super::smtp::config::SmtpConfig;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EmailConfig {
    pub smtp: SmtpConfig,
    #[serde(default = "default_templates_path")]
    pub templates_path: String,
}

impl Default for EmailConfig {
    fn default() -> Self {
        Self {
            smtp: SmtpConfig::default(),
            templates_path: default_templates_path(),
        }
    }
}

fn default_templates_path() -> String {
    "./templates/email".to_string()
}
