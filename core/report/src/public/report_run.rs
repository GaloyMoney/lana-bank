use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use crate::{
    primitives::ReportRunId,
    report_run::{ReportRun, ReportRunState, ReportRunType, RequestedReport},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct PublicReportRun {
    pub id: ReportRunId,
    pub external_id: String,
    pub state: ReportRunState,
    pub run_type: ReportRunType,
    pub requested_report: Option<RequestedReport>,
    pub requested_as_of_date: Option<chrono::NaiveDate>,
}

impl From<&ReportRun> for PublicReportRun {
    fn from(entity: &ReportRun) -> Self {
        PublicReportRun {
            id: entity.id,
            external_id: entity.external_id.clone(),
            state: entity.state,
            run_type: entity.run_type,
            requested_report: entity.requested_report.clone(),
            requested_as_of_date: entity.requested_as_of_date,
        }
    }
}
