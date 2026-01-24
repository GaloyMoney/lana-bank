use keycloak_client::{KeycloakConnectionConfig, Url};
use serde::{Deserialize, Serialize};

#[serde_with::serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CustomerSyncConfig {
    #[serde(default = "default_keycloak")]
    pub keycloak: KeycloakConnectionConfig,
    #[serde(default = "default_activity_update_job_interval_secs")]
    #[serde_as(as = "serde_with::DurationSeconds<u64>")]
    pub activity_update_job_interval: std::time::Duration,
}

impl Default for CustomerSyncConfig {
    fn default() -> Self {
        Self {
            keycloak: default_keycloak(),
            activity_update_job_interval: default_activity_update_job_interval_secs(),
        }
    }
}

fn default_keycloak() -> KeycloakConnectionConfig {
    KeycloakConnectionConfig {
        url: Url::parse("http://localhost:8081").expect("valid default URL"),
        client_id: "customer-service-account".to_string(),
        client_secret: "secret".to_string(),
        realm: "customer".to_string(),
    }
}

fn default_activity_update_job_interval_secs() -> std::time::Duration {
    std::time::Duration::from_secs(24 * 60 * 60) // 24 hours
}
