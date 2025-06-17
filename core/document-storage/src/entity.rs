use chrono::{DateTime, Utc};
use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use audit::AuditInfo;
use es_entity::*;

use crate::primitives::*;

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "DocumentId")]
pub enum DocumentEvent {
    Initialized {
        id: DocumentId,
        #[cfg_attr(feature = "json-schema", schemars(skip))]
        audit_info: AuditInfo,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct Document {
    pub id: DocumentId,
    events: EntityEvents<DocumentEvent>,
}

impl core::fmt::Display for Document {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Document: {}", self.id)
    }
}

impl Document {
    pub fn created_at(&self) -> DateTime<Utc> {
        self.events
            .entity_first_persisted_at()
            .expect("entity_first_persisted_at not found")
    }
}

impl TryFromEvents<DocumentEvent> for Document {
    fn try_from_events(events: EntityEvents<DocumentEvent>) -> Result<Self, EsEntityError> {
        let mut builder = DocumentBuilder::default();

        for event in events.iter_all() {
            match event {
                DocumentEvent::Initialized { id, .. } => {
                    builder = builder.id(*id);
                }
            }
        }

        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
pub struct NewDocument {
    #[builder(setter(into))]
    pub(super) id: DocumentId,
    pub(super) audit_info: AuditInfo,
}

impl NewDocument {
    pub fn builder() -> NewDocumentBuilder {
        NewDocumentBuilder::default()
    }
}

impl IntoEvents<DocumentEvent> for NewDocument {
    fn into_events(self) -> EntityEvents<DocumentEvent> {
        EntityEvents::init(
            self.id,
            [DocumentEvent::Initialized {
                id: self.id,
                audit_info: self.audit_info,
            }],
        )
    }
}