use keycloak_client::KeycloakConnectionConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserOnboardingConfig {
    #[serde(default)]
    pub keycloak: KeycloakConnectionConfig,
}
