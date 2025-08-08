use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserOnboardingConfig {
    #[serde(default = "default_keycloak_realm")]
    pub keycloak_realm: String,
}

impl Default for UserOnboardingConfig {
    fn default() -> Self {
        Self {
            keycloak_realm: default_keycloak_realm(),
        }
    }
}

fn default_keycloak_realm() -> String {
    "internal".to_string()
}
