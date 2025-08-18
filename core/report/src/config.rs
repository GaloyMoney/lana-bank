use airflow::AirflowConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportConfig {
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub airflow: AirflowConfig,
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            airflow: AirflowConfig::default(),
        }
    }
}

fn default_enabled() -> bool {
    std::env::var("DATA_PIPELINE")
        .map(|val| val.to_lowercase() == "true")
        .unwrap_or(false)
}
