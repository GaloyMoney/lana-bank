use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use crate::primitives::*;

#[derive(Debug, Serialize, Deserialize, strum::AsRefStr)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
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
