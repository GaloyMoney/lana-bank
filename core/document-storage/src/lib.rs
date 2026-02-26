#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod entity;
pub mod error;
mod primitives;
mod repo;

use cloud_storage::Storage;
use es_entity::{ListDirection, clock::ClockHandle};
use std::collections::HashMap;
use tracing::instrument;
use tracing_macros::record_error_severity;

pub use entity::{Document, DocumentStatus, GeneratedDocumentDownloadLink, NewDocument};
use error::*;
pub use primitives::*;
pub use repo::{DocumentRepo, document_cursor::DocumentsByCreatedAtCursor};

/// Returns the file extension (including the dot) for a given content type
fn extension_for_content_type(content_type: &str) -> &'static str {
    match content_type {
        "application/pdf" => ".pdf",
        "text/csv" => ".csv",
        _ => "",
    }
}

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
    pub fn new(pool: &sqlx::PgPool, storage: &Storage, clock: ClockHandle) -> Self {
        let repo = DocumentRepo::new(pool, clock);
        Self {
            repo,
            storage: storage.clone(),
        }
    }

    pub async fn begin_op(&self) -> Result<es_entity::DbOp<'_>, sqlx::Error> {
        self.repo.begin_op().await
    }

    #[record_error_severity]
    #[instrument(name = "document_storage.create_in_op", skip(self, db))]
    pub async fn create_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        filename: impl Into<String> + std::fmt::Debug,
        content_type: impl Into<String> + std::fmt::Debug,
        reference_id: impl Into<ReferenceId> + std::fmt::Debug,
        document_type: impl Into<DocumentType> + std::fmt::Debug,
    ) -> Result<Document, DocumentStorageError> {
        let document_id = DocumentId::new();
        let document_type = document_type.into();
        let content_type: String = content_type.into();
        let extension = extension_for_content_type(&content_type);
        let path_in_storage = format!("documents/{document_type}/{document_id}{extension}");
        let storage_identifier = self.storage.identifier();

        let new_document = NewDocument::builder()
            .id(document_id)
            .document_type(document_type)
            .filename(filename)
            .content_type(content_type)
            .path_in_storage(path_in_storage)
            .storage_identifier(storage_identifier)
            .reference_id(reference_id)
            .build()
            .expect("Could not build document");

        let document = self.repo.create_in_op(db, new_document).await?;
        Ok(document)
    }

    #[record_error_severity]
    #[instrument(
        name = "document_storage.upload_in_op",
        skip(self, db, content, document)
    )]
    pub async fn upload_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        content: Vec<u8>,
        document: &mut Document,
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

    #[record_error_severity]
    #[instrument(name = "document_storage.upload", skip(self, content, document))]
    pub async fn upload(
        &self,
        content: Vec<u8>,
        document: &mut Document,
    ) -> Result<(), DocumentStorageError> {
        let mut db = self.begin_op().await?;
        self.upload_in_op(&mut db, content, document).await?;
        db.commit().await?;
        Ok(())
    }

    #[record_error_severity]
    #[instrument(name = "document_storage.create_and_upload", skip(self, content))]
    pub async fn create_and_upload(
        &self,
        content: Vec<u8>,
        filename: impl Into<String> + std::fmt::Debug,
        content_type: impl Into<String> + std::fmt::Debug,
        reference_id: impl Into<ReferenceId> + std::fmt::Debug,
        document_type: impl Into<DocumentType> + std::fmt::Debug,
    ) -> Result<Document, DocumentStorageError> {
        let document_id = DocumentId::new();
        let document_type = document_type.into();
        let content_type: String = content_type.into();
        let extension = extension_for_content_type(&content_type);
        let path_in_storage = format!("documents/{document_type}/{document_id}{extension}");
        let storage_identifier = self.storage.identifier();

        let new_document = NewDocument::builder()
            .id(document_id)
            .document_type(document_type)
            .filename(filename)
            .content_type(content_type)
            .path_in_storage(path_in_storage)
            .storage_identifier(storage_identifier)
            .reference_id(reference_id)
            .build()
            .expect("Could not build document");

        let mut db = self.begin_op().await?;
        let mut document = self.repo.create_in_op(&mut db, new_document).await?;

        self.upload_in_op(&mut db, content, &mut document).await?;

        db.commit().await?;

        Ok(document)
    }

    #[record_error_severity]
    #[instrument(name = "document_storage.find_by_id", skip(self))]
    pub async fn find_by_id(
        &self,
        id: impl Into<DocumentId> + std::fmt::Debug + Copy,
    ) -> Result<Document, DocumentStorageError> {
        self.repo.find_by_id(id.into()).await
    }

    #[record_error_severity]
    #[instrument(name = "document_storage.list_for_reference_id", skip(self))]
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

    #[record_error_severity]
    #[instrument(name = "document_storage.list_for_reference_id_paginated", skip(self))]
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

    #[record_error_severity]
    #[instrument(name = "document_storage.generate_download_link", skip(self))]
    pub async fn generate_download_link(
        &self,
        document_id: impl Into<DocumentId> + std::fmt::Debug,
    ) -> Result<GeneratedDocumentDownloadLink, DocumentStorageError> {
        let document_id = document_id.into();

        let mut document = self.repo.find_by_id(document_id).await?;

        let _ = document.download_link_generated();
        let document_location = document.storage_path();

        let link = self
            .storage
            .generate_download_link(cloud_storage::LocationInStorage {
                path: document_location,
            })
            .await?;

        self.repo.update(&mut document).await?;

        Ok(GeneratedDocumentDownloadLink { document_id, link })
    }

    #[record_error_severity]
    #[instrument(name = "document_storage.delete", skip(self))]
    pub async fn delete(
        &self,
        document_id: impl Into<DocumentId> + std::fmt::Debug + Copy,
    ) -> Result<(), DocumentStorageError> {
        let mut db = self.begin_op().await?;
        let mut document = self
            .repo
            .find_by_id_in_op(&mut db, document_id.into())
            .await?;

        let document_location = document.path_for_removal();
        self.storage
            .remove(cloud_storage::LocationInStorage {
                path: document_location,
            })
            .await?;

        if document.delete().did_execute() {
            self.repo.delete_in_op(&mut db, document).await?;
            db.commit().await?;
        }

        Ok(())
    }

    #[record_error_severity]
    #[instrument(name = "document_storage.archive", skip(self))]
    pub async fn archive(
        &self,
        document_id: impl Into<DocumentId> + std::fmt::Debug + Copy,
    ) -> Result<Document, DocumentStorageError> {
        let mut document = self.repo.find_by_id(document_id.into()).await?;

        if document.archive().did_execute() {
            self.repo.update(&mut document).await?;
        }

        Ok(document)
    }

    #[record_error_severity]
    #[instrument(name = "document_storage.find_all", skip(self))]
    pub async fn find_all<T: From<Document>>(
        &self,
        ids: &[DocumentId],
    ) -> Result<HashMap<DocumentId, T>, DocumentStorageError> {
        self.repo.find_all(ids).await
    }
}
