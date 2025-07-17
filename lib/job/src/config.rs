use serde::{Deserialize, Serialize};

use std::time::Duration;

#[serde_with::serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JobsConfig {
    #[serde_as(as = "serde_with::DurationSeconds<u64>")]
    #[serde(default = "default_job_lost_interval")]
    pub job_lost_interval: Duration,
    #[serde(default = "default_max_jobs_per_process")]
    pub max_jobs_per_process: usize,
    #[serde(default = "default_min_jobs_per_process")]
    pub min_jobs_per_process: usize,
}

impl Default for JobsConfig {
    fn default() -> Self {
        Self {
            job_lost_interval: default_job_lost_interval(),
            max_jobs_per_process: default_max_jobs_per_process(),
            min_jobs_per_process: default_min_jobs_per_process(),
        }
    }
}

fn default_job_lost_interval() -> Duration {
    Duration::from_secs(60)
}

fn default_max_jobs_per_process() -> usize {
    50
}

fn default_min_jobs_per_process() -> usize {
    25
}
