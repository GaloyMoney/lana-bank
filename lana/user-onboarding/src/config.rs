use keycloak_admin::KeycloakAdminConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct UserOnboardingConfig {
    pub keycloak_admin: KeycloakAdminConfig,
}
