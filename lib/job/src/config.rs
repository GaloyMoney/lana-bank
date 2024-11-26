use serde::{Deserialize, Serialize};

use std::time::Duration;

#[serde_with::serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JobExecutorConfig {
    #[serde_as(as = "serde_with::DurationSeconds<u64>")]
    #[serde(default = "default_poll_interval")]
    pub poll_interval: Duration,
    #[serde(default = "default_max_jobs_per_process")]
    pub max_jobs_per_process: usize,
    #[serde(default = "default_min_jobs_per_process")]
    pub min_jobs_per_process: usize,
}

impl Default for JobExecutorConfig {
    fn default() -> Self {
        Self {
            poll_interval: default_poll_interval(),
            max_jobs_per_process: default_max_jobs_per_process(),
            min_jobs_per_process: default_min_jobs_per_process(),
        }
    }
}

fn default_poll_interval() -> Duration {
    Duration::from_secs(5)
}

fn default_max_jobs_per_process() -> usize {
    20
}

fn default_min_jobs_per_process() -> usize {
    10
}
