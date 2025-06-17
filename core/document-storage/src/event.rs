use serde::{Deserialize, Serialize};

use crate::primitives::DocumentId;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CoreDocumentStorageEvent {
    DocumentCreated { id: DocumentId },
}
