mod entity;
pub mod error;
mod repo;

use crate::{
    authorization::{Authorization, Object, UserAction},
    data_export::Export,
    primitives::{CreditFacilityId, Subject},
};

pub use entity::*;
use error::*;
use repo::*;

#[derive(Clone)]
pub struct CreditFacilities {
    pool: sqlx::PgPool,
    authz: Authorization,
    repo: CreditFacilityRepo,
}

impl CreditFacilities {
    pub fn new(pool: &sqlx::PgPool, export: &Export, authz: &Authorization) -> Self {
        let repo = CreditFacilityRepo::new(pool, export);
        Self {
            pool: pool.clone(),
            authz: authz.clone(),
            repo,
        }
    }

    pub async fn create(&self, sub: &Subject) -> Result<CreditFacility, CreditFacilityError> {
        let audit_info = self
            .authz
            .check_permission(sub, Object::User, UserAction::Create)
            .await?;

        let new_credit_facility = NewCreditFacility::builder()
            .id(CreditFacilityId::new())
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
