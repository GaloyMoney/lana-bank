use serde::{Deserialize, Serialize};

use crate::{
    eod_process::EodProcess,
    primitives::{EodProcessId, EodProcessStatus},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
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
