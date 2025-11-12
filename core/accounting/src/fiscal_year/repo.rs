use es_entity::*;
use sqlx::PgPool;

use super::{
    entity::{FiscalYear, FiscalYearEvent},
    error::FiscalYearError,
};
use crate::primitives::FiscalYearId;

#[derive(EsRepo, Clone)]
#[es_repo(
    entity = "FiscalYear",
    err = "FiscalYearError",
    columns(
        chart_reference(ty = "String", list_for),
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
}
