use sqlx::PgPool;

use es_entity::*;

use crate::primitives::*;

use super::{entity::*, error::DisbursalError};

#[derive(EsRepo, Clone)]
#[es_repo(
    entity = "Disbursal",
    err = "DisbursalError",
    columns(
        credit_facility_id(ty = "CreditFacilityId", list_for, update(persist = false)),
        approval_process_id(ty = "ApprovalProcessId", list_by, update(persist = "false")),
        concluded_tx_id(ty = "Option<LedgerTxId>", create(persist = false)),
        idx(ty = "DisbursalIdx", list_by, update(persist = false)),
    )
)]
pub(in crate::credit_facility) struct DisbursalRepo {
    pool: PgPool,
}

impl DisbursalRepo {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }
}

impl From<(DisbursalsSortBy, &Disbursal)> for disbursal_cursor::DisbursalsCursor {
    fn from(disbursal_with_sort: (DisbursalsSortBy, &Disbursal)) -> Self {
        let (sort, disbursal) = disbursal_with_sort;
        match sort {
            DisbursalsSortBy::CreatedAt => {
                disbursal_cursor::DisbursalsByCreatedAtCursor::from(disbursal).into()
            }
            DisbursalsSortBy::Idx => {
                disbursal_cursor::DisbursalsByIdxCursor::from(disbursal).into()
            }
            DisbursalsSortBy::ApprovalProcessId => {
                disbursal_cursor::DisbursalsByApprovalProcessIdCursor::from(disbursal).into()
            }
            DisbursalsSortBy::Id => disbursal_cursor::DisbursalsByIdCursor::from(disbursal).into(),
        }
    }
}
