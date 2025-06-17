#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod entity;
pub mod error;
mod event;
mod primitives;
mod publisher;
mod repo;

use audit::AuditSvc;
use authz::PermissionCheck;
use outbox::{Outbox, OutboxEventMarker};
use tracing::instrument;

pub use entity::{Document, NewDocument};
use error::*;
pub use event::*;
pub use primitives::*;
pub use repo::DocumentRepo;

#[cfg(feature = "json-schema")]
pub mod event_schema {
    pub use crate::entity::DocumentEvent;
}

use publisher::*;

pub struct DocumentStorage<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreDocumentStorageEvent>,
{
    authz: Perms,
    outbox: Outbox<E>,
    repo: DocumentRepo<E>,
}

impl<Perms, E> Clone for DocumentStorage<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreDocumentStorageEvent>,
{
    fn clone(&self) -> Self {
        Self {
            authz: self.authz.clone(),
            outbox: self.outbox.clone(),
            repo: self.repo.clone(),
        }
    }
}

impl<Perms, E> DocumentStorage<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreDocumentStorageAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<DocumentStorageObject>,
    E: OutboxEventMarker<CoreDocumentStorageEvent>,
{
    pub fn new(pool: &sqlx::PgPool, authz: &Perms, outbox: &Outbox<E>) -> Self {
        let publisher = DocumentStoragePublisher::new(outbox);
        let repo = DocumentRepo::new(pool, &publisher);
        Self {
            repo,
            authz: authz.clone(),
            outbox: outbox.clone(),
        }
    }

    #[instrument(name = "document_storage.create_and_upload", skip(self, content), err)]
    pub async fn create_and_upload(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        content: Vec<u8>,
        filename: impl Into<String> + std::fmt::Debug,
        content_type: impl Into<String> + std::fmt::Debug,
    ) -> Result<Document, DocumentStorageError> {
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                DocumentStorageObject::all_documents(),
                CoreDocumentStorageAction::DOCUMENT_CREATE,
            )
            .await?;

        let new_document = NewDocument::builder()
            .id(DocumentId::new())
            .audit_info(audit_info.clone())
            .build()
            .expect("Could not build document");

        let mut db = self.repo.begin_op().await?;
        let mut document = self.repo.create_in_op(&mut db, new_document).await?;

        // Now upload the file (using the same audit_info since upload is implicit in create)
        document.upload_file(
            filename.into(),
            content_type.into(),
            audit_info,
        );

        self.repo.update_in_op(&mut db, &mut document).await?;

        // TODO: Actually upload the content to storage
        // For now we just simulate the upload process
        let _ = content; // Suppress unused warning

        db.commit().await?;

        Ok(document)
    }
}
