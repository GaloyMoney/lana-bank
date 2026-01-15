use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use crate::primitives::*;

#[derive(Debug, Serialize, Deserialize, strum::AsRefStr)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type")]
pub enum CoreReportEvent {
    ReportCreated { id: ReportId },
    ReportRunCreated { id: ReportRunId },
    ReportRunStateUpdated { id: ReportRunId },
}
