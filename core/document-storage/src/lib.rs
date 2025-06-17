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
use es_entity::ListDirection;
use outbox::{Outbox, OutboxEventMarker};
use std::collections::HashMap;
use tracing::instrument;

pub use entity::{Document, GeneratedDocumentDownloadLink, NewDocument};
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

    #[instrument(name = "document_storage.find_by_id", skip(self), err)]
    pub async fn find_by_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        id: impl Into<DocumentId> + std::fmt::Debug + Copy,
    ) -> Result<Option<Document>, DocumentStorageError> {
        self.authz
            .enforce_permission(
                sub,
                DocumentStorageObject::document(id.into()),
                CoreDocumentStorageAction::DOCUMENT_READ,
            )
            .await?;

        match self.repo.find_by_id(id.into()).await {
            Ok(document) => Ok(Some(document)),
            Err(e) if e.was_not_found() => Ok(None),
            Err(e) => Err(e),
        }
    }

    #[instrument(name = "document_storage.list_for_owner_id", skip(self), err)]
    pub async fn list_for_owner_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        owner_id: DocumentOwnerId,
    ) -> Result<Vec<Document>, DocumentStorageError> {
        self.authz
            .enforce_permission(
                sub,
                DocumentStorageObject::all_documents(),
                CoreDocumentStorageAction::DOCUMENT_LIST,
            )
            .await?;

        Ok(self
            .repo
            .list_for_owner_id_by_created_at(
                Some(owner_id),
                Default::default(),
                ListDirection::Descending,
            )
            .await?
            .entities)
    }

    #[instrument(name = "document_storage.generate_download_link", skip(self), err)]
    pub async fn generate_download_link(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        document_id: DocumentId,
    ) -> Result<GeneratedDocumentDownloadLink, DocumentStorageError> {
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                DocumentStorageObject::document(document_id),
                CoreDocumentStorageAction::DOCUMENT_READ,
            )
            .await?;

        let mut document = self.repo.find_by_id(document_id).await?;

        let document_location = document.download_link_generated(audit_info);

        let link = self
            .storage
            .generate_download_link(cloud_storage::LocationInStorage {
                path_in_bucket: document_location,
            })
            .await?;

        self.repo.update(&mut document).await?;

        Ok(GeneratedDocumentDownloadLink { document_id, link })
    }

    #[instrument(name = "document_storage.delete", skip(self), err)]
    pub async fn delete(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        document_id: impl Into<DocumentId> + std::fmt::Debug + Copy,
    ) -> Result<(), DocumentStorageError> {
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                DocumentStorageObject::document(document_id.into()),
                CoreDocumentStorageAction::DOCUMENT_READ,
            )
            .await?;

        let mut db = self.repo.begin_op().await?;
        let mut document = self.repo.find_by_id(document_id.into()).await?;

        let document_location = document.path_for_removal();
        self.storage
            .remove(cloud_storage::LocationInStorage {
                path_in_bucket: document_location,
            })
            .await?;

        document.delete(audit_info);
        self.repo.delete_in_op(&mut db, document).await?;
        db.commit().await?;

        Ok(())
    }

    #[instrument(name = "document_storage.archive", skip(self), err)]
    pub async fn archive(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        document_id: impl Into<DocumentId> + std::fmt::Debug + Copy,
    ) -> Result<Document, DocumentStorageError> {
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                DocumentStorageObject::document(document_id.into()),
                CoreDocumentStorageAction::DOCUMENT_READ,
            )
            .await?;

        let mut document = self.repo.find_by_id(document_id.into()).await?;

        document.archive(audit_info);
        self.repo.update(&mut document).await?;

        Ok(document)
    }

    #[instrument(name = "document_storage.find_all", skip(self), err)]
    pub async fn find_all<T: From<Document>>(
        &self,
        ids: &[DocumentId],
    ) -> Result<HashMap<DocumentId, T>, DocumentStorageError> {
        self.repo.find_all(ids).await
    }
}
