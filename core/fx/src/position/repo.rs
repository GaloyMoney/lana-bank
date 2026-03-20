use es_entity::clock::ClockHandle;
use sqlx::PgPool;

use es_entity::*;

use crate::primitives::FxPositionId;

use super::entity::*;

#[derive(EsRepo, Clone)]
#[es_repo(
    entity = "FxPosition",
    columns(currency(ty = "String", list_by)),
    tbl_prefix = "core"
)]
pub(super) struct FxPositionRepo {
    pool: PgPool,
    clock: ClockHandle,
}

impl FxPositionRepo {
    pub fn new(pool: &PgPool, clock: ClockHandle) -> Self {
        Self {
            pool: pool.clone(),
            clock,
        }
    }
}

impl From<(FxPositionsSortBy, &FxPosition)> for fx_position_cursor::FxPositionsCursor {
    fn from(position_with_sort: (FxPositionsSortBy, &FxPosition)) -> Self {
        let (sort, position) = position_with_sort;
        match sort {
            FxPositionsSortBy::CreatedAt => {
                fx_position_cursor::FxPositionsByCreatedAtCursor::from(position).into()
            }
            FxPositionsSortBy::Id => {
                fx_position_cursor::FxPositionsByIdCursor::from(position).into()
            }
            FxPositionsSortBy::Currency => {
                fx_position_cursor::FxPositionsByCurrencyCursor::from(position).into()
            }
        }
    }
}
