mod entity;
pub mod error;
mod repo;

use crate::{
    authorization::{Authorization, CreditFacilityAction, Object},
    customer::Customers,
    data_export::Export,
    ledger::{credit_facility::*, Ledger},
    primitives::{CreditFacilityId, CustomerId, Subject, UsdCents, UserId},
    user::{UserRepo, Users},
};

pub use entity::*;
use error::*;
use repo::*;

#[derive(Clone)]
pub struct CreditFacilities {
    pool: sqlx::PgPool,
    authz: Authorization,
    customers: Customers,
    credit_facility_repo: CreditFacilityRepo,
    user_repo: UserRepo,
    ledger: Ledger,
}

impl CreditFacilities {
    pub fn new(
        pool: &sqlx::PgPool,
        export: &Export,
        authz: &Authorization,
        customers: &Customers,
        users: &Users,
        ledger: &Ledger,
    ) -> Self {
        let repo = CreditFacilityRepo::new(pool, export);

        Self {
            pool: pool.clone(),
            authz: authz.clone(),
            customers: customers.clone(),
            credit_facility_repo: repo,
            user_repo: users.repo().clone(),
            ledger: ledger.clone(),
        }
    }

    pub async fn create(
        &self,
        sub: &Subject,
        customer_id: impl Into<CustomerId> + std::fmt::Debug,
        facility: UsdCents,
    ) -> Result<CreditFacility, CreditFacilityError> {
        let customer_id = customer_id.into();

        let audit_info = self
            .authz
            .check_permission(sub, Object::CreditFacility, CreditFacilityAction::Create)
            .await?;

        let customer = match self.customers.find_by_id(Some(sub), customer_id).await? {
            Some(customer) => customer,
            None => return Err(CreditFacilityError::CustomerNotFound(customer_id)),
        };

        let new_credit_facility = NewCreditFacility::builder()
            .id(CreditFacilityId::new())
            .customer_id(customer_id)
            .facility(facility)
            .account_ids(CreditFacilityAccountIds::new())
            .customer_account_ids(customer.account_ids)
            .audit_info(audit_info)
            .build()
            .expect("could not build new credit facility");

        let mut db_tx = self.pool.begin().await?;
        let credit_facility = self
            .credit_facility_repo
            .create_in_tx(&mut db_tx, new_credit_facility)
            .await?;
        self.ledger
            .create_accounts_for_credit_facility(credit_facility.id, credit_facility.account_ids)
            .await?;

        db_tx.commit().await?;

        Ok(credit_facility)
    }

    pub async fn add_approval(
        &self,
        sub: &Subject,
        credit_facility_id: impl Into<CreditFacilityId> + std::fmt::Debug,
    ) -> Result<CreditFacility, CreditFacilityError> {
        let credit_facility_id = credit_facility_id.into();

        let audit_info = self
            .authz
            .check_permission(sub, Object::CreditFacility, CreditFacilityAction::Approve)
            .await?;

        let mut credit_facility = self
            .credit_facility_repo
            .find_by_id(credit_facility_id)
            .await?;

        let subject_id = uuid::Uuid::from(sub);
        let user = self.user_repo.find_by_id(UserId::from(subject_id)).await?;

        let mut db_tx = self.pool.begin().await?;

        if let Some(credit_facility_approval) =
            credit_facility.add_approval(user.id, user.current_roles(), audit_info)?
        {
            let executed_at = self
                .ledger
                .approve_credit_facility(credit_facility_approval.clone())
                .await?;
            credit_facility.confirm_approval(credit_facility_approval, executed_at, audit_info);
        }

        self.credit_facility_repo
            .persist_in_tx(&mut db_tx, &mut credit_facility)
            .await?;
        db_tx.commit().await?;

        Ok(credit_facility)
    }
}
