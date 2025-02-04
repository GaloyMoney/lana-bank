use serde::{Deserialize, Serialize};

use super::kratos_admin::KratosAdminConfig;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CustomerOnboardingConfig {
    pub kratos_admin: KratosAdminConfig,
}
