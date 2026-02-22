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
    err = "FiscalYearError",
    columns(
        chart_id(ty = "ChartId", update(persist = false), list_for),
        reference(ty = "String", create(accessor = "reference()")),
    ),
    tbl_prefix = "core",
    list_for_by = false
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
            Err(e) => Err(e),
            Ok(res) => Ok(Some(res)),
        }
    }
}
