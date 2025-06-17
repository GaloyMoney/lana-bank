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
use cloud_storage::Storage;
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
    storage: Storage,
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
            storage: self.storage.clone(),
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
    pub fn new(pool: &sqlx::PgPool, authz: &Perms, outbox: &Outbox<E>, storage: &Storage) -> Self {
        let publisher = DocumentStoragePublisher::new(outbox);
        let repo = DocumentRepo::new(pool, &publisher);
        Self {
            repo,
            authz: authz.clone(),
            outbox: outbox.clone(),
            storage: storage.clone(),
        }
    }

    #[instrument(name = "document_storage.create_and_upload", skip(self, content), err)]
    pub async fn create_and_upload(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        content: Vec<u8>,
        filename: impl Into<String> + std::fmt::Debug,
        content_type: impl Into<String> + std::fmt::Debug,
        owner_id: impl Into<Option<DocumentOwnerId>> + std::fmt::Debug,
    ) -> Result<Document, DocumentStorageError> {
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                DocumentStorageObject::all_documents(),
                CoreDocumentStorageAction::DOCUMENT_CREATE,
            )
            .await?;

        let filename_str = filename.into();
        let content_type_str = content_type.into();
        let owner_id_opt = owner_id.into();
        let document_id = DocumentId::new();
        let path_in_storage = format!("documents/{}", document_id);
        let storage_identifier = self.storage.storage_identifier();

        let new_document = NewDocument::builder()
            .id(document_id)
            .filename(filename_str)
            .content_type(content_type_str.clone())
            .path_in_storage(path_in_storage)
            .storage_identifier(storage_identifier)
            .owner_id(owner_id_opt)
            .audit_info(audit_info.clone())
            .build()
            .expect("Could not build document");

        let mut db = self.repo.begin_op().await?;
        let mut document = self.repo.create_in_op(&mut db, new_document).await?;

        self.storage
            .upload(content, &document.path_in_storage, &document.content_type)
            .await?;

        // Now record the upload in the entity
        document.upload_file(audit_info);

        self.repo.update_in_op(&mut db, &mut document).await?;
        db.commit().await?;

        Ok(document)
    }
}
