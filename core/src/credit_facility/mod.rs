mod entity;
pub mod error;
mod repo;

use crate::{
    authorization::{Authorization, Object, UserAction},
    customer::Customers,
    data_export::Export,
    primitives::{CreditFacilityId, CustomerId, Subject, UsdCents},
};

pub use entity::*;
use error::*;
use repo::*;

#[derive(Clone)]
pub struct CreditFacilities {
    pool: sqlx::PgPool,
    authz: Authorization,
    customers: Customers,
    repo: CreditFacilityRepo,
}

impl CreditFacilities {
    pub fn new(
        pool: &sqlx::PgPool,
        export: &Export,
        authz: &Authorization,
        customers: &Customers,
    ) -> Self {
        let repo = CreditFacilityRepo::new(pool, export);
        Self {
            pool: pool.clone(),
            authz: authz.clone(),
            customers: customers.clone(),
            repo,
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
            .check_permission(sub, Object::User, UserAction::Create)
            .await?;

        let _customer = match self.customers.find_by_id(Some(sub), customer_id).await? {
            Some(customer) => customer,
            None => return Err(CreditFacilityError::CustomerNotFound(customer_id)),
        };

        let new_credit_facility = NewCreditFacility::builder()
            .id(CreditFacilityId::new())
            .customer_id(customer_id)
            .facility(facility)
            .audit_info(audit_info)
            .build()
            .expect("could not build new credit facility");

        let mut db_tx = self.pool.begin().await?;
        let credit_facility = self
            .repo
            .create_in_tx(&mut db_tx, new_credit_facility)
            .await?;
        db_tx.commit().await?;

        Ok(credit_facility)
    }
}
