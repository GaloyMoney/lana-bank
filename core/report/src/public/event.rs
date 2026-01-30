use obix::out::EphemeralEventType;
use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use super::PublicReportRun;

pub const REPORT_RUN_EVENT_TYPE: EphemeralEventType =
    EphemeralEventType::new("core.report.report-run");

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub enum CoreReportEvent {
    ReportRunCreated { entity: PublicReportRun },
    ReportRunStateUpdated { entity: PublicReportRun },
}
