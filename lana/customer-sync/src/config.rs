use keycloak_admin::KeycloakAdminConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerSyncConfig {
    #[serde(default = "default_auto_create_deposit_account")]
    pub auto_create_deposit_account: bool,
    #[serde(default = "default_customer_status_sync_active")]
    pub customer_status_sync_active: bool,
    #[serde(default = "default_keycloak_admin_config")]
    pub keycloak_admin: KeycloakAdminConfig,
    #[serde(default = "default_create_deposit_account_on_customer_create")]
    pub create_deposit_account_on_customer_create: bool,
}

impl Default for CustomerSyncConfig {
    fn default() -> Self {
        Self {
            auto_create_deposit_account: default_auto_create_deposit_account(),
            keycloak_admin: default_keycloak_admin_config(),
            customer_status_sync_active: default_customer_status_sync_active(),
            create_deposit_account_on_customer_create:
                default_create_deposit_account_on_customer_create(),
        }
    }
}

fn default_keycloak_admin_config() -> KeycloakAdminConfig {
    KeycloakAdminConfig {
        realm: "lana-customer".to_string(),
        ..KeycloakAdminConfig::default()
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
