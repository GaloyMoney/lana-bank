mod config;
mod entity;
pub mod error;
mod kratos;
mod repo;

use std::collections::HashMap;
use tracing::instrument;

use authz::PermissionCheck;

use crate::{
    audit::{Audit, AuditInfo, AuditSvc},
    authorization::{Action, Authorization, CustomerAction, CustomerAllOrOne, Object},
    data_export::Export,
    ledger::*,
    primitives::{CustomerId, KycLevel, Subject},
};

pub use config::*;
pub use entity::*;
use error::CustomerError;
use kratos::*;
pub use repo::{cursor::*, CustomerRepo};

#[derive(Clone)]
pub struct Customers {
    pool: sqlx::PgPool,
    repo: CustomerRepo,
    ledger: Ledger,
    kratos: KratosClient,
    authz: Authorization,
    audit: Audit,
}

impl Customers {
    pub fn new(
        pool: &sqlx::PgPool,
        config: &CustomerConfig,
        ledger: &Ledger,
        authz: &Authorization,
        audit: &Audit,
        export: &Export,
    ) -> Self {
        let repo = CustomerRepo::new(pool, export);
        let kratos = KratosClient::new(&config.kratos);
        Self {
            pool: pool.clone(),
            repo,
            ledger: ledger.clone(),
            kratos,
            authz: authz.clone(),
            audit: audit.clone(),
        }
    }

    pub fn repo(&self) -> &CustomerRepo {
        &self.repo
    }

    pub async fn subject_can_create_customer(
        &self,
        sub: &Subject,
        enforce: bool,
    ) -> Result<Option<AuditInfo>, CustomerError> {
        Ok(self
            .authz
            .evaluate_permission(
                sub,
                Object::Customer(CustomerAllOrOne::All),
                CustomerAction::Create,
                enforce,
            )
            .await?)
    }

    #[instrument(name = "lava.customer.create", skip(self), err)]
    pub async fn create(
        &self,
        sub: &Subject,
        email: String,
        telegram_id: String,
    ) -> Result<Customer, CustomerError> {
        let audit_info = self
            .subject_can_create_customer(sub, true)
            .await?
            .expect("audit info missing");
        let customer_id = self.kratos.create_identity(&email).await?.into();

        let ledger_account_ids = self
            .ledger
            .create_accounts_for_customer(customer_id)
            .await?;
        let new_customer = NewCustomer::builder()
            .id(customer_id)
            .email(email)
            .telegram_id(telegram_id)
            .account_ids(ledger_account_ids)
            .audit_info(audit_info)
            .build()
            .expect("Could not build customer");

        let mut db = self.pool.begin().await?;
        let customer = self.repo.create_in_tx(&mut db, new_customer).await;
        db.commit().await?;

        customer
    }

    pub async fn create_customer_through_kratos(
        &self,
        id: CustomerId,
        email: String,
    ) -> Result<Customer, CustomerError> {
        let mut db = self.pool.begin().await?;

        let audit_info = &self
            .audit
            .record_system_entry_in_tx(
                &mut db,
                Object::Customer(CustomerAllOrOne::All),
                Action::Customer(CustomerAction::Create),
            )
            .await?;

        let ledger_account_ids = self.ledger.create_accounts_for_customer(id).await?;
        let new_customer = NewCustomer::builder()
            .id(id)
            .email(email)
            .account_ids(ledger_account_ids)
            .audit_info(audit_info.clone())
            .build()
            .expect("Could not build customer");

        let customer = self.repo.create_in_tx(&mut db, new_customer).await;
        db.commit().await?;

        customer
    }

    #[instrument(name = "customer.create_customer", skip(self), err)]
    pub async fn find_by_id(
        &self,
        sub: &Subject,
        id: impl Into<CustomerId> + std::fmt::Debug,
    ) -> Result<Option<Customer>, CustomerError> {
        let id = id.into();
        self.authz
            .enforce_permission(
                sub,
                Object::Customer(CustomerAllOrOne::ById(id)),
                CustomerAction::Read,
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
        sub: &Subject,
        email: String,
    ) -> Result<Option<Customer>, CustomerError> {
        self.authz
            .enforce_permission(
                sub,
                Object::Customer(CustomerAllOrOne::All),
                CustomerAction::Read,
            )
            .await?;

        match self.repo.find_by_email(email).await {
            Ok(customer) => Ok(Some(customer)),
            Err(e) if e.was_not_found() => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub async fn find_by_id_internal(
        &self,
        id: impl Into<CustomerId> + std::fmt::Debug,
    ) -> Result<Option<Customer>, CustomerError> {
        match self.repo.find_by_id(id.into()).await {
            Ok(customer) => Ok(Some(customer)),
            Err(e) if e.was_not_found() => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub async fn list(
        &self,
        sub: &Subject,
        query: es_entity::PaginatedQueryArgs<CustomerByEmailCursor>,
    ) -> Result<es_entity::PaginatedQueryRet<Customer, CustomerByEmailCursor>, CustomerError> {
        self.authz
            .enforce_permission(
                sub,
                Object::Customer(CustomerAllOrOne::All),
                CustomerAction::List,
            )
            .await?;
        self.repo
            .list_by_email(query, es_entity::ListDirection::Ascending)
            .await
    }

    pub async fn start_kyc(
        &self,
        db: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        customer_id: CustomerId,
        applicant_id: String,
    ) -> Result<Customer, CustomerError> {
        let mut customer = self.repo.find_by_id(customer_id).await?;

        let audit_info = self
            .audit
            .record_system_entry_in_tx(
                db,
                Object::Customer(CustomerAllOrOne::ById(customer_id)),
                Action::Customer(CustomerAction::StartKyc),
            )
            .await?;

        customer.start_kyc(applicant_id, audit_info);

        self.repo.update_in_tx(db, &mut customer).await?;

        Ok(customer)
    }

    pub async fn approve_basic(
        &self,
        db: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        customer_id: CustomerId,
        applicant_id: String,
    ) -> Result<Customer, CustomerError> {
        let mut customer = self.repo.find_by_id(customer_id).await?;

        let audit_info = self
            .audit
            .record_system_entry_in_tx(
                db,
                Object::Customer(CustomerAllOrOne::ById(customer_id)),
                Action::Customer(CustomerAction::ApproveKyc),
            )
            .await?;

        customer.approve_kyc(KycLevel::Basic, applicant_id, audit_info);

        self.repo.update_in_tx(db, &mut customer).await?;

        Ok(customer)
    }

    pub async fn deactivate(
        &self,
        db: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        customer_id: CustomerId,
        applicant_id: String,
    ) -> Result<Customer, CustomerError> {
        let mut customer = self.repo.find_by_id(customer_id).await?;

        let audit_info = self
            .audit
            .record_system_entry_in_tx(
                db,
                Object::Customer(CustomerAllOrOne::ById(customer_id)),
                Action::Customer(CustomerAction::DeclineKyc),
            )
            .await?;

        customer.deactivate(applicant_id, audit_info);
        self.repo.update_in_tx(db, &mut customer).await?;

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
        sub: &Subject,
        customer_id: impl Into<CustomerId> + std::fmt::Debug,
        new_telegram_id: String,
    ) -> Result<Customer, CustomerError> {
        let customer_id = customer_id.into();
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                Object::Customer(CustomerAllOrOne::ById(customer_id)),
                CustomerAction::Update,
            )
            .await?;

        let mut customer = self.repo.find_by_id(customer_id).await?;
        customer.update_telegram_id(new_telegram_id, audit_info);

        self.repo.update(&mut customer).await?;

        Ok(customer)
    }
}
