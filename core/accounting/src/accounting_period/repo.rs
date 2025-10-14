use sqlx::PgPool;

use es_entity::*;

use crate::primitives::AccountingPeriodId;

use super::{entity::*, error::AccountingPeriodError};

#[derive(EsRepo, Clone)]
#[es_repo(
    entity = "AccountingPeriod",
    err = "AccountingPeriodError",
    // columns(chart_id(ty = "ChartId", update(persist = false), parent)),
    tbl_prefix = "core"
)]
pub struct AccountingPeriodRepo {
    pool: PgPool,
}
