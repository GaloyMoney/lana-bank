use serde::{Deserialize, Serialize};

use super::KratosConfig;

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct CustomerConfig {
    #[serde(default)]
    pub kratos: KratosConfig,
}
