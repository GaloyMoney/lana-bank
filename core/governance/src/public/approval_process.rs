use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use crate::{
    approval_process::ApprovalProcess,
    primitives::{ApprovalProcessId, ApprovalProcessStatus, ApprovalProcessType},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct PublicApprovalProcess {
    pub id: ApprovalProcessId,
    pub process_type: ApprovalProcessType,
    pub status: ApprovalProcessStatus,
    pub target_ref: String,
}

impl From<&ApprovalProcess> for PublicApprovalProcess {
    fn from(process: &ApprovalProcess) -> Self {
        PublicApprovalProcess {
            id: process.id,
            process_type: process.process_type.clone(),
            status: process.status(),
            target_ref: process.target_ref().to_string(),
        }
    }
}
