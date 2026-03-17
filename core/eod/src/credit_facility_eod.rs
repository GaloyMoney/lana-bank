use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use job::*;

pub const CREDIT_FACILITY_EOD_JOB_TYPE: JobType = JobType::new("task.eod.credit-facility-eod");

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreditFacilityEodConfig {
    pub date: NaiveDate,
}

pub type CreditFacilityEodJobSpawner = JobSpawner<CreditFacilityEodConfig>;
