use sqlx::PgPool;

use es_entity::*;

use crate::primitives::{CreditFacilityId, ObligationId};

use super::{entity::*, error::*};

#[derive(EsRepo)]
#[es_repo(
    entity = "Obligation",
    err = "ObligationError",
    columns(
        credit_facility_id(ty = "CreditFacilityId", list_for, update(persist = false)),
        reference(ty = "String", create(accessor = "reference()")),
    ),
    tbl_prefix = "core"
)]
pub struct ObligationRepo {
    pool: PgPool,
}

impl Clone for ObligationRepo {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
        }
    }
}

impl ObligationRepo {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }
}
