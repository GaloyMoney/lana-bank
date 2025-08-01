use airflow::AirflowConfig;
use serde::{Deserialize, Serialize};

use std::time::Duration;

#[serde_with::serde_as]
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ReportConfig {
    #[serde(default)]
    pub airflow: AirflowConfig,
    #[serde_as(as = "serde_with::DurationSeconds<u64>")]
    #[serde(default = "default_find_new_report_run_job_interval")]
    pub find_new_report_run_job_interval: Duration,
}

fn default_find_new_report_run_job_interval() -> Duration {
    Duration::from_secs(60 * 1)
}
