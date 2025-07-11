use serde::{Deserialize, Serialize};

use crate::primitives::ReportId;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CoreReportEvent {
    ReportCreated {
        id: ReportId,
        path_in_bucket: String,
        date: chrono::DateTime<chrono::Utc>,
    },
}
