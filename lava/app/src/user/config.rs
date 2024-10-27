use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct UserConfig {
    #[serde(default)]
    pub superuser_email: Option<String>,
}
