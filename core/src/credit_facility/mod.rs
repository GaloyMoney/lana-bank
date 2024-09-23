mod entity;
pub mod error;
mod repo;

use sqlx::PgPool;

use crate::data_export::Export;

pub use entity::*;
use error::*;
use repo::*;

pub struct CreditFacilities {
    credit_facility_repo: CreditFacilityRepo,
}

impl CreditFacilities {
    pub fn new(pool: &PgPool, export: &Export) -> Self {
        let credit_facility_repo = CreditFacilityRepo::new(pool, export);
        Self {
            credit_facility_repo,
        }
    }
}
