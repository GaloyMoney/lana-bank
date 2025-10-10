use serde::{Deserialize, Serialize};

use crate::primitives::*;

#[derive(Debug, Serialize, Deserialize, strum::AsRefStr)]
#[serde(tag = "type")]
pub enum GovernanceEvent {
    ApprovalProcessConcluded {
        id: ApprovalProcessId,
        process_type: ApprovalProcessType,
        approved: bool,
        denied_reason: Option<String>,
        target_ref: String,
    },
}
