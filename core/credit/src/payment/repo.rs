use sqlx::PgPool;

use es_entity::*;

use crate::primitives::*;

use super::{entity::*, error::PaymentError};

#[derive(EsRepo, Clone)]
#[es_repo(
    entity = "Payment",
    err = "PaymentError",
    columns(credit_facility_id(ty = "CreditFacilityId", list_for, update(persist = false)),)
)]
pub struct PaymentRepo {
    #[allow(dead_code)]
    pool: PgPool,
}

impl PaymentRepo {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }
}
