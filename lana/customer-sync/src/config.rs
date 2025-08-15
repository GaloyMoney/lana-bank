use keycloak_client::KeycloakConnectionConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerSyncConfig {
    #[serde(default = "default_auto_create_deposit_account")]
    pub auto_create_deposit_account: bool,
    #[serde(default = "default_customer_status_sync_active")]
    pub customer_status_sync_active: bool,
    #[serde(default = "default_create_deposit_account_on_customer_create")]
    pub create_deposit_account_on_customer_create: bool,
    #[serde(default = "default_keycloak")]
    pub keycloak: KeycloakConnectionConfig,
    #[serde(default = "default_activity_check_enabled")]
    pub activity_check_enabled: bool,
    #[serde(default = "default_inactive_threshold_days")]
    pub inactive_threshold_days: i64,
    #[serde(default = "default_escheatment_threshold_days")]
    pub escheatment_threshold_days: i64,
    #[serde(default = "default_activity_check_hour")]
    pub activity_check_hour: u32,
    #[serde(default = "default_activity_check_minute")]
    pub activity_check_minute: u32,
}

impl Default for CustomerSyncConfig {
    fn default() -> Self {
        Self {
            auto_create_deposit_account: default_auto_create_deposit_account(),
            customer_status_sync_active: default_customer_status_sync_active(),
            create_deposit_account_on_customer_create:
                default_create_deposit_account_on_customer_create(),
            keycloak: default_keycloak(),
            activity_check_enabled: default_activity_check_enabled(),
            inactive_threshold_days: default_inactive_threshold_days(),
            escheatment_threshold_days: default_escheatment_threshold_days(),
            activity_check_hour: default_activity_check_hour(),
            activity_check_minute: default_activity_check_minute(),
        }
    }
}

fn default_keycloak() -> KeycloakConnectionConfig {
    KeycloakConnectionConfig {
        url: "http://localhost:8081".to_string(),
        client_id: "customer-service-account".to_string(),
        client_secret: "secret".to_string(),
        realm: "customer".to_string(),
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

fn default_activity_check_enabled() -> bool {
    true
}

fn default_inactive_threshold_days() -> i64 {
    365
}

fn default_escheatment_threshold_days() -> i64 {
    3650
}

fn default_activity_check_hour() -> u32 {
    0
}

fn default_activity_check_minute() -> u32 {
    0
}
