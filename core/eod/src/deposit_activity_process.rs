use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use job::*;

pub const DEPOSIT_ACTIVITY_PROCESS_JOB_TYPE: JobType =
    JobType::new("task.eod.deposit-activity-process");

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DepositActivityProcessConfig {
    pub date: NaiveDate,
    pub closing_time: DateTime<Utc>,
}

pub type DepositActivityProcessSpawner = JobSpawner<DepositActivityProcessConfig>;
