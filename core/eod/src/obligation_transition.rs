use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use job::*;

pub const OBLIGATION_TRANSITION_JOB_TYPE: JobType = JobType::new("task.eod.obligation-transition");

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObligationTransitionConfig {
    pub date: NaiveDate,
}

pub type ObligationTransitionJobSpawner = JobSpawner<ObligationTransitionConfig>;
