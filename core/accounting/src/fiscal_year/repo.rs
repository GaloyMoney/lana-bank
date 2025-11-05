use chrono::{DateTime, Utc};
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
        last_month_closed_at(ty = "Option<DateTime<Utc>>", create(persist = false)),
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
