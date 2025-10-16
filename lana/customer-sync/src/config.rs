use keycloak_client::KeycloakConnectionConfig;
use serde::{Deserialize, Serialize};

#[serde_with::serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CustomerSyncConfig {
    #[serde(default = "default_customer_status_sync_active")]
    pub customer_status_sync_active: bool,
    #[serde(default = "default_create_deposit_account_on_customer_create")]
    pub create_deposit_account_on_customer_create: bool,
    #[serde(default = "default_keycloak")]
    pub keycloak: KeycloakConnectionConfig,
    #[serde(default = "default_activity_update_job_interval_secs")]
    #[serde_as(as = "serde_with::DurationSeconds<u64>")]
    pub activity_update_job_interval: std::time::Duration,
}

impl Default for CustomerSyncConfig {
    fn default() -> Self {
        Self {
            customer_status_sync_active: default_customer_status_sync_active(),
            create_deposit_account_on_customer_create:
                default_create_deposit_account_on_customer_create(),
            keycloak: default_keycloak(),
            activity_update_job_interval: default_activity_update_job_interval_secs(),
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

fn default_customer_status_sync_active() -> bool {
    true
}

fn default_create_deposit_account_on_customer_create() -> bool {
    false
}

fn default_activity_update_job_interval_secs() -> std::time::Duration {
    std::time::Duration::from_secs(24 * 60 * 60) // 24 hours
}
