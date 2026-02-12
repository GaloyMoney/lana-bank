#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod config;
mod customer_activity_repo;
mod entity;
pub mod error;
pub mod kyc;
mod primitives;
pub mod prospect;
pub mod public;
mod publisher;
mod repo;

use chrono::{DateTime, Utc};
use std::collections::HashMap;
use tracing::instrument;
use tracing_macros::record_error_severity;

use audit::AuditSvc;
use authz::PermissionCheck;
use document_storage::{
    Document, DocumentId, DocumentStorage, DocumentType, GeneratedDocumentDownloadLink,
};
use es_entity::clock::ClockHandle;
use obix::out::{Outbox, OutboxEventMarker};
use public_id::PublicIds;

pub use config::*;
pub use customer_activity_repo::CustomerActivityRepo;
pub use entity::Customer;
use error::*;
pub use primitives::*;
pub use prospect::{Prospect, ProspectRepo, ProspectsSortBy, prospect_cursor};
pub use public::*;
pub use repo::{CustomerRepo, CustomersFilters, CustomersSortBy, Sort, customer_cursor::*};

pub const CUSTOMER_DOCUMENT: DocumentType = DocumentType::new("customer_document");

#[cfg(feature = "json-schema")]
pub mod event_schema {
    pub use crate::entity::CustomerEvent;
}

use prospect::publisher::ProspectPublisher;
use publisher::*;

