use sqlx::PgPool;

use es_entity::*;

use crate::primitives::*;

use super::{entity::*, error::PaymentError};

#[derive(EsRepo, Clone)]
#[es_repo(
    entity = "Payment",
    err = "PaymentError",
    columns(
        credit_facility_id(ty = "CreditFacilityId", list_for, update(persist = false)),
        credit_facility_payment_idx(ty = "i32", list_by)
    ),
    tbl_prefix = "core"
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
