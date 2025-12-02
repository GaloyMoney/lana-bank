use airflow::AirflowConfig;
use dagster::DagsterConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReportConfig {
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub airflow: AirflowConfig,
    #[serde(default)]
    pub dagster: DagsterConfig,
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            airflow: AirflowConfig::default(),
            dagster: DagsterConfig::default(),
        }
    }
}

fn default_enabled() -> bool {
    std::env::var("DATA_PIPELINE")
        .map(|val| val.to_lowercase() == "true")
        .unwrap_or(false)
}
