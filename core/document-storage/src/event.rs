use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use crate::entity::DocumentStatus;
use crate::primitives::*;

#[derive(Debug, Serialize, Deserialize, strum::AsRefStr)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type")]
pub enum CoreDocumentStorageEvent {
    DocumentStatusChanged {
        document_id: DocumentId,
        reference_id: ReferenceId,
        status: DocumentStatus,
        recorded_at: DateTime<Utc>,
    },
}
