use serde::{Deserialize, Serialize};

use crate::primitives::*;

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
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
