use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use super::PublicApprovalProcess;

#[derive(Debug, Serialize, Deserialize, strum::AsRefStr)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type")]
pub enum GovernanceEvent {
    ApprovalProcessConcluded { entity: PublicApprovalProcess },
}
