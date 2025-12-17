use es_entity::*;
use sqlx::PgPool;

use super::{
    entity::{FiscalYear, FiscalYearEvent},
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
        year(ty = "String", create(accessor = "year()")),
    ),
    tbl_prefix = "core"
)]
pub struct FiscalYearRepo {
    pool: PgPool,
}

impl FiscalYearRepo {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
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
        let res = es_entity::es_query!(
            entity = FiscalYear,
            r#"
            SELECT * 
            FROM core_fiscal_years
            WHERE chart_id = $1 
            AND year = $2
            "#,
            chart_id as ChartId,
            year as &str
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(res)
    }
}
