use serde::{Deserialize, Serialize};

use crate::primitives::*;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CoreReportEvent {
    ReportCreated { id: ReportId },
    ReportRunCreated { id: ReportRunId },
    ReportRunStateUpdated { id: ReportRunId },
}
