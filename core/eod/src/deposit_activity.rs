use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use job::*;

pub const DEPOSIT_ACTIVITY_JOB_TYPE: JobType = JobType::new("task.eod.deposit-activity");

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DepositActivityConfig {
    pub date: NaiveDate,
    pub closing_time: DateTime<Utc>,
}

pub type DepositActivityJobSpawner = JobSpawner<DepositActivityConfig>;
