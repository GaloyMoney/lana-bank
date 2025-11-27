use serde::{Deserialize, Serialize};

use domain_configurations::{ConfigKey, DomainConfigurationKey};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExampleConfig {
    pub feature_enabled: bool,
    pub threshold: u32,
}

pub struct ExampleConfigKey;

impl ConfigKey<ExampleConfig> for ExampleConfigKey {
    fn key() -> DomainConfigurationKey {
        DomainConfigurationKey::new("example-config")
    }
}
