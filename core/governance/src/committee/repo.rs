use es_entity::clock::ClockHandle;
use sqlx::PgPool;

use es_entity::*;

use crate::primitives::CommitteeId;

use super::entity::*;

#[derive(EsRepo, Clone)]
#[es_repo(
    entity = "Committee",
    columns(name(ty = "String", list_by)),
    tbl_prefix = "core"
)]
pub struct CommitteeRepo {
    #[allow(dead_code)]
    pool: PgPool,
    clock: ClockHandle,
}

impl CommitteeRepo {
    pub fn new(pool: &PgPool, clock: ClockHandle) -> Self {
        Self {
            pool: pool.clone(),
            clock,
        }
    }
}

impl From<(CommitteesSortBy, &Committee)> for committee_cursor::CommitteesCursor {
    fn from(committee_with_sort: (CommitteesSortBy, &Committee)) -> Self {
        let (sort, committee) = committee_with_sort;
        match sort {
            CommitteesSortBy::CreatedAt => {
                committee_cursor::CommitteesByCreatedAtCursor::from(committee).into()
            }
            CommitteesSortBy::Id => committee_cursor::CommitteesByIdCursor::from(committee).into(),
            CommitteesSortBy::Name => {
                committee_cursor::CommitteesByNameCursor::from(committee).into()
            }
        }
    }
}
