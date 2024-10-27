mod entity;
mod error;
mod repo;

use authz::PermissionCheck;

use error::DocumentError;
use repo::DocumentsRepo;

pub use entity::*;

use crate::{
    authorization::{Authorization, DocumentAction, Object},
    primitives::{CustomerId, DocumentId, Subject},
    storage::Storage,
};

#[derive(Clone)]
pub struct Documents {
    pool: sqlx::PgPool,
    authz: Authorization,
    storage: Storage,
    repo: DocumentsRepo,
}

impl Documents {
    pub fn new(pool: &sqlx::PgPool, storage: &Storage, authz: &Authorization) -> Self {
        Self {
            pool: pool.clone(),
            storage: storage.clone(),
            repo: DocumentsRepo::new(pool),
            authz: authz.clone(),
        }
    }

    pub async fn create(
        &self,
        sub: &Subject,
        content: Vec<u8>,
        customer_id: impl Into<CustomerId>,
        filename: String,
    ) -> Result<Document, DocumentError> {
        let audit_info = self
            .authz
            .enforce_permission(sub, Object::Document, DocumentAction::Create)
            .await?;

        let new_document = NewDocument::builder()
            .id(DocumentId::new())
            .customer_id(customer_id.into())
            .bucket(self.storage.bucket_name())
            .filename(filename)
            .audit_info(audit_info)
            .build()?;

        let mut tx = self.pool.begin().await?;
        let document = self.repo.create_in_tx(&mut tx, new_document).await?;

        self.storage
            .upload(content, &document.path_in_bucket, "application/pdf")
            .await?;

        tx.commit().await?;
        Ok(document)
    }

    pub async fn find_by_id(
        &self,
        sub: &Subject,
        id: DocumentId,
    ) -> Result<Option<Document>, DocumentError> {
        self.authz
            .enforce_permission(sub, Object::Document, DocumentAction::Read)
            .await?;

        match self.repo.find_by_id(id).await {
            Ok(document) => Ok(Some(document)),
            Err(DocumentError::NotFound) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub async fn list_by_customer_id(
        &self,
        sub: &Subject,
        customer_id: CustomerId,
    ) -> Result<Vec<Document>, DocumentError> {
        self.authz
            .enforce_permission(sub, Object::Document, DocumentAction::List)
            .await?;

        Ok(self
            .repo
            .list_for_customer_id_by_created_at(customer_id, Default::default())
            .await?
            .entities)
    }

    pub async fn generate_download_link(
        &self,
        sub: &Subject,
        document_id: DocumentId,
    ) -> Result<GeneratedDocumentDownloadLink, DocumentError> {
        let audit_info = self
            .authz
            .enforce_permission(sub, Object::Document, DocumentAction::GenerateDownloadLink)
            .await?;

        let mut document = self.repo.find_by_id(document_id).await?;

        let document_location = document.download_link_generated(audit_info);

        let link = self
            .storage
            .generate_download_link(document_location)
            .await?;

        let mut tx = self.pool.begin().await?;

        self.repo.update_in_tx(&mut tx, &mut document).await?;

        Ok(GeneratedDocumentDownloadLink { document_id, link })
    }

    pub async fn delete(
        &self,
        sub: &Subject,
        document_id: DocumentId,
    ) -> Result<(), DocumentError> {
        let audit_info = self
            .authz
            .enforce_permission(sub, Object::Document, DocumentAction::Delete)
            .await?;

        let mut db: sqlx::Transaction<'_, sqlx::Postgres> = self.pool.begin().await?;
        let mut document = self.repo.find_by_id(document_id).await?;

        let document_location = document.path_for_removal();
        self.storage.remove(document_location).await?;

        document.delete(audit_info);
        self.repo.delete_in_tx(&mut db, document).await?;
        db.commit().await?;

        Ok(())
    }

    pub async fn archive(
        &self,
        sub: &Subject,
        document_id: DocumentId,
    ) -> Result<Document, DocumentError> {
        let audit_info = self
            .authz
            .enforce_permission(sub, Object::Document, DocumentAction::Archive)
            .await?;

        let mut db: sqlx::Transaction<'_, sqlx::Postgres> = self.pool.begin().await?;
        let mut document = self.repo.find_by_id(document_id).await?;

        document.archive(audit_info);
        self.repo.update_in_tx(&mut db, &mut document).await?;
        db.commit().await?;

        Ok(document)
    }
}
