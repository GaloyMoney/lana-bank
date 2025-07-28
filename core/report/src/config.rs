use airflow::AirflowConfig;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ReportConfig {
    #[serde(default)]
    pub airflow: AirflowConfig,
}
