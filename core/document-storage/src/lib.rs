#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod entity;
pub mod error;
mod primitives;
mod repo;

use audit::AuditInfo;
use cloud_storage::Storage;
use es_entity::ListDirection;
use std::collections::HashMap;
use tracing::instrument;

pub use entity::{
    Document, DocumentStatus, GeneratedDocumentDownloadLink, NewDocument, UploadStatus,
};
use error::*;
pub use primitives::*;
pub use repo::{document_cursor::DocumentsByCreatedAtCursor, DocumentRepo};

#[cfg(feature = "json-schema")]
pub mod event_schema {
    pub use crate::entity::DocumentEvent;
}

pub struct DocumentStorage {
    repo: DocumentRepo,
    storage: Storage,
}

impl Clone for DocumentStorage {
    fn clone(&self) -> Self {
        Self {
            repo: self.repo.clone(),
            storage: self.storage.clone(),
        }
    }
}

impl DocumentStorage {
    pub fn new(pool: &sqlx::PgPool, storage: &Storage) -> Self {
        let repo = DocumentRepo::new(pool);
        Self {
            repo,
            storage: storage.clone(),
        }
    }

    pub async fn begin_op(&self) -> Result<es_entity::DbOp<'_>, sqlx::Error> {
        self.repo.begin_op().await
    }

    #[instrument(name = "document_storage.create_in_op", skip(self, db), err)]
    pub async fn create_in_op(
        &self,
        audit_info: AuditInfo,
        filename: impl Into<String> + std::fmt::Debug,
        content_type: impl Into<String> + std::fmt::Debug,
        reference_id: impl Into<ReferenceId> + std::fmt::Debug,
        document_type: impl Into<DocumentType> + std::fmt::Debug,
        db: &mut es_entity::DbOp<'_>,
    ) -> Result<Document, DocumentStorageError> {
        let document_id = DocumentId::new();
        let document_type = document_type.into();
        let path_in_storage = format!("documents/{}/{}", document_type, document_id);
        let storage_identifier = self.storage.identifier();

        let new_document = NewDocument::builder()
            .id(document_id)
            .document_type(document_type)
            .filename(filename)
            .content_type(content_type)
            .path_in_storage(path_in_storage)
            .storage_identifier(storage_identifier)
            .reference_id(reference_id)
            .audit_info(audit_info)
            .build()
            .expect("Could not build document");

        let document = self.repo.create_in_op(db, new_document).await?;
        Ok(document)
    }

    #[instrument(
        name = "document_storage.upload_in_op",
        skip(self, content, document, db),
        err
    )]
    pub async fn upload_in_op(
        &self,
        content: Vec<u8>,
        document: &mut Document,
        db: &mut es_entity::DbOp<'_>,
    ) -> Result<(), DocumentStorageError> {
        self.storage
            .upload(content, &document.path_in_storage, &document.content_type)
            .await?;

        // Now record the upload in the entity
        if document.upload_file().did_execute() {
            self.repo.update_in_op(db, document).await?;
        }

        Ok(())
    }

    #[instrument(name = "document_storage.upload", skip(self, content, document), err)]
    pub async fn upload(
        &self,
        content: Vec<u8>,
        document: &mut Document,
    ) -> Result<(), DocumentStorageError> {
        let mut db = self.repo.begin_op().await?;
        self.upload_in_op(content, document, &mut db).await?;
        db.commit().await?;
        Ok(())
    }

    #[instrument(name = "document_storage.create_and_upload", skip(self, content), err)]
    pub async fn create_and_upload(
        &self,
        audit_info: AuditInfo,
        content: Vec<u8>,
        filename: impl Into<String> + std::fmt::Debug,
        content_type: impl Into<String> + std::fmt::Debug,
        reference_id: impl Into<ReferenceId> + std::fmt::Debug,
        document_type: impl Into<DocumentType> + std::fmt::Debug,
    ) -> Result<Document, DocumentStorageError> {
        let document_id = DocumentId::new();
        let document_type = document_type.into();
        let path_in_storage = format!("documents/{}/{}", document_type, document_id);
        let storage_identifier = self.storage.identifier();

        let new_document = NewDocument::builder()
            .id(document_id)
            .document_type(document_type)
            .filename(filename)
            .content_type(content_type)
            .path_in_storage(path_in_storage)
            .storage_identifier(storage_identifier)
            .reference_id(reference_id)
            .audit_info(audit_info.clone())
            .build()
            .expect("Could not build document");

        let mut db = self.repo.begin_op().await?;
        let mut document = self.repo.create_in_op(&mut db, new_document).await?;

        self.upload_in_op(content, &mut document, &mut db).await?;

        db.commit().await?;

        Ok(document)
    }

    #[instrument(name = "document_storage.find_by_id", skip(self), err)]
    pub async fn find_by_id(
        &self,
        id: impl Into<DocumentId> + std::fmt::Debug + Copy,
    ) -> Result<Option<Document>, DocumentStorageError> {
        match self.repo.find_by_id(id.into()).await {
            Ok(document) => Ok(Some(document)),
            Err(e) if e.was_not_found() => Ok(None),
            Err(e) => Err(e),
        }
    }

    #[instrument(name = "document_storage.list_for_reference_id", skip(self), err)]
    pub async fn list_for_reference_id(
        &self,
        reference_id: impl Into<ReferenceId> + std::fmt::Debug,
    ) -> Result<Vec<Document>, DocumentStorageError> {
        Ok(self
            .repo
            .list_for_reference_id_by_created_at(
                reference_id.into(),
                Default::default(),
                ListDirection::Descending,
            )
            .await?
            .entities)
    }

    #[instrument(
        name = "document_storage.list_for_reference_id_paginated",
        skip(self),
        err
    )]
    pub async fn list_for_reference_id_paginated(
        &self,
        reference_id: impl Into<ReferenceId> + std::fmt::Debug,
        query: es_entity::PaginatedQueryArgs<DocumentsByCreatedAtCursor>,
    ) -> Result<
        es_entity::PaginatedQueryRet<Document, DocumentsByCreatedAtCursor>,
        DocumentStorageError,
    > {
        self.repo
            .list_for_reference_id_by_created_at(
                reference_id.into(),
                query,
                ListDirection::Descending,
            )
            .await
    }

    #[instrument(name = "document_storage.generate_download_link", skip(self), err)]
    pub async fn generate_download_link(
        &self,
        audit_info: AuditInfo,
        document_id: impl Into<DocumentId> + std::fmt::Debug,
    ) -> Result<GeneratedDocumentDownloadLink, DocumentStorageError> {
        let document_id = document_id.into();

        let mut document = self.repo.find_by_id(document_id).await?;

        let document_location = document.download_link_generated(audit_info);

        let link = self
            .storage
            .generate_download_link(cloud_storage::LocationInStorage {
                path: document_location,
            })
            .await?;

        self.repo.update(&mut document).await?;

        Ok(GeneratedDocumentDownloadLink { document_id, link })
    }

    #[instrument(name = "document_storage.delete", skip(self), err)]
    pub async fn delete(
        &self,
        audit_info: AuditInfo,
        document_id: impl Into<DocumentId> + std::fmt::Debug + Copy,
    ) -> Result<(), DocumentStorageError> {
        let mut db = self.repo.begin_op().await?;
        let mut document = self.repo.find_by_id(document_id.into()).await?;

        let document_location = document.path_for_removal();
        self.storage
            .remove(cloud_storage::LocationInStorage {
                path: document_location,
            })
            .await?;

        if document.delete(audit_info).did_execute() {
            self.repo.delete_in_op(&mut db, document).await?;
            db.commit().await?;
        }

        Ok(())
    }

    #[instrument(name = "document_storage.archive", skip(self), err)]
    pub async fn archive(
        &self,
        audit_info: AuditInfo,
        document_id: impl Into<DocumentId> + std::fmt::Debug + Copy,
    ) -> Result<Document, DocumentStorageError> {
        let mut document = self.repo.find_by_id(document_id.into()).await?;

        if document.archive(audit_info).did_execute() {
            self.repo.update(&mut document).await?;
        }

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
