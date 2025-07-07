use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct AccessConfig {
    pub superuser_email: Option<String>,
}
