use keycloak_client::KeycloakConnectionConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct UserOnboardingConfig {
    #[serde(default)]
    pub keycloak: KeycloakConnectionConfig,
}
