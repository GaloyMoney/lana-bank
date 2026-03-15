use es_entity::clock::ClockHandle;
use sqlx::PgPool;

use es_entity::*;

use crate::primitives::*;

use super::entity::*;

#[derive(EsRepo, Clone)]
#[es_repo(
    entity = "PriceProvider",
    columns(name(ty = "String", list_by), provider(ty = "String", find_by)),
    tbl_prefix = "core"
)]
pub(crate) struct PriceProviderRepo {
    pool: PgPool,
    clock: ClockHandle,
}

impl PriceProviderRepo {
    pub(crate) fn new(pool: &PgPool, clock: ClockHandle) -> Self {
        Self {
            pool: pool.clone(),
            clock,
        }
    }
}

impl From<(PriceProvidersSortBy, &PriceProvider)> for price_provider_cursor::PriceProvidersCursor {
    fn from(provider_with_sort: (PriceProvidersSortBy, &PriceProvider)) -> Self {
        let (sort, provider) = provider_with_sort;
        match sort {
            PriceProvidersSortBy::CreatedAt => {
                price_provider_cursor::PriceProvidersByCreatedAtCursor::from(provider).into()
            }
            PriceProvidersSortBy::Id => {
                price_provider_cursor::PriceProvidersByIdCursor::from(provider).into()
            }
            PriceProvidersSortBy::Name => {
                price_provider_cursor::PriceProvidersByNameCursor::from(provider).into()
            }
        }
    }
}
