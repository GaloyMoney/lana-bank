use sqlx::PgPool;

use es_entity::*;

use crate::primitives::{AccountingCalendarId, ChartId};

use super::{entity::*, error::AccountingCalendarError};

#[derive(EsRepo, Clone)]
#[es_repo(
    entity = "AccountingCalendar",
    err = "AccountingCalendarError",
    columns(
        reference(ty = "String", create(accessor = "reference()")),
        chart_id(ty = "ChartId", list_for),
    ),
    tbl_prefix = "core"
)]
pub struct AccountingCalendarRepo {
    pool: PgPool,
}

impl AccountingCalendarRepo {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }
}
