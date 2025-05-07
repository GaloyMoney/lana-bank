use kratos_admin::KratosAdminConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerSyncConfig {
    #[serde(default = "default_auto_create_deposit_account")]
    pub auto_create_deposit_account: bool,
    #[serde(default = "default_customer_status_sync_active")]
    pub customer_status_sync_active: bool,
    #[serde(default = "default_kratos_admin_config")]
    pub kratos_admin: KratosAdminConfig,
    #[serde(default = "default_create_deposit_account_on_customer_create")]
    pub create_deposit_account_on_customer_create: bool,
}

impl Default for CustomerSyncConfig {
    fn default() -> Self {
        Self {
            auto_create_deposit_account: default_auto_create_deposit_account(),
            kratos_admin: default_kratos_admin_config(),
            customer_status_sync_active: default_customer_status_sync_active(),
            create_deposit_account_on_customer_create:
                default_create_deposit_account_on_customer_create(),
        }
    }
}

fn default_kratos_admin_config() -> KratosAdminConfig {
    KratosAdminConfig {
        kratos_admin_url: "http://localhost:4436".to_string(),
    }
}

fn default_auto_create_deposit_account() -> bool {
    true
}

fn default_customer_status_sync_active() -> bool {
    true
}

fn default_create_deposit_account_on_customer_create() -> bool {
    false
}
