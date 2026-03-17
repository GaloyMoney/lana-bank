use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use job::*;

pub const DEPOSIT_ACTIVITY_JOB_TYPE: JobType = JobType::new("task.eod.deposit-activity");

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DepositActivityConfig {
    pub date: NaiveDate,
}

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DepositActivityState {
    #[default]
    Collecting,
    Tracking {
        total: usize,
        completed: usize,
        entity_jobs: Vec<JobId>,
    },
    Completed,
}

pub type DepositActivityJobSpawner = JobSpawner<DepositActivityConfig>;
