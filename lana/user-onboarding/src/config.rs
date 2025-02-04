use kratos_admin::KratosAdminConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct UserOnboardingConfig {
    pub kratos_admin: KratosAdminConfig,
}
