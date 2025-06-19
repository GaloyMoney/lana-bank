use chrono::{DateTime, Utc};
use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use audit::AuditInfo;
use es_entity::*;

use crate::primitives::*;

#[derive(Debug, Clone)]
pub struct GeneratedDocumentDownloadLink {
    pub document_id: DocumentId,
    pub link: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "graphql", derive(async_graphql::Enum))]
pub enum DocumentStatus {
    Active,
    Archived,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UploadStatus {
    Pending,
    Completed,
    Failed,
}

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "DocumentId")]
pub enum DocumentEvent {
    Initialized {
        id: DocumentId,
        document_type: DocumentType,
        reference_id: ReferenceId,
        sanitized_filename: String,
        original_filename: String,
        content_type: String,
        path_in_storage: String,
        storage_identifier: String,
        audit_info: AuditInfo,
    },
    FileUploaded {
        audit_info: AuditInfo,
    },
    UploadFailed {
        error: String,
    },
    DownloadLinkGenerated {
        audit_info: AuditInfo,
    },
    Deleted {
        audit_info: AuditInfo,
    },
    Archived {
        audit_info: AuditInfo,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct Document {
    pub id: DocumentId,
    pub filename: String,
    pub content_type: String,
    pub(super) path_in_storage: String,
    pub reference_id: ReferenceId,
    pub status: DocumentStatus,
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

    pub fn upload_file(&mut self, audit_info: AuditInfo) -> Idempotent<()> {
        idempotency_guard!(self.events.iter_all(), DocumentEvent::FileUploaded { .. });

        self.events.push(DocumentEvent::FileUploaded { audit_info });
        Idempotent::Executed(())
    }

    pub fn upload_failed(&mut self, error: String) {
        self.events.push(DocumentEvent::UploadFailed { error });
    }

    pub fn upload_status(&self) -> UploadStatus {
        for e in self.events.iter_all().rev() {
            match e {
                DocumentEvent::FileUploaded { .. } => return UploadStatus::Completed,
                DocumentEvent::UploadFailed { .. } => return UploadStatus::Failed,
                _ => {}
            }
        }
        UploadStatus::Pending
    }

    pub fn storage_path(&self) -> &str {
        &self.path_in_storage
    }

    pub fn download_link_generated(&mut self, audit_info: AuditInfo) -> &str {
        self.events
            .push(DocumentEvent::DownloadLinkGenerated { audit_info });
        &self.path_in_storage
    }

    pub fn path_for_removal(&self) -> &str {
        &self.path_in_storage
    }

    pub fn delete(&mut self, audit_info: AuditInfo) -> Idempotent<()> {
        idempotency_guard!(self.events.iter_all(), DocumentEvent::Deleted { .. });

        self.events.push(DocumentEvent::Deleted { audit_info });
        Idempotent::Executed(())
    }

    pub fn archive(&mut self, audit_info: AuditInfo) -> Idempotent<()> {
        idempotency_guard!(self.events.iter_all(), DocumentEvent::Archived { .. });

        self.events.push(DocumentEvent::Archived { audit_info });
        self.status = DocumentStatus::Archived;
        Idempotent::Executed(())
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
                    reference_id,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .filename(sanitized_filename.clone())
                        .content_type(content_type.clone())
                        .path_in_storage(path_in_storage.clone())
                        .reference_id(*reference_id)
                        .status(DocumentStatus::Active);
                }
                DocumentEvent::FileUploaded { .. } => {
                    // FileUploaded event doesn't modify any fields now
                }
                DocumentEvent::UploadFailed { .. } => {
                    // UploadFailed event doesn't modify any fields
                }
                DocumentEvent::DownloadLinkGenerated { .. } => {
                    // DownloadLinkGenerated event doesn't modify any fields
                }
                DocumentEvent::Deleted { .. } => {
                    // Deleted event doesn't modify any fields (soft delete)
                }
                DocumentEvent::Archived { .. } => {
                    builder = builder.status(DocumentStatus::Archived);
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
    #[builder(setter(into))]
    document_type: DocumentType,
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
    #[builder(setter(into))]
    pub(super) reference_id: ReferenceId,
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
                document_type: self.document_type,
                audit_info: self.audit_info,
                sanitized_filename: self.sanitized_filename,
                original_filename: self.filename,
                content_type: self.content_type,
                path_in_storage: self.path_in_storage,
                storage_identifier: self.storage_identifier,
                reference_id: self.reference_id,
            }],
        )
    }
}
