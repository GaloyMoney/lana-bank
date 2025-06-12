use serde::{Deserialize, Serialize};

use crate::email::EmailConfig;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct NotificationConfig {
    #[serde(default)]
    pub email: EmailConfig,
}
