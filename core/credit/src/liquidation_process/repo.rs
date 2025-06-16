use sqlx::PgPool;

use es_entity::*;

use crate::primitives::*;

use super::{entity::*, error::*};

#[derive(EsRepo)]
#[es_repo(
    entity = "LiquidationProcess",
    err = "LiquidationProcessError",
    columns(
        parent_obligation_id(ty = "ObligationId", list_for, update(persist = false)),
        credit_facility_id(ty = "CreditFacilityId", list_for, update(persist = false)),
    ),
    tbl_prefix = "core"
)]
pub struct LiquidationProcessRepo {
    pool: PgPool,
}

impl Clone for LiquidationProcessRepo {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
        }
    }
}

impl LiquidationProcessRepo {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }
}
