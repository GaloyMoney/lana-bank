use serde::{Deserialize, Serialize};

use super::smtp::config::SmtpConfig;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EmailConfig {
    pub smtp: SmtpConfig,
}

impl Default for EmailConfig {
    fn default() -> Self {
        Self {
            smtp: SmtpConfig::default(),
        }
    }
}
