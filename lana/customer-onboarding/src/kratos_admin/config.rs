use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KratosAdminConfig {
    #[serde(default = "default_kratos_admin_url")]
    pub kratos_admin_url: String,
}

impl Default for KratosAdminConfig {
    fn default() -> Self {
        Self {
            kratos_admin_url: default_kratos_admin_url(),
        }
    }
}

fn default_kratos_admin_url() -> String {
    "http://localhost:4434".to_string()
}
