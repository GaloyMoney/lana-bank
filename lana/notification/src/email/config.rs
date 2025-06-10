use serde::{Deserialize, Serialize};

use super::smtp::config::SmtpConfig;

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct EmailConfig {
    #[serde(default)]
    pub smtp: SmtpConfig,
}
