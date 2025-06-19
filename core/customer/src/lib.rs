#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod entity;
pub mod error;
mod event;
mod primitives;
mod publisher;
mod repo;

use std::collections::HashMap;
use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;
use document_storage::{
    Document, DocumentId, DocumentStorage, DocumentType, GeneratedDocumentDownloadLink,
};
use outbox::{Outbox, OutboxEventMarker};

pub use entity::Customer;
use entity::*;
use error::*;
pub use event::*;
pub use primitives::*;
pub use repo::{customer_cursor::*, CustomerRepo, CustomersSortBy, FindManyCustomers, Sort};

#[cfg(feature = "json-schema")]
pub mod event_schema {
    pub use crate::entity::CustomerEvent;
}

use publisher::*;

pub struct Customers<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    authz: Perms,
    outbox: Outbox<E>,
    repo: CustomerRepo<E>,
    document_storage: DocumentStorage,
}

impl<Perms, E> Clone for Customers<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    fn clone(&self) -> Self {
        Self {
            authz: self.authz.clone(),
            outbox: self.outbox.clone(),
            repo: self.repo.clone(),
            document_storage: self.document_storage.clone(),
        }
    }
}

impl<Perms, E> Customers<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCustomerAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CustomerObject>,
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    pub fn new(
        pool: &sqlx::PgPool,
        authz: &Perms,
        outbox: &Outbox<E>,
        document_storage: DocumentStorage,
    ) -> Self {
        let publisher = CustomerPublisher::new(outbox);
        let repo = CustomerRepo::new(pool, &publisher);
        Self {
            repo,
            authz: authz.clone(),
            outbox: outbox.clone(),
            document_storage,
        }
    }

    pub async fn subject_can_create_customer(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        enforce: bool,
    ) -> Result<Option<AuditInfo>, CustomerError> {
        Ok(self
            .authz
            .evaluate_permission(
                sub,
                CustomerObject::all_customers(),
                CoreCustomerAction::CUSTOMER_CREATE,
                enforce,
            )
            .await?)
    }

    #[instrument(name = "customer.create_customer", skip(self), err)]
    pub async fn create(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        email: impl Into<String> + std::fmt::Debug,
        telegram_id: impl Into<String> + std::fmt::Debug,
        customer_type: impl Into<CustomerType> + std::fmt::Debug,
    ) -> Result<Customer, CustomerError> {
        let audit_info = self
            .subject_can_create_customer(sub, true)
            .await?
            .expect("audit info missing");

        let email = email.into();
        let telegram_id = telegram_id.into();

        let new_customer = NewCustomer::builder()
            .id(CustomerId::new())
            .email(email.clone())
            .telegram_id(telegram_id)
            .customer_type(customer_type)
            .audit_info(audit_info)
            .build()
            .expect("Could not build customer");

        let mut db = self.repo.begin_op().await?;
        let customer = self.repo.create_in_op(&mut db, new_customer).await?;

        db.commit().await?;

        Ok(customer)
    }

    #[instrument(name = "customer.find_for_subject", skip(self))]
    pub async fn find_for_subject(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
    ) -> Result<Customer, CustomerError>
    where
        CustomerId: for<'a> TryFrom<&'a <<Perms as PermissionCheck>::Audit as AuditSvc>::Subject>,
    {
        let id = CustomerId::try_from(sub).map_err(|_| CustomerError::SubjectIsNotCustomer)?;
        self.repo.find_by_id(id).await
    }

    #[instrument(name = "customer.find_by_id", skip(self), err)]
    pub async fn find_by_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        id: impl Into<CustomerId> + std::fmt::Debug,
    ) -> Result<Option<Customer>, CustomerError> {
        let id = id.into();
        self.authz
            .enforce_permission(
                sub,
                CustomerObject::customer(id),
                CoreCustomerAction::CUSTOMER_READ,
            )
            .await?;

        match self.repo.find_by_id(id).await {
            Ok(customer) => Ok(Some(customer)),
            Err(e) if e.was_not_found() => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub async fn find_by_id_without_audit(
        &self,
        id: impl Into<CustomerId> + std::fmt::Debug,
    ) -> Result<Customer, CustomerError> {
        self.repo.find_by_id(id.into()).await
    }

    #[instrument(name = "customer.find_by_email", skip(self), err)]
    pub async fn find_by_email(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        email: String,
    ) -> Result<Option<Customer>, CustomerError> {
        self.authz
            .enforce_permission(
                sub,
                CustomerObject::all_customers(),
                CoreCustomerAction::CUSTOMER_READ,
            )
            .await?;

        match self.repo.find_by_email(email).await {
            Ok(customer) => Ok(Some(customer)),
            Err(e) if e.was_not_found() => Ok(None),
            Err(e) => Err(e),
        }
    }

    #[instrument(name = "customer.list", skip(self), err)]
    pub async fn list(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        query: es_entity::PaginatedQueryArgs<CustomersCursor>,
        filter: FindManyCustomers,
        sort: impl Into<Sort<CustomersSortBy>> + std::fmt::Debug,
    ) -> Result<es_entity::PaginatedQueryRet<Customer, CustomersCursor>, CustomerError> {
        self.authz
            .enforce_permission(
                sub,
                CustomerObject::all_customers(),
                CoreCustomerAction::CUSTOMER_LIST,
            )
            .await?;
        self.repo.find_many(filter, sort.into(), query).await
    }

    #[instrument(
        name = "customer.update_authentication_id_for_customer",
        skip(self, authentication_id)
    )]
    pub async fn update_authentication_id_for_customer(
        &self,
        customer_id: CustomerId,
        authentication_id: AuthenticationId,
    ) -> Result<Customer, CustomerError> {
        self.authz
            .audit()
            .record_system_entry(
                CustomerObject::customer(customer_id),
                CoreCustomerAction::CUSTOMER_UPDATE_AUTHENTICATION_ID,
            )
            .await?;

        let mut customer = self.repo.find_by_id(customer_id).await?;
        if customer
            .update_authentication_id(authentication_id)
            .did_execute()
        {
            self.repo.update(&mut customer).await?;
        }
        Ok(customer)
    }

    #[instrument(
        name = "customer.find_by_authentication_id",
        skip(self, authentication_id)
    )]
    pub async fn find_by_authentication_id(
        &self,
        authentication_id: AuthenticationId,
    ) -> Result<Customer, CustomerError> {
        self.repo
            .find_by_authentication_id(Some(authentication_id))
            .await
    }

    #[instrument(name = "customer.start_kyc", skip(self, db), err)]
    pub async fn start_kyc(
        &self,
        db: &mut es_entity::DbOp<'_>,
        customer_id: CustomerId,
        applicant_id: String,
    ) -> Result<Customer, CustomerError> {
        let mut customer = self.repo.find_by_id(customer_id).await?;

        let audit_info = self
            .authz
            .audit()
            .record_system_entry_in_tx(
                db.tx(),
                CustomerObject::customer(customer_id),
                CoreCustomerAction::CUSTOMER_START_KYC,
            )
            .await?;

        customer.start_kyc(applicant_id, audit_info);

        self.repo.update_in_op(db, &mut customer).await?;

        Ok(customer)
    }

    #[instrument(name = "customer.approve_kyc", skip(self, db), err)]
    pub async fn approve_kyc(
        &self,
        db: &mut es_entity::DbOp<'_>,
        customer_id: CustomerId,
        applicant_id: String,
    ) -> Result<Customer, CustomerError> {
        let mut customer = self.repo.find_by_id(customer_id).await?;

        let audit_info = self
            .authz
            .audit()
            .record_system_entry_in_tx(
                db.tx(),
                CustomerObject::customer(customer_id),
                CoreCustomerAction::CUSTOMER_APPROVE_KYC,
            )
            .await?;

        if customer
            // TODO: this is wrong, we should pass the SumSub verification level
            // because we also have KYB approval
            .approve_kyc(KycLevel::Basic, applicant_id, audit_info)
            .did_execute()
        {
            self.repo.update_in_op(db, &mut customer).await?;
        }

        Ok(customer)
    }

    #[instrument(name = "customer.decline_kyc", skip(self, db), err)]
    pub async fn decline_kyc(
        &self,
        db: &mut es_entity::DbOp<'_>,
        customer_id: CustomerId,
        applicant_id: String,
    ) -> Result<Customer, CustomerError> {
        let mut customer = self.repo.find_by_id(customer_id).await?;

        let audit_info = self
            .authz
            .audit()
            .record_system_entry_in_tx(
                db.tx(),
                CustomerObject::customer(customer_id),
                CoreCustomerAction::CUSTOMER_DECLINE_KYC,
            )
            .await?;

        if customer.decline_kyc(applicant_id, audit_info).did_execute() {
            self.repo.update_in_op(db, &mut customer).await?;
        }

        Ok(customer)
    }

    #[instrument(name = "customer.find_all", skip(self), err)]
    pub async fn find_all<T: From<Customer>>(
        &self,
        ids: &[CustomerId],
    ) -> Result<HashMap<CustomerId, T>, CustomerError> {
        self.repo.find_all(ids).await
    }

    #[instrument(name = "customer.update_telegram_id", skip(self), err)]
    pub async fn update_telegram_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        customer_id: impl Into<CustomerId> + std::fmt::Debug,
        new_telegram_id: String,
    ) -> Result<Customer, CustomerError> {
        let customer_id = customer_id.into();
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                CustomerObject::customer(customer_id),
                CoreCustomerAction::CUSTOMER_UPDATE,
            )
            .await?;

        let mut customer = self.repo.find_by_id(customer_id).await?;
        if customer
            .update_telegram_id(new_telegram_id, audit_info)
            .did_execute()
        {
            self.repo.update(&mut customer).await?;
        }

        Ok(customer)
    }

    #[instrument(name = "customer.update_email", skip(self), err)]
    pub async fn update_email(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        customer_id: impl Into<CustomerId> + std::fmt::Debug,
        new_email: String,
    ) -> Result<Customer, CustomerError> {
        let customer_id = customer_id.into();
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                CustomerObject::customer(customer_id),
                CoreCustomerAction::CUSTOMER_UPDATE,
            )
            .await?;

        let mut customer = self.repo.find_by_id(customer_id).await?;
        if customer.update_email(new_email, audit_info).did_execute() {
            self.repo.update(&mut customer).await?;
        }

        Ok(customer)
    }

    // Document management methods
    #[instrument(name = "customer.create_document", skip(self, content), err)]
    pub async fn create_document(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        customer_id: impl Into<CustomerId> + std::fmt::Debug,
        content: Vec<u8>,
        filename: impl Into<String> + std::fmt::Debug,
        content_type: impl Into<String> + std::fmt::Debug,
    ) -> Result<Document, CustomerError> {
        let customer_id = customer_id.into();
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                CustomerObject::all_customer_documents(),
                CoreCustomerAction::CUSTOMER_DOCUMENT_CREATE,
            )
            .await?;

        let document = self
            .document_storage
            .create_and_upload(
                audit_info,
                content,
                filename,
                content_type,
                customer_id,
                DocumentType::CustomerDocument,
            )
            .await?;

        Ok(document)
    }

    #[instrument(name = "customer.list_documents_for_customer", skip(self), err)]
    pub async fn list_documents_for_customer_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        customer_id: impl Into<CustomerId> + std::fmt::Debug,
    ) -> Result<Vec<Document>, CustomerError> {
        let customer_id = customer_id.into();
        self.authz
            .enforce_permission(
                sub,
                CustomerObject::all_customer_documents(),
                CoreCustomerAction::CUSTOMER_DOCUMENT_LIST,
            )
            .await?;

        let documents = self
            .document_storage
            .list_for_reference_id(customer_id)
            .await?;

        Ok(documents)
    }

    #[instrument(name = "customer.generate_document_download_link", skip(self), err)]
    pub async fn generate_document_download_link(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        document_id: impl Into<CustomerDocumentId> + std::fmt::Debug + Copy,
    ) -> Result<GeneratedDocumentDownloadLink, CustomerError> {
        let customer_document_id = document_id.into();
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                CustomerObject::customer_document(customer_document_id),
                CoreCustomerAction::CUSTOMER_DOCUMENT_GENERATE_DOWNLOAD_LINK,
            )
            .await?;

        let link = self
            .document_storage
            .generate_download_link(audit_info, customer_document_id)
            .await?;

        Ok(link)
    }

    #[instrument(name = "customer.delete_document", skip(self), err)]
    pub async fn delete_document(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        document_id: impl Into<CustomerDocumentId> + std::fmt::Debug + Copy,
    ) -> Result<(), CustomerError> {
        let customer_document_id = document_id.into();
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                CustomerObject::customer_document(customer_document_id),
                CoreCustomerAction::CUSTOMER_DOCUMENT_DELETE,
            )
            .await?;

        self.document_storage
            .delete(audit_info, customer_document_id)
            .await?;

        Ok(())
    }

    #[instrument(name = "customer.archive_document", skip(self), err)]
    pub async fn archive_document(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        document_id: impl Into<CustomerDocumentId> + std::fmt::Debug + Copy,
    ) -> Result<Document, CustomerError> {
        let customer_document_id = document_id.into();
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                CustomerObject::customer_document(customer_document_id),
                CoreCustomerAction::CUSTOMER_DOCUMENT_DELETE,
            )
            .await?;

        let document = self
            .document_storage
            .archive(audit_info, customer_document_id)
            .await?;

        Ok(document)
    }

    #[instrument(name = "customer.find_customer_document_by_id", skip(self), err)]
    pub async fn find_customer_document_by_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        document_id: impl Into<CustomerDocumentId> + std::fmt::Debug + Copy,
    ) -> Result<Option<Document>, CustomerError> {
        let customer_document_id = document_id.into();
        self.authz
            .enforce_permission(
                sub,
                CustomerObject::customer_document(customer_document_id),
                CoreCustomerAction::CUSTOMER_DOCUMENT_READ,
            )
            .await?;

        let document = self
            .document_storage
            .find_by_id(customer_document_id)
            .await?;

        Ok(document)
    }

    #[instrument(name = "customer.find_all_documents", skip(self), err)]
    pub async fn find_all_documents<T: From<Document>>(
        &self,
        ids: &[CustomerDocumentId],
    ) -> Result<HashMap<CustomerDocumentId, T>, CustomerError> {
        let document_ids: Vec<DocumentId> = ids.iter().map(|id| (*id).into()).collect();
        let documents: HashMap<DocumentId, T> =
            self.document_storage.find_all(&document_ids).await?;

        let result = documents
            .into_iter()
            .map(|(doc_id, document)| (CustomerDocumentId::from(doc_id), document))
            .collect();

        Ok(result)
    }
}
