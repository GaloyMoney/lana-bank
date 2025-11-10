use sqlx::PgPool;

use es_entity::*;

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
        chart_id(ty = "ChartId", update(persist = false), list_for, parent),
        reference(ty = "String", create(accessor = "reference()")),
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
    pub async fn find_current_by_chart_id(
        &self,
        chart_id: ChartId,
    ) -> Result<FiscalYear, FiscalYearError> {
        self.list_for_chart_id_by_created_at(
            chart_id,
            Default::default(),
            es_entity::ListDirection::Descending,
        )
        .await?
        .entities
        .first()
        .cloned()
        .ok_or(FiscalYearError::CurrentYearNotFound)
    }
}
