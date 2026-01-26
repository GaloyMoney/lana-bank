use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use super::PublicReportRun;

#[derive(Debug, Serialize, Deserialize, strum::AsRefStr)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type")]
pub enum CoreReportEvent {
    ReportRunCreated { entity: PublicReportRun },
    ReportRunStateUpdated { entity: PublicReportRun },
}
