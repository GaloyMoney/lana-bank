use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use job::*;

pub const CREDIT_FACILITY_EOD_PROCESS_JOB: JobType =
    JobType::new("process.eod.credit-facility-eod-process");

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreditFacilityEodProcessConfig {
    pub date: NaiveDate,
}

pub type CreditFacilityEodProcessSpawner = JobSpawner<CreditFacilityEodProcessConfig>;
