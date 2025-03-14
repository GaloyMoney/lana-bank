use kratos_admin::KratosAdminConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerOnboardingConfig {
    #[serde(default = "default_auto_create_deposit_account")]
    pub auto_create_deposit_account: bool,
    pub customer_status_sync_active: bool,
    pub kratos_admin: KratosAdminConfig,
}

impl Default for CustomerOnboardingConfig {
    fn default() -> Self {
        Self {
            auto_create_deposit_account: default_auto_create_deposit_account(),
            kratos_admin: KratosAdminConfig {
                kratos_admin_url: default_kratos_admin_url(),
            },
            customer_status_sync_active: default_customer_status_sync_active(),
        }
    }
}

fn default_kratos_admin_url() -> String {
    "http://localhost:4436".to_string()
}

fn default_auto_create_deposit_account() -> bool {
    true
}

fn default_customer_status_sync_active() -> bool {
    true
}
