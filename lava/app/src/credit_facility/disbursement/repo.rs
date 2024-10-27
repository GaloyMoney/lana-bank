use sqlx::PgPool;

use es_entity::*;

use crate::{
    data_export::Export,
    primitives::{CreditFacilityId, DisbursementId, DisbursementIdx},
};

use super::{entity::*, error::DisbursementError};

#[derive(EsRepo, Clone)]
#[es_repo(
    entity = "Disbursement",
    err = "DisbursementError",
    columns(
        credit_facility_id(ty = "CreditFacilityId", update(persist = false), list_for),
        idx(ty = "DisbursementIdx", update(persist = false)),
    )
)]
pub(in crate::credit_facility) struct DisbursementRepo {
    pool: PgPool,
    _export: Export,
}

impl DisbursementRepo {
    pub fn new(pool: &PgPool, export: &Export) -> Self {
        Self {
            pool: pool.clone(),
            _export: export.clone(),
        }
    }
}
