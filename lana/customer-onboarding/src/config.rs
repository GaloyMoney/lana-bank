use kratos_admin::KratosAdminConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerOnboardingConfig {
    pub kratos_admin: KratosAdminConfig,
}

impl Default for CustomerOnboardingConfig {
    fn default() -> Self {
        Self {
            kratos_admin: KratosAdminConfig {
                kratos_admin_url: default_kratos_admin_url(),
            },
        }
    }
}

fn default_kratos_admin_url() -> String {
    "http://localhost:4436".to_string()
}
