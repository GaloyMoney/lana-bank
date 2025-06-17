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
        audit_info: AuditInfo,
        sanitized_filename: String,
        original_filename: String,
        content_type: String,
        path_in_storage: String,
        storage_identifier: String,
    },
    FileUploaded {
        audit_info: AuditInfo,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct Document {
    pub id: DocumentId,
    pub filename: String,
    pub(super) content_type: String,
    pub(super) path_in_storage: String,
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

    pub fn upload_file(&mut self, audit_info: AuditInfo) {
        self.events.push(DocumentEvent::FileUploaded { audit_info });
    }
}

impl TryFromEvents<DocumentEvent> for Document {
    fn try_from_events(events: EntityEvents<DocumentEvent>) -> Result<Self, EsEntityError> {
        let mut builder = DocumentBuilder::default();

        for event in events.iter_all() {
            match event {
                DocumentEvent::Initialized {
                    id,
                    sanitized_filename,
                    content_type,
                    path_in_storage,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .filename(sanitized_filename.clone())
                        .content_type(content_type.clone())
                        .path_in_storage(path_in_storage.clone());
                }
                DocumentEvent::FileUploaded { .. } => {
                    // FileUploaded event doesn't modify any fields now
                }
            }
        }

        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct NewDocument {
    #[builder(setter(into))]
    pub(super) id: DocumentId,
    #[builder(setter(custom))]
    filename: String,
    #[builder(private)]
    sanitized_filename: String,
    #[builder(setter(into))]
    pub(super) content_type: String,
    #[builder(setter(into))]
    pub(super) path_in_storage: String,
    #[builder(setter(into))]
    pub(super) storage_identifier: String,
    pub(super) audit_info: AuditInfo,
}

impl NewDocumentBuilder {
    pub fn filename<T: Into<String>>(mut self, filename: T) -> Self {
        let filename = filename.into();
        let sanitized = filename
            .trim()
            .replace(|c: char| !c.is_alphanumeric() && c != '-', "-");
        self.filename = Some(filename);
        self.sanitized_filename = Some(sanitized);
        self
    }
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
                sanitized_filename: self.sanitized_filename,
                original_filename: self.filename,
                content_type: self.content_type,
                path_in_storage: self.path_in_storage,
                storage_identifier: self.storage_identifier,
            }],
        )
    }
}
