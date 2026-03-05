use chrono::NaiveDate;
use es_entity::clock::ClockHandle;
use es_entity::*;
use sqlx::PgPool;

use super::{
    entity::{FiscalYear, FiscalYearEvent, FiscalYearReference},
    error::FiscalYearError,
};
use crate::primitives::{ChartId, FiscalYearId};

#[derive(EsRepo, Clone)]
#[es_repo(
    entity = "FiscalYear",
    columns(
        chart_id(ty = "ChartId", update(persist = false), list_for),
        reference(ty = "String", create(accessor = "reference()")),
        opened_as_of(ty = "NaiveDate", list_by, create(accessor = "opened_as_of")),
    ),
    tbl_prefix = "core"
)]
pub struct FiscalYearRepo {
    pool: PgPool,
    clock: ClockHandle,
}

impl FiscalYearRepo {
    pub fn new(pool: &PgPool, clock: ClockHandle) -> Self {
        Self {
            pool: pool.clone(),
            clock,
        }
    }

    #[tracing::instrument(
        name = "core_accounting.fiscal_year_repo.maybe_find_by_chart_id_and_year",
        skip(self),
        fields(chart_id = %chart_id, year)
    )]
    pub async fn maybe_find_by_chart_id_and_year(
        &self,
        chart_id: ChartId,
        year: &str,
    ) -> Result<Option<FiscalYear>, FiscalYearError> {
        let reference = FiscalYearReference::try_new(chart_id, year)?;
        match self.find_by_reference(reference).await {
            Err(e) if e.was_not_found() => Ok(None),
            Err(e) => Err(e.into()),
            Ok(res) => Ok(Some(res)),
        }
    }
}

impl From<(FiscalYearsSortBy, &FiscalYear)> for fiscal_year_cursor::FiscalYearsCursor {
    fn from(fy_with_sort: (FiscalYearsSortBy, &FiscalYear)) -> Self {
        let (sort, fy) = fy_with_sort;
        match sort {
            FiscalYearsSortBy::CreatedAt => {
                fiscal_year_cursor::FiscalYearsByCreatedAtCursor::from(fy).into()
            }
            FiscalYearsSortBy::Id => fiscal_year_cursor::FiscalYearsByIdCursor::from(fy).into(),
            FiscalYearsSortBy::OpenedAsOf => {
                fiscal_year_cursor::FiscalYearsByOpenedAsOfCursor::from(fy).into()
            }
        }
    }
}
