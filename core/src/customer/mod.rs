mod config;
mod cursor;
mod entity;
pub mod error;
mod kratos;
mod repo;

use std::collections::HashMap;

use crate::{
    audit::Audit,
    authorization::{Action, Authorization, CustomerAction, Object},
    ledger::*,
    primitives::{CustomerId, KycLevel, Subject},
};

pub use config::*;
pub use cursor::*;
pub use entity::*;
use error::CustomerError;
use kratos::*;
pub use repo::CustomerRepo;

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
        ledger: &Ledger,
        config: &CustomerConfig,
        authz: &Authorization,
        audit: &Audit,
    ) -> Self {
        let repo = CustomerRepo::new(pool);
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

    pub async fn create_customer_through_admin(
        &self,
        sub: &Subject,
        email: String,
    ) -> Result<Customer, CustomerError> {
        let audit_info = self
            .authz
            .check_permission(sub, Object::Customer, CustomerAction::Create)
            .await?;
        let customer_id = self.kratos.create_identity(&email).await?.into();

        let ledger_account_ids = self
            .ledger
            .create_accounts_for_customer(customer_id)
            .await?;
        let new_customer = NewCustomer::builder()
            .id(customer_id)
            .email(email)
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
            .record_entry_in_tx(
                &mut db,
                &Subject::System(crate::primitives::SystemNode::Kratos),
                Object::Customer,
                Action::Customer(CustomerAction::Create),
                true,
            )
            .await?;

        let ledger_account_ids = self.ledger.create_accounts_for_customer(id).await?;
        let new_customer = NewCustomer::builder()
            .id(id)
            .email(email)
            .account_ids(ledger_account_ids)
            .audit_info(*audit_info)
            .build()
            .expect("Could not build customer");

        let customer = self.repo.create_in_tx(&mut db, new_customer).await;
        db.commit().await?;

        customer
    }

    pub async fn find_by_id(
        &self,
        sub: Option<&Subject>,
        id: impl Into<CustomerId> + std::fmt::Debug,
    ) -> Result<Option<Customer>, CustomerError> {
        if let Some(sub) = sub {
            self.authz
                .check_permission(sub, Object::Customer, CustomerAction::Read)
                .await?;
        }
        match self.repo.find_by_id(id.into()).await {
            Ok(customer) => Ok(Some(customer)),
            Err(CustomerError::CouldNotFindById(_)) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub async fn find_by_email(
        &self,
        sub: &Subject,
        email: String,
    ) -> Result<Option<Customer>, CustomerError> {
        self.authz
            .check_permission(sub, Object::Customer, CustomerAction::Read)
            .await?;

        match self.repo.find_by_email(&email).await {
            Ok(customer) => Ok(Some(customer)),
            Err(CustomerError::CouldNotFindByEmail(_)) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub async fn find_by_id_internal(
        &self,
        id: impl Into<CustomerId> + std::fmt::Debug,
    ) -> Result<Option<Customer>, CustomerError> {
        match self.repo.find_by_id(id.into()).await {
            Ok(customer) => Ok(Some(customer)),
            Err(CustomerError::CouldNotFindById(_)) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub async fn list(
        &self,
        sub: &Subject,
        query: crate::query::PaginatedQueryArgs<CustomerByNameCursor>,
    ) -> Result<crate::query::PaginatedQueryRet<Customer, CustomerByNameCursor>, CustomerError>
    {
        self.authz
            .check_permission(sub, Object::Customer, CustomerAction::List)
            .await?;
        self.repo.list(query).await
    }

    pub async fn start_kyc(
        &self,
        customer_id: CustomerId,
        applicant_id: String,
    ) -> Result<Customer, CustomerError> {
        let mut customer = self.repo.find_by_id(customer_id).await?;

        let mut db_tx = self.pool.begin().await?;

        let audit_info = self
            .audit
            .record_entry_in_tx(
                &mut db_tx,
                &Subject::System(crate::primitives::SystemNode::Sumsub),
                Object::Customer,
                Action::Customer(CustomerAction::StartKyc),
                true,
            )
            .await?;

        customer.start_kyc(applicant_id, audit_info);

        self.repo.persist_in_tx(&mut db_tx, &mut customer).await?;
        db_tx.commit().await?;

        Ok(customer)
    }

    pub async fn approve_basic(
        &self,
        customer_id: CustomerId,
        applicant_id: String,
    ) -> Result<Customer, CustomerError> {
        let mut customer = self.repo.find_by_id(customer_id).await?;

        let mut db_tx = self.pool.begin().await?;

        let audit_info = self
            .audit
            .record_entry_in_tx(
                &mut db_tx,
                &Subject::System(crate::primitives::SystemNode::Sumsub),
                Object::Customer,
                Action::Customer(CustomerAction::ApproveKyc),
                true,
            )
            .await?;

        customer.approve_kyc(KycLevel::Basic, applicant_id, audit_info);

        self.repo.persist_in_tx(&mut db_tx, &mut customer).await?;
        db_tx.commit().await?;

        Ok(customer)
    }

    pub async fn deactivate(
        &self,
        customer_id: CustomerId,
        applicant_id: String,
    ) -> Result<Customer, CustomerError> {
        let mut customer = self.repo.find_by_id(customer_id).await?;

        let mut db_tx = self.pool.begin().await?;

        let audit_info = self
            .audit
            .record_entry_in_tx(
                &mut db_tx,
                &Subject::System(crate::primitives::SystemNode::Sumsub),
                Object::Customer,
                Action::Customer(CustomerAction::DeclineKyc),
                true,
            )
            .await?;

        customer.deactivate(applicant_id, audit_info);

        self.repo.persist_in_tx(&mut db_tx, &mut customer).await?;
        db_tx.commit().await?;

        Ok(customer)
    }

    pub async fn find_all<T: From<Customer>>(
        &self,
        ids: &[CustomerId],
    ) -> Result<HashMap<CustomerId, T>, CustomerError> {
        self.repo.find_all(ids).await
    }
}
