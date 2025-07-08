use serde::{Deserialize, Serialize};

use crate::primitives::ReportId;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CoreReportEvent {
    ReportCreated {
        id: ReportId,
        name: String,
        date: chrono::DateTime<chrono::Utc>,
    },
}