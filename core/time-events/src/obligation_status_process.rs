use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use job::*;

pub const OBLIGATION_STATUS_PROCESS_JOB_TYPE: JobType =
    JobType::new("task.eod.obligation-status-process");

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObligationStatusProcessConfig {
    pub date: NaiveDate,
}

pub type ObligationStatusProcessSpawner = JobSpawner<ObligationStatusProcessConfig>;
