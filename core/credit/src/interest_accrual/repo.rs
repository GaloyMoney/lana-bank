use sqlx::PgPool;

use es_entity::*;

use crate::{
    data_export::Export,
    primitives::{CreditFacilityId, InterestAccrualId, InterestAccrualIdx},
};

use super::{entity::*, InterestAccrualError};

#[derive(EsRepo, Clone)]
#[es_repo(
    entity = "InterestAccrual",
    err = "InterestAccrualError",
    columns(
        credit_facility_id(ty = "CreditFacilityId", update(persist = false), list_for, parent),
        idx(ty = "InterestAccrualIdx", update(persist = false)),
    )
)]
pub(in crate::credit_facility) struct InterestAccrualRepo {
    pool: PgPool,
    export: Export,
}

impl InterestAccrualRepo {
    pub fn new(pool: &PgPool, export: &Export) -> Self {
        Self {
            pool: pool.clone(),
            export: export.clone(),
        }
    }
}
