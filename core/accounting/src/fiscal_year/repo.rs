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
        year(ty = "String", create(accessor = "year()"), find_by),
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
}
