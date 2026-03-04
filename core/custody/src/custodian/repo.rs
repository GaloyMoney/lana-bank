use es_entity::clock::ClockHandle;
use sqlx::PgPool;

use es_entity::*;

use crate::primitives::*;

use super::{entity::*, error::*};

#[derive(EsRepo, Clone)]
#[es_repo(
    entity = "Custodian",
    columns(name(ty = "String", list_by), provider(ty = "String", find_by)),
    tbl_prefix = "core"
)]
pub(crate) struct CustodianRepo {
    pool: PgPool,
    clock: ClockHandle,
}

impl CustodianRepo {
    pub(crate) fn new(pool: &PgPool, clock: ClockHandle) -> Self {
        Self {
            pool: pool.clone(),
            clock,
        }
    }

    pub async fn list_all_in_op(
        &self,
        op: &mut impl es_entity::AtomicOperation,
    ) -> Result<Vec<Custodian>, CustodianError> {
        let mut custodians = Vec::new();
        let mut next = Some(PaginatedQueryArgs::default());

        while let Some(query) = next.take() {
            let mut ret = self
                .list_by_id_in_op(&mut *op, query, Default::default())
                .await?;

            custodians.append(&mut ret.entities);
            next = ret.into_next_query();
        }

        Ok(custodians)
    }
}

impl From<(CustodiansSortBy, &Custodian)> for custodian_cursor::CustodiansCursor {
    fn from(custodian_with_sort: (CustodiansSortBy, &Custodian)) -> Self {
        let (sort, custodian) = custodian_with_sort;
        match sort {
            CustodiansSortBy::CreatedAt => {
                custodian_cursor::CustodiansByCreatedAtCursor::from(custodian).into()
            }
            CustodiansSortBy::Id => custodian_cursor::CustodiansByIdCursor::from(custodian).into(),
            CustodiansSortBy::Name => {
                custodian_cursor::CustodiansByNameCursor::from(custodian).into()
            }
        }
    }
}
