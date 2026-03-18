use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use job::*;

pub const OBLIGATION_TRANSITION_PROCESS_JOB_TYPE: JobType =
    JobType::new("task.eod.obligation-transition-process");

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObligationTransitionProcessConfig {
    pub date: NaiveDate,
}

pub type ObligationTransitionProcessSpawner = JobSpawner<ObligationTransitionProcessConfig>;
