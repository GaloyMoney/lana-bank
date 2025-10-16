use sqlx::PgPool;
use chrono::{DateTime, Utc};
use es_entity::*;

use crate::primitives::{AccountingPeriodId, ChartId};

use super::{entity::{AccountingPeriod, AccountingPeriodEvent}, error::AccountingPeriodError};

#[derive(EsRepo, Clone)]
#[es_repo(
    entity = "AccountingPeriod",
    err = "AccountingPeriodError",
    columns(
        chart_id(ty = "ChartId", update(persist = false)),
        closed_as_of(ty = "Option<DateTime<Utc>>", update(persist = false)),
    ),
    tbl_prefix = "core"
)]
pub struct AccountingPeriodRepo {
    pool: PgPool,
}