pub struct Customers<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    authz: Perms,
    outbox: Outbox<E>,
    repo: CustomerRepo<E>,
    prospect_repo: ProspectRepo<E>,
    customer_activity_repo: CustomerActivityRepo,
    document_storage: DocumentStorage,
    public_ids: PublicIds,
    config: CustomerConfig,
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
            prospect_repo: self.prospect_repo.clone(),
            customer_activity_repo: self.customer_activity_repo.clone(),
            document_storage: self.document_storage.clone(),
            public_ids: self.public_ids.clone(),
            config: self.config.clone(),
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
        public_id_service: PublicIds,
        clock: ClockHandle,
    ) -> Self {
        let publisher = CustomerPublisher::new(outbox);
        let repo = CustomerRepo::new(pool, &publisher, clock.clone());
        let prospect_publisher = ProspectPublisher::new(outbox);
        let prospect_repo = ProspectRepo::new(pool, &prospect_publisher, clock);
        let customer_activity_repo = CustomerActivityRepo::new(pool.clone());
        Self {
            repo,
            prospect_repo,
            authz: authz.clone(),
            outbox: outbox.clone(),
            customer_activity_repo,
            document_storage,
            public_ids: public_id_service,
            config: CustomerConfig::default(),
        }
    }

    pub async fn subject_can_create_prospect(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        enforce: bool,
    ) -> Result<Option<AuditInfo>, CustomerError> {
        Ok(self
            .authz
            .evaluate_permission(
                sub,
                CustomerObject::all_prospects(),
                CoreCustomerAction::PROSPECT_CREATE,
                enforce,
            )
            .await?)
    }

    #[record_error_severity]
    #[instrument(name = "customer.create_prospect", fields(prospect_id = tracing::field::Empty), skip(self))]
    pub async fn create_prospect(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        email: impl Into<String> + std::fmt::Debug,
        telegram_handle: impl Into<String> + std::fmt::Debug,
        customer_type: impl Into<CustomerType> + std::fmt::Debug,
    ) -> Result<Prospect, CustomerError> {
        self.subject_can_create_prospect(sub, true)
            .await?
            .expect("audit info missing");

        let prospect_id = ProspectId::new();
        tracing::Span::current().record("prospect_id", prospect_id.to_string().as_str());

        let mut db = self.prospect_repo.begin_op().await?;

        let public_id = self
            .public_ids
            .create_in_op(&mut db, PROSPECT_REF_TARGET, prospect_id)
            .await?;

        let new_prospect = prospect::NewProspect::builder()
            .id(prospect_id)
            .email(email.into())
            .telegram_handle(telegram_handle.into())
            .customer_type(customer_type)
            .public_id(public_id.id)
            .build()
            .expect("Could not build prospect");

        let prospect = self
            .prospect_repo
            .create_in_op(&mut db, new_prospect)
            .await?;

        db.commit().await?;

        Ok(prospect)
    }

    #[record_error_severity]
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

    #[record_error_severity]
    #[instrument(name = "customer.find_by_id", skip(self))]
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

        self.repo.maybe_find_by_id(id).await
    }

    #[record_error_severity]
    #[instrument(name = "customer.find_by_id_without_audit", skip(self))]
    pub async fn find_by_id_without_audit(
        &self,
        id: impl Into<CustomerId> + std::fmt::Debug,
    ) -> Result<Customer, CustomerError> {
        self.repo.find_by_id(id.into()).await
    }

    #[record_error_severity]
    #[instrument(name = "customer.find_by_email", skip(self))]
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

        self.repo.maybe_find_by_email(email).await
    }

    #[record_error_severity]
    #[instrument(name = "customer.find_by_public_id", skip(self))]
    pub async fn find_by_public_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        public_id: impl Into<public_id::PublicId> + std::fmt::Debug,
    ) -> Result<Option<Customer>, CustomerError> {
        self.authz
            .enforce_permission(
                sub,
                CustomerObject::all_customers(),
                CoreCustomerAction::CUSTOMER_READ,
            )
            .await?;

        self.repo.maybe_find_by_public_id(public_id.into()).await
    }

    #[record_error_severity]
    #[instrument(name = "customer.list", skip(self))]
    pub async fn list(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        query: es_entity::PaginatedQueryArgs<CustomersCursor>,
        filter: CustomersFilters,
        sort: impl Into<Sort<CustomersSortBy>> + std::fmt::Debug,
    ) -> Result<es_entity::PaginatedQueryRet<Customer, CustomersCursor>, CustomerError> {
        self.authz
            .enforce_permission(
                sub,
                CustomerObject::all_customers(),
                CoreCustomerAction::CUSTOMER_LIST,
            )
            .await?;
        self.repo.list_for_filters(filter, sort.into(), query).await
    }

    #[record_error_severity]
    #[instrument(name = "customer.handle_kyc_started_if_exists", skip(self))]
    pub async fn handle_kyc_started_if_exists(
        &self,
        prospect_id: ProspectId,
        applicant_id: String,
    ) -> Result<Option<Prospect>, CustomerError> {
        let Some(mut prospect) = self.prospect_repo.maybe_find_by_id(prospect_id).await? else {
            return Ok(None);
        };

        self.authz
            .audit()
            .record_system_entry(
                crate::primitives::SUMSUB,
                CustomerObject::prospect(prospect.id),
                CoreCustomerAction::PROSPECT_START_KYC,
            )
            .await?;

        if prospect.start_kyc(applicant_id).did_execute() {
            self.prospect_repo.update(&mut prospect).await?;
        }

        Ok(Some(prospect))
    }

    #[record_error_severity]
    #[instrument(name = "customer.handle_kyc_started", skip(self))]
    pub async fn handle_kyc_started(
        &self,
        prospect_id: ProspectId,
        applicant_id: String,
    ) -> Result<Prospect, CustomerError> {
        let mut prospect = self.prospect_repo.find_by_id(prospect_id).await?;

        self.authz
            .audit()
            .record_system_entry(
                crate::primitives::SUMSUB,
                CustomerObject::prospect(prospect.id),
                CoreCustomerAction::PROSPECT_START_KYC,
            )
            .await?;

        if prospect.start_kyc(applicant_id).did_execute() {
            self.prospect_repo.update(&mut prospect).await?;
        }

        Ok(prospect)
    }

    #[record_error_severity]
    #[instrument(name = "customer.handle_kyc_approved_if_exists", skip(self))]
    pub async fn handle_kyc_approved_if_exists(
        &self,
        prospect_id: ProspectId,
        applicant_id: String,
    ) -> Result<Option<Customer>, CustomerError> {
        let Some(mut prospect) = self.prospect_repo.maybe_find_by_id(prospect_id).await? else {
            return Ok(None);
        };

        self.authz
            .audit()
            .record_system_entry(
                crate::primitives::SUMSUB,
                CustomerObject::prospect(prospect.id),
                CoreCustomerAction::PROSPECT_APPROVE_KYC,
            )
            .await?;

        match prospect.approve_kyc(KycLevel::Basic, applicant_id) {
            es_entity::Idempotent::Executed(new_customer) => {
                let mut db = self.prospect_repo.begin_op().await?;
                self.prospect_repo
                    .update_in_op(&mut db, &mut prospect)
                    .await?;
                let customer = self.repo.create_in_op(&mut db, new_customer).await?;
                db.commit().await?;
                Ok(Some(customer))
            }
            es_entity::Idempotent::AlreadyApplied => {
                let customer_id = CustomerId::from(prospect_id);
                Ok(self.repo.maybe_find_by_id(customer_id).await?)
            }
        }
    }

    #[record_error_severity]
    #[instrument(name = "customer.handle_kyc_approved", skip(self))]
    pub async fn handle_kyc_approved(
        &self,
        prospect_id: ProspectId,
        applicant_id: String,
    ) -> Result<Customer, CustomerError> {
        let mut prospect = self.prospect_repo.find_by_id(prospect_id).await?;

        self.authz
            .audit()
            .record_system_entry(
                crate::primitives::SUMSUB,
                CustomerObject::prospect(prospect.id),
                CoreCustomerAction::PROSPECT_APPROVE_KYC,
            )
            .await?;

        match prospect.approve_kyc(KycLevel::Basic, applicant_id) {
            es_entity::Idempotent::Executed(new_customer) => {
                let mut db = self.prospect_repo.begin_op().await?;
                self.prospect_repo
                    .update_in_op(&mut db, &mut prospect)
                    .await?;
                let customer = self.repo.create_in_op(&mut db, new_customer).await?;
                db.commit().await?;
                Ok(customer)
            }
            es_entity::Idempotent::AlreadyApplied => {
                let customer_id = CustomerId::from(prospect_id);
                Ok(self.repo.find_by_id(customer_id).await?)
            }
        }
    }

    #[record_error_severity]
    #[instrument(name = "customer.handle_kyc_declined_if_exists", skip(self))]
    pub async fn handle_kyc_declined_if_exists(
        &self,
        prospect_id: ProspectId,
        applicant_id: String,
    ) -> Result<Option<Prospect>, CustomerError> {
        let Some(mut prospect) = self.prospect_repo.maybe_find_by_id(prospect_id).await? else {
            return Ok(None);
        };

        self.authz
            .audit()
            .record_system_entry(
                crate::primitives::SUMSUB,
                CustomerObject::prospect(prospect.id),
                CoreCustomerAction::PROSPECT_DECLINE_KYC,
            )
            .await?;

        if prospect.decline_kyc(applicant_id).did_execute() {
            self.prospect_repo.update(&mut prospect).await?;
        }

        Ok(Some(prospect))
    }

    #[record_error_severity]
    #[instrument(name = "customer.handle_kyc_declined", skip(self))]
    pub async fn handle_kyc_declined(
        &self,
        prospect_id: ProspectId,
        applicant_id: String,
    ) -> Result<Prospect, CustomerError> {
        let mut prospect = self.prospect_repo.find_by_id(prospect_id).await?;

        self.authz
            .audit()
            .record_system_entry(
                crate::primitives::SUMSUB,
                CustomerObject::prospect(prospect.id),
                CoreCustomerAction::PROSPECT_DECLINE_KYC,
            )
            .await?;

        if prospect.decline_kyc(applicant_id).did_execute() {
            self.prospect_repo.update(&mut prospect).await?;
        }

        Ok(prospect)
    }

    #[record_error_severity]
    #[instrument(name = "customer.find_prospect_by_id", skip(self))]
    pub async fn find_prospect_by_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        id: impl Into<ProspectId> + std::fmt::Debug,
    ) -> Result<Option<Prospect>, CustomerError> {
        let id = id.into();
        self.authz
            .enforce_permission(
                sub,
                CustomerObject::prospect(id),
                CoreCustomerAction::PROSPECT_READ,
            )
            .await?;

        Ok(self.prospect_repo.maybe_find_by_id(id).await?)
    }

    #[record_error_severity]
    #[instrument(name = "customer.find_prospect_by_id_without_audit", skip(self))]
    pub async fn find_prospect_by_id_without_audit(
        &self,
        id: impl Into<ProspectId> + std::fmt::Debug,
    ) -> Result<Prospect, CustomerError> {
        Ok(self.prospect_repo.find_by_id(id.into()).await?)
    }

    #[record_error_severity]
    #[instrument(name = "customer.find_prospect_by_public_id", skip(self))]
    pub async fn find_prospect_by_public_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        public_id: impl Into<public_id::PublicId> + std::fmt::Debug,
    ) -> Result<Option<Prospect>, CustomerError> {
        self.authz
            .enforce_permission(
                sub,
                CustomerObject::all_prospects(),
                CoreCustomerAction::PROSPECT_READ,
            )
            .await?;

        Ok(self
            .prospect_repo
            .maybe_find_by_public_id(public_id.into())
            .await?)
    }

    #[record_error_severity]
    #[instrument(name = "customer.list_prospects", skip(self))]
    pub async fn list_prospects(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        query: es_entity::PaginatedQueryArgs<prospect::prospect_cursor::ProspectsByCreatedAtCursor>,
        direction: es_entity::ListDirection,
    ) -> Result<
        es_entity::PaginatedQueryRet<
            Prospect,
            prospect::prospect_cursor::ProspectsByCreatedAtCursor,
        >,
        CustomerError,
    > {
        self.authz
            .enforce_permission(
                sub,
                CustomerObject::all_prospects(),
                CoreCustomerAction::PROSPECT_LIST,
            )
            .await?;
        Ok(self
            .prospect_repo
            .list_by_created_at(query, direction)
            .await?)
    }

    #[record_error_severity]
    #[instrument(name = "customer.find_all", skip(self))]
    pub async fn find_all<T: From<Customer>>(
        &self,
        ids: &[CustomerId],
    ) -> Result<HashMap<CustomerId, T>, CustomerError> {
        self.repo.find_all(ids).await
    }

    #[record_error_severity]
    #[instrument(name = "customer.find_all_prospects", skip(self))]
    pub async fn find_all_prospects<T: From<Prospect>>(
        &self,
        ids: &[ProspectId],
    ) -> Result<HashMap<ProspectId, T>, CustomerError> {
        Ok(self.prospect_repo.find_all(ids).await?)
    }

    #[record_error_severity]
    #[instrument(name = "customer.update_telegram_handle", skip(self))]
    pub async fn update_telegram_handle(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        customer_id: impl Into<CustomerId> + std::fmt::Debug,
        new_telegram_handle: String,
    ) -> Result<Customer, CustomerError> {
        let customer_id = customer_id.into();
        self.authz
            .enforce_permission(
                sub,
                CustomerObject::customer(customer_id),
                CoreCustomerAction::CUSTOMER_UPDATE,
            )
            .await?;

        let mut customer = self.repo.find_by_id(customer_id).await?;
        if customer
            .update_telegram_handle(new_telegram_handle)
            .did_execute()
        {
            self.repo.update(&mut customer).await?;
        }

        Ok(customer)
    }

    #[record_error_severity]
    #[instrument(name = "customer.update_email", skip(self))]
    pub async fn update_email(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        customer_id: impl Into<CustomerId> + std::fmt::Debug,
        new_email: String,
    ) -> Result<Customer, CustomerError> {
        let customer_id = customer_id.into();
        self.authz
            .enforce_permission(
                sub,
                CustomerObject::customer(customer_id),
                CoreCustomerAction::CUSTOMER_UPDATE,
            )
            .await?;

        let mut customer = self.repo.find_by_id(customer_id).await?;
        if customer.update_email(new_email).did_execute() {
            self.repo.update(&mut customer).await?;
        }

        Ok(customer)
    }

    // Document management methods
    #[record_error_severity]
    #[instrument(name = "customer.create_document", skip(self, content))]
    pub async fn create_document(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        customer_id: impl Into<CustomerId> + std::fmt::Debug,
        content: Vec<u8>,
        filename: impl Into<String> + std::fmt::Debug,
        content_type: impl Into<String> + std::fmt::Debug,
    ) -> Result<Document, CustomerError> {
        let customer_id = customer_id.into();
        self.authz
            .enforce_permission(
                sub,
                CustomerObject::all_customer_documents(),
                CoreCustomerAction::CUSTOMER_DOCUMENT_CREATE,
            )
            .await?;

        let document = self
            .document_storage
            .create_and_upload(
                content,
                filename,
                content_type,
                customer_id,
                CUSTOMER_DOCUMENT,
            )
            .await?;

        Ok(document)
    }

    #[record_error_severity]
    #[instrument(name = "customer.list_documents_for_customer", skip(self))]
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

    #[record_error_severity]
    #[instrument(name = "customer.generate_document_download_link", skip(self))]
    pub async fn generate_document_download_link(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        document_id: impl Into<CustomerDocumentId> + std::fmt::Debug + Copy,
    ) -> Result<GeneratedDocumentDownloadLink, CustomerError> {
        let customer_document_id = document_id.into();
        self.authz
            .enforce_permission(
                sub,
                CustomerObject::customer_document(customer_document_id),
                CoreCustomerAction::CUSTOMER_DOCUMENT_GENERATE_DOWNLOAD_LINK,
            )
            .await?;

        let link = self
            .document_storage
            .generate_download_link(customer_document_id)
            .await?;

        Ok(link)
    }

    #[record_error_severity]
    #[instrument(name = "customer.delete_document", skip(self))]
    pub async fn delete_document(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        document_id: impl Into<CustomerDocumentId> + std::fmt::Debug + Copy,
    ) -> Result<(), CustomerError> {
        let customer_document_id = document_id.into();
        self.authz
            .enforce_permission(
                sub,
                CustomerObject::customer_document(customer_document_id),
                CoreCustomerAction::CUSTOMER_DOCUMENT_DELETE,
            )
            .await?;

        self.document_storage.delete(customer_document_id).await?;

        Ok(())
    }

    #[record_error_severity]
    #[instrument(name = "customer.archive_document", skip(self))]
    pub async fn archive_document(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        document_id: impl Into<CustomerDocumentId> + std::fmt::Debug + Copy,
    ) -> Result<Document, CustomerError> {
        let customer_document_id = document_id.into();
        self.authz
            .enforce_permission(
                sub,
                CustomerObject::customer_document(customer_document_id),
                CoreCustomerAction::CUSTOMER_DOCUMENT_DELETE,
            )
            .await?;

        let document = self.document_storage.archive(customer_document_id).await?;

        Ok(document)
    }

    #[record_error_severity]
    #[instrument(name = "customer.find_customer_document_by_id", skip(self))]
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

        match self.document_storage.find_by_id(customer_document_id).await {
            Ok(document) => Ok(Some(document)),
            Err(e) if e.was_not_found() => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    #[record_error_severity]
    #[instrument(name = "customer.find_all_documents", skip(self))]
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

    #[record_error_severity]
    #[instrument(name = "customer.record_last_activity_date", skip(self))]
    pub async fn record_last_activity_date(
        &self,
        customer_id: CustomerId,
        activity_date: DateTime<Utc>,
    ) -> Result<(), CustomerError> {
        self.customer_activity_repo
            .upsert_activity(customer_id, activity_date)
            .await?;
        Ok(())
    }

    async fn update_customers_by_activity_and_date_range(
        &self,
        start_threshold: DateTime<Utc>,
        end_threshold: DateTime<Utc>,
        activity: Activity,
    ) -> Result<(), CustomerError> {
        let customer_ids = self
            .customer_activity_repo
            .find_customers_needing_activity_update(start_threshold, end_threshold, activity)
            .await?;
        // TODO: Add a batch update for the customers
        for customer_id in customer_ids {
            let mut customer = self.repo.find_by_id(customer_id).await?;
            if customer.update_activity(activity).did_execute() {
                self.repo.update(&mut customer).await?;
            }
        }

        Ok(())
    }

    #[record_error_severity]
    #[instrument(name = "customer.perform_customer_activity_status_update", skip(self))]
    pub async fn perform_customer_activity_status_update(
        &self,
        now: DateTime<Utc>,
    ) -> Result<(), CustomerError> {
        let escheatment_date = self.config.get_escheatment_threshold_date(now);
        let inactive_date = self.config.get_inactive_threshold_date(now);

        // Update customers with very old activity (10+ years) to Suspended
        self.update_customers_by_activity_and_date_range(
            EARLIEST_SEARCH_START,
            escheatment_date,
            Activity::Suspended,
        )
        .await?;

        // Update customers with old activity (1-10 years) to Inactive
        self.update_customers_by_activity_and_date_range(
            escheatment_date,
            inactive_date,
            Activity::Inactive,
        )
        .await?;

        // Update customers with recent activity (<1 year) to Active
        self.update_customers_by_activity_and_date_range(inactive_date, now, Activity::Active)
            .await?;

        Ok(())
    }
}
