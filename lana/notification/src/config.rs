use serde::{Deserialize, Serialize};

use crate::email::EmailConfig;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct NotificationConfig {
    #[serde(default)]
    pub email: EmailConfig,
}
