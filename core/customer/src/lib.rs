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
use outbox::{Outbox, OutboxEventMarker};

use governance::{GovernanceAction, GovernanceEvent, GovernanceObject};

pub use entity::Customer;
use entity::*;
use error::*;
pub use event::*;
pub use primitives::*;
pub use repo::{customer_cursor::*, CustomerRepo, CustomersSortBy, FindManyCustomers, Sort};

use publisher::*;

pub struct Customers<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustomerEvent> + OutboxEventMarker<GovernanceEvent>,
{
    authz: Perms,
    outbox: Outbox<E>,
    repo: CustomerRepo<E>,
}

impl<Perms, E> Clone for Customers<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustomerEvent> + OutboxEventMarker<GovernanceEvent>,
{
    fn clone(&self) -> Self {
        Self {
            authz: self.authz.clone(),
            outbox: self.outbox.clone(),
            repo: self.repo.clone(),
        }
    }
}

impl<Perms, E> Customers<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCustomerAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CustomerObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreCustomerEvent> + OutboxEventMarker<GovernanceEvent>,
{
    pub fn new(pool: &sqlx::PgPool, authz: &Perms, outbox: &Outbox<E>) -> Self {
        let publisher = CustomerPublisher::new(outbox);
        let repo = CustomerRepo::new(pool, &publisher);
        Self {
            repo,
            authz: authz.clone(),
            outbox: outbox.clone(),
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

    pub async fn list(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        query: es_entity::PaginatedQueryArgs<CustomersCursor>,
        filter: FindManyCustomers,
        sort: impl Into<Sort<CustomersSortBy>>,
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

    pub async fn find_all<T: From<Customer>>(
        &self,
        ids: &[CustomerId],
    ) -> Result<HashMap<CustomerId, T>, CustomerError> {
        self.repo.find_all(ids).await
    }

    #[instrument(name = "customer.update", skip(self), err)]
    pub async fn update(
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
}
