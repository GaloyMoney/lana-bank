use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use crate::{
    eod_process::EodProcess,
    primitives::{EodProcessId, EodProcessStatus},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct PublicEodProcess {
    pub id: EodProcessId,
    pub date: chrono::NaiveDate,
    pub status: EodProcessStatus,
}

impl From<&EodProcess> for PublicEodProcess {
    fn from(entity: &EodProcess) -> Self {
        PublicEodProcess {
            id: entity.id,
            date: entity.date,
            status: entity.status(),
        }
    }
}
